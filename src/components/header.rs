use yew::prelude::*;

#[function_component(Header)]
pub fn header() -> Html {
    html! {
        <header class="header">
            <div class="header-inner">
                <div class="header-logo">
                    <span class="header-icon">{"⚡"}</span>
                    <h1 class="header-title">{"GitHub CI/CD Checker"}</h1>
                </div>
                <nav class="header-nav">
                    <a href="https://github.com" target="_blank" rel="noopener" class="header-link">
                        {"GitHub"}
                    </a>
                </nav>
            </div>
        </header>
    }
}
