use yew::prelude::*;

#[function_component(Footer)]
pub fn footer() -> Html {
    html! {
        <footer class="footer">
            <p>
                {"Propulsé par Rust + WebAssembly • "}
                <a href="https://github.com" target="_blank" rel="noopener">
                    {"Code source"}
                </a>
            </p>
        </footer>
    }
}
