use yew::prelude::*;

use crate::checks::CheckEngine;
use crate::models::ScoreReport;
use crate::services::GithubClient;

use super::footer::Footer;
use super::header::Header;
use super::results::Results;
use super::search_bar::SearchBar;

/// Application state
#[derive(Debug, Clone, PartialEq)]
pub enum AnalysisState {
    Idle,
    Loading,
    Done(ScoreReport),
    Error(String),
}

#[function_component(App)]
pub fn app() -> Html {
    let state = use_state(|| AnalysisState::Idle);
    let token = use_state(|| Option::<String>::None);

    let on_analyze = {
        let state = state.clone();
        let token = token.clone();
        Callback::from(move |(url, pat): (String, Option<String>)| {
            let state = state.clone();
            token.set(pat.clone());
            let pat = pat.clone();

            state.set(AnalysisState::Loading);

            wasm_bindgen_futures::spawn_local(async move {
                let client = GithubClient::new(pat);
                let repo = match GithubClient::parse_repo_url(&url) {
                    Ok(r) => r,
                    Err(e) => {
                        state.set(AnalysisState::Error(e));
                        return;
                    }
                };

                let engine = CheckEngine::new(client);
                match engine.analyze(&repo).await {
                    Ok(report) => state.set(AnalysisState::Done(report)),
                    Err(e) => state.set(AnalysisState::Error(e)),
                }
            });
        })
    };

    let on_reset = {
        let state = state.clone();
        Callback::from(move |_: ()| {
            state.set(AnalysisState::Idle);
        })
    };

    html! {
        <div class="app">
            <Header />
            <main class="main-content">
                <SearchBar
                    on_analyze={on_analyze}
                    is_loading={*state == AnalysisState::Loading}
                />

                { match &*state {
                    AnalysisState::Idle => html! {
                        <div class="hero-section">
                            <div class="hero-icon">{"🔍"}</div>
                            <h2 class="hero-title">
                                {"Analysez la qualité CI/CD de n'importe quel repo GitHub"}
                            </h2>
                            <p class="hero-subtitle">
                                {"Entrez l'URL d'un dépôt GitHub pour obtenir un score détaillé de sa pipeline CI/CD, avec des recommandations d'amélioration."}
                            </p>
                        </div>
                    },
                    AnalysisState::Loading => html! {
                        <div class="loading-section">
                            <div class="loading-spinner"></div>
                            <p class="loading-text">{"Analyse en cours..."}</p>
                            <p class="loading-subtext">
                                {"Vérification des workflows, tests, sécurité, déploiement..."}
                            </p>
                        </div>
                    },
                    AnalysisState::Done(report) => html! {
                        <Results
                            report={report.clone()}
                            on_reset={on_reset.clone()}
                            token={(*token).clone()}
                        />
                    },
                    AnalysisState::Error(msg) => html! {
                        <div class="error-section">
                            <div class="error-icon">{"⚠️"}</div>
                            <h3 class="error-title">{"Erreur d'analyse"}</h3>
                            <p class="error-message">{msg}</p>
                            <button class="btn-secondary" onclick={
                                let on_reset = on_reset.clone();
                                move |_| on_reset.emit(())
                            }>
                                {"Réessayer"}
                            </button>
                        </div>
                    },
                }}
            </main>
            <Footer />
        </div>
    }
}
