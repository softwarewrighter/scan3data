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
            // TODO: Call Imagen 3 API
            // For now, just copy original to cleaned
            let mut data = (*pipeline_data).clone();
            data.cleaned_image = data.original_image.clone();
            pipeline_data.set(data);
            current_stage.set(PipelineStage::OcrExtraction);
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
