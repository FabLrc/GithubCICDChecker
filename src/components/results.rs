use yew::prelude::*;

use crate::models::{CategoryScore, CheckResult, CheckStatus, ScoreReport};

use super::score_gauge::ScoreGauge;

#[derive(Properties, PartialEq, Clone)]
pub struct ResultsProps {
    pub report: ScoreReport,
    pub on_reset: Callback<()>,
}

#[function_component(Results)]
pub fn results(props: &ResultsProps) -> Html {
    let report = &props.report;

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
