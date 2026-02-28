use yew::prelude::*;

use crate::models::ai_review::{AiReview, AiReviewState};

// ── Props ────────────────────────────────────────────────────────────────────

#[derive(Properties, PartialEq)]
pub struct AiReviewPanelProps {
    /// Current state of the AI review process.
    pub state: AiReviewState,
    /// Callback triggered when the user clicks "Request AI review".
    pub on_request: Callback<()>,
    /// Whether a GitHub PAT was provided (gates the feature).
    pub has_token: bool,
}

// ── Component ────────────────────────────────────────────────────────────────

#[function_component(AiReviewPanel)]
pub fn ai_review_panel(props: &AiReviewPanelProps) -> Html {
    html! {
        <section class="ai-review-panel" aria-label="Analyse IA">
            <div class="ai-panel-header">
                <span class="ai-panel-icon" aria-hidden="true">{"🤖"}</span>
                <h3 class="ai-panel-title">{"Analyse IA"}</h3>
                <span class="ai-panel-badge">{"GitHub Models"}</span>
            </div>

            { match &props.state {
                AiReviewState::Idle          => render_idle(props.has_token, props.on_request.clone()),
                AiReviewState::Loading       => render_loading(),
                AiReviewState::Done(review)  => render_review(review),
                AiReviewState::Unavailable   => render_unavailable(),
                AiReviewState::Error(msg)    => render_error(msg, props.on_request.clone()),
            }}
        </section>
    }
}

// ── State renderers ───────────────────────────────────────────────────────────

fn render_idle(has_token: bool, on_request: Callback<()>) -> Html {
    if !has_token {
        return render_unavailable();
    }
    html! {
        <div class="ai-state ai-state--idle">
            <p class="ai-idle-text">
                {"Obtenez des recommandations contextuelles générées par IA \
                  basées sur vos checks échoués et votre workflow CI."}
            </p>
            <button
                class="btn-ai-request"
                onclick={move |_| on_request.emit(())}
            >
                <span aria-hidden="true">{"✨"}</span>
                {" Demander l'analyse IA"}
            </button>
        </div>
    }
}

fn render_loading() -> Html {
    html! {
        <div class="ai-state ai-state--loading" role="status" aria-live="polite">
            <div class="ai-spinner" aria-hidden="true" />
            <p>{"Analyse en cours par l'IA…"}</p>
        </div>
    }
}

fn render_unavailable() -> Html {
    html! {
        <div class="ai-state ai-state--unavailable">
            <p>
                <span aria-hidden="true">{"🔑 "}</span>
                {"Fournissez un "}
                <strong>{"GitHub Personal Access Token"}</strong>
                {" pour activer l'analyse IA via GitHub Models."}
            </p>
        </div>
    }
}

fn render_error(msg: &str, on_request: Callback<()>) -> Html {
    html! {
        <div class="ai-state ai-state--error" role="alert">
            <p class="ai-error-text">
                <span aria-hidden="true">{"⚠️ "}</span>
                {msg}
            </p>
            <button
                class="btn-secondary btn-sm"
                onclick={move |_| on_request.emit(())}
            >
                {"Réessayer"}
            </button>
        </div>
    }
}

fn render_review(review: &AiReview) -> Html {
    html! {
        <div class="ai-state ai-state--done">
            <p class="ai-summary">{&review.summary}</p>
            <div class="ai-recommendations">
                { for review.recommendations.iter().map(|rec| {
                    let bg   = format!("{}22", rec.priority.color());
                    let fg   = rec.priority.color().to_string();
                    html! {
                        <article class="ai-rec-card">
                            <div class="ai-rec-header">
                                <span
                                    class="ai-rec-priority"
                                    style={format!("background:{};color:{}", bg, fg)}
                                >
                                    {rec.priority.label()}
                                </span>
                                <h4 class="ai-rec-title">{&rec.title}</h4>
                            </div>
                            <p class="ai-rec-description">{&rec.description}</p>
                        </article>
                    }
                })}
            </div>
        </div>
    }
}
