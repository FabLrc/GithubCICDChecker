use gloo_net::http::Request;
use serde::{Deserialize, Serialize};

use crate::models::ai_review::AiReview;
use crate::models::{CheckStatus, ScoreReport};

const GITHUB_MODELS_ENDPOINT: &str =
    "https://models.inference.ai.azure.com/chat/completions";
const AI_MODEL: &str = "gpt-4.1-mini";
const MAX_YAML_CHARS: usize = 3_000;
const MAX_AI_TOKENS: u32 = 1_500;

// ── Request DTOs ────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct ChatRequest {
    model: &'static str,
    messages: Vec<ChatMessage>,
    temperature: f32,
    max_tokens: u32,
    response_format: ResponseFormat,
}

#[derive(Serialize)]
struct ChatMessage {
    role: &'static str,
    content: String,
}

#[derive(Serialize)]
struct ResponseFormat {
    #[serde(rename = "type")]
    format_type: &'static str,
}

// ── Response DTOs ────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessageContent,
}

#[derive(Deserialize)]
struct ChatMessageContent {
    content: String,
}

// ── Client ───────────────────────────────────────────────────────────────────

/// Thin client wrapping the GitHub Models (OpenAI-compatible) API.
///
/// Construction fails gracefully: `new` returns `None` when no token is
/// available so callers can display the "unavailable" state without any
/// additional branching.
pub struct AiClient {
    token: String,
}

impl AiClient {
    /// Returns `None` when no GitHub PAT is provided.
    pub fn new(token: Option<String>) -> Option<Self> {
        token.map(|t| Self { token: t })
    }

    // ── Prompt builder ───────────────────────────────────────────────────

    /// Builds the user prompt from the score report and an optional workflow
    /// YAML snippet.  The YAML is truncated to avoid exceeding context limits.
    pub fn build_prompt(report: &ScoreReport, workflow_yaml: Option<&str>) -> String {
        let failed_checks = Self::collect_failed_checks(report);

        let yaml_section = workflow_yaml
            .map(|yaml| {
                let snippet = if yaml.len() > MAX_YAML_CHARS {
                    format!("{}… (tronqué)", &yaml[..MAX_YAML_CHARS])
                } else {
                    yaml.to_string()
                };
                format!("\n\n## Workflow CI principal (YAML)\n```yaml\n{}\n```", snippet)
            })
            .unwrap_or_default();

        let failed_summary = if failed_checks.is_empty() {
            "Aucun check échoué 🎉".to_string()
        } else {
            failed_checks.join("\n")
        };

        let json_schema = concat!(
            "{\n",
            "  \"summary\": \"Résumé global en 2-3 phrases\",\n",
            "  \"recommendations\": [\n",
            "    {\n",
            "      \"title\": \"Titre court\",\n",
            "      \"description\": \"Description détaillée et actionable\",\n",
            "      \"priority\": \"high\" | \"medium\" | \"low\"\n",
            "    }\n",
            "  ]\n",
            "}"
        );

        format!(
            "Analyse le rapport CI/CD du dépôt GitHub `{}` et fournis des recommandations concrètes.\n\n\
             ## Checks échoués ({} sur {})\n\
             {}\
             {}\n\n\
             Réponds en JSON avec ce format exact :\n\
             {}\n\n\
             Donne 3 à 6 recommandations priorisées par impact. \
             Réponds uniquement en JSON valide, sans texte supplémentaire.",
            report.repository,
            report.total.saturating_sub(report.passed),
            report.total,
            failed_summary,
            yaml_section,
            json_schema,
        )
    }

    fn collect_failed_checks(report: &ScoreReport) -> Vec<String> {
        report
            .categories
            .iter()
            .flat_map(|cat| {
                cat.results.iter().filter_map(|r| {
                    if r.status == CheckStatus::Failed {
                        Some(format!(
                            "- [{}] {}: {}",
                            cat.category.label(),
                            r.check.name,
                            r.detail
                        ))
                    } else {
                        None
                    }
                })
            })
            .collect()
    }

    // ── API call ─────────────────────────────────────────────────────────

    /// Calls the GitHub Models API and returns a parsed [`AiReview`].
    pub async fn review(
        &self,
        report: &ScoreReport,
        workflow_yaml: Option<&str>,
    ) -> Result<AiReview, String> {
        let user_content = Self::build_prompt(report, workflow_yaml);

        let payload = ChatRequest {
            model: AI_MODEL,
            messages: vec![
                ChatMessage {
                    role: "system",
                    content: "Tu es un expert DevOps et CI/CD. \
                              Tu analyses des pipelines GitHub et fournis des recommandations \
                              techniques précises et actionnables. \
                              Tu réponds toujours en JSON valide."
                        .to_string(),
                },
                ChatMessage {
                    role: "user",
                    content: user_content,
                },
            ],
            temperature: 0.3,
            max_tokens: MAX_AI_TOKENS,
            response_format: ResponseFormat {
                format_type: "json_object",
            },
        };

        let body =
            serde_json::to_string(&payload).map_err(|e| format!("Serialization error: {}", e))?;

        let response = Request::post(GITHUB_MODELS_ENDPOINT)
            .header("Content-Type", "application/json")
            .header("Authorization", &format!("Bearer {}", self.token))
            .header("User-Agent", "github-cicd-checker")
            .body(body)
            .map_err(|e| format!("Request build error: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        let status = response.status();
        if status != 200 {
            let error_body = response.text().await.unwrap_or_default();
            let user_message = if status == 401 {
                "Token invalide ou permission manquante. \
                 Assurez-vous d'utiliser un fine-grained token avec la permission \
                 \"Models\" (Read-only) activée.".to_string()
            } else if status == 403 {
                "Accès refusé. Vérifiez que votre token a la permission \
                 \"Models\" et que vous avez accès à GitHub Models.".to_string()
            } else {
                format!("Erreur API {} : {}", status, error_body)
            };
            return Err(user_message);
        }

        let chat: ChatResponse = response
            .json()
            .await
            .map_err(|e| format!("Response parse error: {}", e))?;

        let raw_content = chat
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .ok_or_else(|| "Empty response from AI model".to_string())?;

        serde_json::from_str::<AiReview>(&raw_content).map_err(|e| {
            format!(
                "AI JSON parse error: {} — Réponse reçue : {}",
                e, raw_content
            )
        })
    }
}
