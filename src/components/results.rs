use yew::prelude::*;

use crate::models::{AiReviewState, CategoryScore, CheckResult, CheckStatus, ScoreReport};
use crate::services::{AiClient, GithubClient, RepoIdentifier};

use super::ai_review_panel::AiReviewPanel;
use super::score_gauge::ScoreGauge;

#[derive(Properties, PartialEq, Clone)]
pub struct ResultsProps {
    pub report: ScoreReport,
    pub on_reset: Callback<()>,
    /// Optional GitHub PAT — required to activate the AI review feature.
    #[prop_or_default]
    pub token: Option<String>,
}

#[function_component(Results)]
pub fn results(props: &ResultsProps) -> Html {
    let report = &props.report;
    let ai_state = use_state(|| {
        if props.token.is_some() {
            AiReviewState::Idle
        } else {
            AiReviewState::Unavailable
        }
    });

    let on_ai_request = {
        let ai_state = ai_state.clone();
        let report = props.report.clone();
        let token = props.token.clone();

        Callback::from(move |_: ()| {
            let ai_state = ai_state.clone();
            let report = report.clone();
            let token = token.clone();

            let Some(client) = AiClient::new(token) else {
                ai_state.set(AiReviewState::Unavailable);
                return;
            };

            ai_state.set(AiReviewState::Loading);

            wasm_bindgen_futures::spawn_local(async move {
                // Try to retrieve the first workflow YAML to enrich the prompt.
                let workflow_yaml = fetch_first_workflow_yaml(&report.repository).await;

                match client.review(&report, workflow_yaml.as_deref()).await {
                    Ok(review) => ai_state.set(AiReviewState::Done(review)),
                    Err(err)   => ai_state.set(AiReviewState::Error(err)),
                }
            });
        })
    };

    html! {
        <div class="results-section">
            // ── Top bar with repo name ──
            <div class="results-header">
                <div class="results-repo">
                    <span class="results-repo-icon">{"📦"}</span>
                    <a
                        href={format!("https://github.com/{}", report.repository)}
                        target="_blank"
                        rel="noopener"
                        class="results-repo-link"
                    >
                        {&report.repository}
                    </a>
                </div>
                <button class="btn-secondary" onclick={
                    let on_reset = props.on_reset.clone();
                    move |_| on_reset.emit(())
                }>
                    {"← Nouvelle analyse"}
                </button>
            </div>

            // ── Score gauge (PageSpeed style) ──
            <div class="results-score-area">
                <ScoreGauge passed={report.passed} total={report.total} />
            </div>

            // ── AI Review Panel ──
            <AiReviewPanel
                state={(*ai_state).clone()}
                on_request={on_ai_request}
                has_token={props.token.is_some()}
            />

            // ── Category breakdown ──
            <div class="categories-grid">
                { for report.categories.iter().map(|cat| html! {
                    <CategoryCard category={cat.clone()} />
                })}
            </div>

            // ── Timestamp ──
            <p class="results-timestamp">
                {format!("Analysé le {}", &report.analyzed_at)}
            </p>
        </div>
    }
}

/// Attempts to fetch the raw content of the first `.yml` file found under
/// `.github/workflows/`.  Returns `None` on any error so the AI prompt is
/// simply sent without a YAML snippet.
async fn fetch_first_workflow_yaml(repository: &str) -> Option<String> {
    let parts: Vec<&str> = repository.splitn(2, '/').collect();
    if parts.len() != 2 {
        return None;
    }
    let repo_id = RepoIdentifier {
        owner: parts[0].to_string(),
        repo: parts[1].to_string(),
    };

    let client = GithubClient::new(None);
    let files = client.fetch_workflow_files(&repo_id).await.ok()?;
    let first_yml = files.into_iter().find(|f| {
        f.name.ends_with(".yml") || f.name.ends_with(".yaml")
    })?;

    client
        .fetch_raw_file(&repo_id, &first_yml.path)
        .await
        .ok()
}

// ── Category Card ──

#[derive(Properties, PartialEq, Clone)]
struct CategoryCardProps {
    category: CategoryScore,
}

#[function_component(CategoryCard)]
fn category_card(props: &CategoryCardProps) -> Html {
    let cat = &props.category;
    let expanded = use_state(|| true);

    let toggle = {
        let expanded = expanded.clone();
        Callback::from(move |_: MouseEvent| {
            expanded.set(!*expanded);
        })
    };

    let pct = cat.percentage();
    let color = if pct >= 90.0 {
        "#0cce6b"
    } else if pct >= 50.0 {
        "#ffa400"
    } else {
        "#ff4e42"
    };

    let bar_width = format!("{}%", pct.min(100.0));

    html! {
        <div class="category-card">
            <div class="category-header" onclick={toggle}>
                <div class="category-title-area">
                    <span class="category-icon">{cat.category.icon()}</span>
                    <h3 class="category-title">{cat.category.label()}</h3>
                </div>
                <div class="category-score-area">
                    <span class="category-score" style={format!("color: {}", color)}>
                        {format!("{}/{}", cat.passed, cat.total)}
                    </span>
                    <div class="category-bar-bg">
                        <div
                            class="category-bar-fill"
                            style={format!("width: {}; background: {}", bar_width, color)}
                        />
                    </div>
                    <span class="category-chevron">
                        if *expanded { {"▾"} } else { {"▸"} }
                    </span>
                </div>
            </div>

            if *expanded {
                <div class="category-checks">
                    { for cat.results.iter().map(|r| html! {
                        <CheckRow result={r.clone()} />
                    })}
                </div>
            }
        </div>
    }
}

// ── Check Row ──

#[derive(Properties, PartialEq, Clone)]
struct CheckRowProps {
    result: CheckResult,
}

#[function_component(CheckRow)]
fn check_row(props: &CheckRowProps) -> Html {
    let r = &props.result;
    let show_detail = use_state(|| false);

    let toggle = {
        let show_detail = show_detail.clone();
        Callback::from(move |_: MouseEvent| {
            show_detail.set(!*show_detail);
        })
    };

    let (status_icon, status_class) = match r.status {
        CheckStatus::Passed => ("✓", "check-passed"),
        CheckStatus::Failed => ("✗", "check-failed"),
        CheckStatus::Warning => ("!", "check-warning"),
        CheckStatus::Skipped => ("—", "check-skipped"),
    };

    html! {
        <div class={classes!("check-row", status_class)} onclick={toggle}>
            <div class="check-row-main">
                <span class="check-status-icon">{status_icon}</span>
                <div class="check-info">
                    <span class="check-name">{&r.check.name}</span>
                </div>
            </div>

            if *show_detail {
                <div class="check-detail">
                    <p class="check-detail-text">{&r.detail}</p>
                    if let Some(ref suggestion) = r.suggestion {
                        <div class="check-suggestion">
                            <span class="suggestion-icon">{"💡"}</span>
                            <span>{suggestion}</span>
                        </div>
                    }
                </div>
            }
        </div>
    }
}
