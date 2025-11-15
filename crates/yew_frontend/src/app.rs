//! Main application component

use crate::components::upload::UploadComponent;
use yew::prelude::*;

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <div class="app">
            <header class="app-header">
                <h1>{ "scan3data" }</h1>
                <p>{ "Three-Phase Pipeline: Scan → Classify & Correct → Convert" }</p>
            </header>
            <main class="app-main">
                <UploadComponent />
            </main>
            <footer class="app-footer">
                <p>{ "Phase 1: Scan (non-LLM baseline) | Phase 2: Classify & Correct | Phase 3: Convert" }</p>
            </footer>
        </div>
    }
}
