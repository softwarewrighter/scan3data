//! scan3data Yew frontend

mod app;
mod components;

pub use app::App;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn run_app() {
    // Set up console error panic hook for better debugging
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    yew::Renderer::<App>::new().render();
}
