//! Main application component

use crate::components::upload::UploadComponent;
use yew::prelude::*;

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <div class="app">
            <header class="app-header">
                <h1>{ "scan2data" }</h1>
                <p>{ "IBM 1130 Punch Card and Listing Scanner" }</p>
            </header>
            <main class="app-main">
                <UploadComponent />
            </main>
            <footer class="app-footer">
                <p>{ "Phase 1: Non-LLM baseline" }</p>
            </footer>
        </div>
    }
}
