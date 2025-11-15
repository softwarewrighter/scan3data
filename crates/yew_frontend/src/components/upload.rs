//! File upload component

use web_sys::{Event, HtmlInputElement};
use yew::prelude::*;

#[function_component(UploadComponent)]
pub fn upload_component() -> Html {
    let files_state = use_state(|| Vec::<String>::new());

    let on_file_change = {
        let files_state = files_state.clone();
        Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            if let Some(file_list) = input.files() {
                let mut files = Vec::new();
                for i in 0..file_list.length() {
                    if let Some(file) = file_list.get(i) {
                        files.push(file.name());
                    }
                }
                files_state.set(files);
            }
        })
    };

    html! {
        <div class="upload-component">
            <h2>{ "Upload Scans" }</h2>
            <input
                type="file"
                multiple=true
                accept="image/*,.pdf"
                onchange={on_file_change}
            />
            <div class="file-list">
                <h3>{ "Selected Files:" }</h3>
                <ul>
                    { for files_state.iter().map(|name| {
                        html! { <li>{ name }</li> }
                    })}
                </ul>
            </div>
            <button>{ "Process" }</button>
        </div>
    }
}
