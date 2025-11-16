//! Main application component

use crate::components::pipeline::{Pipeline, PipelineData, PipelineStage};
use yew::prelude::*;

#[function_component(App)]
pub fn app() -> Html {
    let pipeline_data = use_state(PipelineData::default);
    let current_stage = use_state(|| PipelineStage::Upload);

    // Callbacks for pipeline stages
    let on_upload = {
        let pipeline_data = pipeline_data.clone();
        let current_stage = current_stage.clone();
        Callback::from(move |image_bytes: Vec<u8>| {
            // Convert to base64 data URL
            let base64 =
                base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &image_bytes);
            let data_url = format!("data:image/jpeg;base64,{}", base64);

            let mut data = (*pipeline_data).clone();
            data.original_image = Some(data_url);
            pipeline_data.set(data);
            current_stage.set(PipelineStage::ImageCleaning);
        })
    };

    let on_clean_image = {
        let pipeline_data = pipeline_data.clone();
        let current_stage = current_stage.clone();
        Callback::from(move |_| {
            let pipeline_data = pipeline_data.clone();
            let current_stage = current_stage.clone();

            wasm_bindgen_futures::spawn_local(async move {
                // Get the original image data (format: "data:image/jpeg;base64,...")
                let data = (*pipeline_data).clone();
                if let Some(data_url) = &data.original_image {
                    // Extract base64 part after the comma
                    if let Some(base64_data) = data_url.split(',').nth(1) {
                        // Call the API endpoint
                        let request = serde_json::json!({
                            "image_data": base64_data
                        });

                        match gloo_net::http::Request::post("http://localhost:7214/api/clean-image")
                            .json(&request)
                            .unwrap()
                            .send()
                            .await
                        {
                            Ok(response) => {
                                if response.ok() {
                                    if let Ok(json) = response.json::<serde_json::Value>().await {
                                        if let Some(cleaned_b64) =
                                            json.get("cleaned_image_data").and_then(|v| v.as_str())
                                        {
                                            // Create new data URL with cleaned image
                                            let cleaned_url =
                                                format!("data:image/jpeg;base64,{}", cleaned_b64);

                                            let mut new_data = (*pipeline_data).clone();
                                            new_data.cleaned_image = Some(cleaned_url);
                                            pipeline_data.set(new_data);
                                            current_stage.set(PipelineStage::OcrExtraction);
                                        }
                                    }
                                }
                            }
                            Err(err) => {
                                gloo::console::error!("Failed to clean image:", err.to_string());
                            }
                        }
                    }
                }
            });
        })
    };

    let on_run_ocr = {
        let pipeline_data = pipeline_data.clone();
        let current_stage = current_stage.clone();
        Callback::from(move |_| {
            // TODO: Call OCR API
            // For now, set placeholder text
            let mut data = (*pipeline_data).clone();
            data.raw_ocr_text = Some("OCR text will appear here...".to_string());
            pipeline_data.set(data);
            current_stage.set(PipelineStage::Validation);
        })
    };

    let on_validate = {
        let pipeline_data = pipeline_data.clone();
        Callback::from(move |_| {
            // TODO: Run validation rules
            // For now, just mark as validated
            let data = (*pipeline_data).clone();
            pipeline_data.set(data);
        })
    };

    let on_text_edit = {
        let pipeline_data = pipeline_data.clone();
        Callback::from(move |new_text: String| {
            let mut data = (*pipeline_data).clone();
            data.raw_ocr_text = Some(new_text);
            pipeline_data.set(data);
        })
    };

    html! {
        <div class="app">
            <main class="app-main">
                <Pipeline
                    data={(*pipeline_data).clone()}
                    current_stage={(*current_stage).clone()}
                    on_upload={on_upload}
                    on_clean_image={on_clean_image}
                    on_run_ocr={on_run_ocr}
                    on_validate={on_validate}
                    on_text_edit={on_text_edit}
                />
            </main>
        </div>
    }
}
