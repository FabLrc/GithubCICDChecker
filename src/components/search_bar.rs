use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct SearchBarProps {
    pub on_analyze: Callback<(String, Option<String>)>,
    pub is_loading: bool,
}

#[function_component(SearchBar)]
pub fn search_bar(props: &SearchBarProps) -> Html {
    let url_ref = use_node_ref();
    let token_ref = use_node_ref();
    let show_token = use_state(|| false);

    let on_submit = {
        let url_ref = url_ref.clone();
        let token_ref = token_ref.clone();
        let on_analyze = props.on_analyze.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let url = url_ref
                .cast::<HtmlInputElement>()
                .map(|el| el.value())
                .unwrap_or_default();
            let token = token_ref
                .cast::<HtmlInputElement>()
                .map(|el| el.value())
                .unwrap_or_default();

            if !url.is_empty() {
                let token = if token.is_empty() { None } else { Some(token) };
                on_analyze.emit((url, token));
            }
        })
    };

    let toggle_token = {
        let show_token = show_token.clone();
        Callback::from(move |_: MouseEvent| {
            show_token.set(!*show_token);
        })
    };

    html! {
        <div class="search-section">
            <form class="search-form" onsubmit={on_submit}>
                <div class="search-input-wrapper">
                    <span class="search-icon">{"🔗"}</span>
                    <input
                        ref={url_ref}
                        type="text"
                        class="search-input"
                        placeholder="Entrez l'URL d'un repo GitHub (ex: rust-lang/rust)"
                        disabled={props.is_loading}
                        autofocus=true
                    />
                    <button
                        type="submit"
                        class="btn-analyze"
                        disabled={props.is_loading}
                    >
                        if props.is_loading {
                            <span class="btn-spinner"></span>
                            {"Analyse..."}
                        } else {
                            {"Analyser"}
                        }
                    </button>
                </div>

                <div class="token-section">
                    <button type="button" class="token-toggle" onclick={toggle_token}>
                        if *show_token {
                            {"▾ Masquer le token GitHub"}
                        } else {
                            {"▸ Token GitHub (optionnel — débloque les checks avancés)"}
                        }
                    </button>
                    if *show_token {
                        <div class="token-input-wrapper">
                            <input
                                ref={token_ref}
                                type="password"
                                class="token-input"
                                placeholder="ghp_xxxxxxxxxxxx"
                                disabled={props.is_loading}
                            />
                            <p class="token-hint">
                                {"Le token n'est jamais stocké. Il est utilisé uniquement pour les appels API dans votre navigateur."}
                            </p>
                        </div>
                    }
                </div>
            </form>
        </div>
    }
}
