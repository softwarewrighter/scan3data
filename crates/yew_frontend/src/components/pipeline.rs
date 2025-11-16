//! Pipeline visualization component for IBM 1130 OCR processing
//!
//! Displays multi-stage processing pipeline:
//! 1. Upload → 2. Image Cleaning → 3. OCR → 4. Validation

use yew::prelude::*;

/// Processing stage in the pipeline
#[derive(Clone, PartialEq, Debug)]
pub enum PipelineStage {
    Upload,
    ImageCleaning,
    OcrExtraction,
    Validation,
}

/// Data for each pipeline stage
#[derive(Clone, PartialEq, Default)]
pub struct PipelineData {
    pub original_image: Option<String>, // base64 data URL
    pub cleaned_image: Option<String>,  // base64 data URL
    pub raw_ocr_text: Option<String>,
    pub corrected_text: Option<String>,
    pub validation_errors: Vec<ValidationError>,
}

/// Validation error with line number and description
#[derive(Clone, PartialEq)]
pub struct ValidationError {
    pub line_number: usize,
    pub error_type: ErrorType,
    pub description: String,
    pub suggestion: Option<String>,
}

#[derive(Clone, PartialEq)]
#[allow(dead_code)] // Will be used when validation is implemented
pub enum ErrorType {
    SequenceError,   // Hex address out of sequence
    CharacterError,  // Wrong character (C→0, 6→0)
    WhitespaceError, // Missing column spacing
    ExtraneousChar,  // Extra dashes/hyphens
}

#[derive(Properties, PartialEq)]
pub struct PipelineProps {
    pub data: PipelineData,
    pub current_stage: PipelineStage,
    #[prop_or_default]
    pub on_upload: Callback<Vec<u8>>,
    #[prop_or_default]
    pub on_clean_image: Callback<()>,
    #[prop_or_default]
    pub on_run_ocr: Callback<()>,
    #[prop_or_default]
    pub on_validate: Callback<()>,
    #[prop_or_default]
    pub on_text_edit: Callback<String>,
}

#[function_component(Pipeline)]
pub fn pipeline(props: &PipelineProps) -> Html {
    html! {
        <div class="pipeline-container">
            <h1>{ "IBM 1130 OCR Pipeline" }</h1>

            // Stage 1: Upload
            <div class="pipeline-stage" data-testid="stage-upload">
                <h2>{ "1. Upload Image" }</h2>
                <div class="stage-content">
                    <input
                        type="file"
                        accept="image/*"
                        data-testid="file-input"
                    />
                </div>
            </div>

            // Stage 2: Image Cleaning
            if props.data.original_image.is_some() {
                <div class="pipeline-stage" data-testid="stage-cleaning">
                    <h2>{ "2. Image Cleaning" }</h2>
                    <div class="stage-content side-by-side">
                        <div class="panel">
                            <h3>{ "Original" }</h3>
                            <img
                                src={props.data.original_image.clone()}
                                alt="Original scan"
                                data-testid="original-image"
                            />
                        </div>
                        <div class="panel">
                            <h3>{ "Cleaned" }</h3>
                            if let Some(cleaned) = &props.data.cleaned_image {
                                <img
                                    src={cleaned.clone()}
                                    alt="Cleaned scan"
                                    data-testid="cleaned-image"
                                />
                            } else {
                                <button
                                    onclick={props.on_clean_image.reform(|_| ())}
                                    data-testid="clean-button"
                                >
                                    { "Clean Image" }
                                </button>
                            }
                        </div>
                    </div>
                </div>
            }

            // Stage 3: OCR Extraction
            if props.data.cleaned_image.is_some() {
                <div class="pipeline-stage" data-testid="stage-ocr">
                    <h2>{ "3. OCR Extraction" }</h2>
                    <div class="stage-content side-by-side">
                        <div class="panel">
                            <h3>{ "Cleaned Image" }</h3>
                            <img
                                src={props.data.cleaned_image.clone()}
                                alt="Cleaned scan"
                            />
                        </div>
                        <div class="panel">
                            <h3>{ "OCR Text" }</h3>
                            if let Some(ocr_text) = &props.data.raw_ocr_text {
                                <textarea
                                    class="ocr-text"
                                    value={ocr_text.clone()}
                                    oninput={props.on_text_edit.reform(|e: InputEvent| {
                                        e.target_unchecked_into::<web_sys::HtmlTextAreaElement>().value()
                                    })}
                                    data-testid="ocr-textarea"
                                />
                            } else {
                                <button
                                    onclick={props.on_run_ocr.reform(|_| ())}
                                    data-testid="ocr-button"
                                >
                                    { "Run OCR" }
                                </button>
                            }
                        </div>
                    </div>
                </div>
            }

            // Stage 4: Validation
            if props.data.raw_ocr_text.is_some() {
                <div class="pipeline-stage" data-testid="stage-validation">
                    <h2>{ "4. IBM 1130 Validation" }</h2>
                    <div class="stage-content">
                        if props.data.validation_errors.is_empty() {
                            <button
                                onclick={props.on_validate.reform(|_| ())}
                                data-testid="validate-button"
                            >
                                { "Validate" }
                            </button>
                        } else {
                            <div class="validation-errors" data-testid="validation-errors">
                                <h3>{ format!("Issues Found: {}", props.data.validation_errors.len()) }</h3>
                                <ul>
                                    { for props.data.validation_errors.iter().map(|error| {
                                        html! {
                                            <li class="validation-error" data-testid="validation-error">
                                                <strong>{ format!("Line {}: ", error.line_number) }</strong>
                                                { &error.description }
                                                if let Some(suggestion) = &error.suggestion {
                                                    <div class="suggestion">
                                                        { "Suggestion: " }{ suggestion }
                                                    </div>
                                                }
                                            </li>
                                        }
                                    })}
                                </ul>
                            </div>
                        }
                    </div>
                </div>
            }
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_data_default() {
        let data = PipelineData::default();
        assert!(data.original_image.is_none());
        assert!(data.cleaned_image.is_none());
        assert!(data.raw_ocr_text.is_none());
        assert!(data.corrected_text.is_none());
        assert!(data.validation_errors.is_empty());
    }

    #[test]
    fn test_validation_error_creation() {
        let error = ValidationError {
            line_number: 2,
            error_type: ErrorType::SequenceError,
            description: "Out of sequence".to_string(),
            suggestion: Some("Fix it".to_string()),
        };

        assert_eq!(error.line_number, 2);
        assert!(matches!(error.error_type, ErrorType::SequenceError));
        assert_eq!(error.description, "Out of sequence");
        assert_eq!(error.suggestion, Some("Fix it".to_string()));
    }

    #[test]
    fn test_pipeline_stage_equality() {
        assert_eq!(PipelineStage::Upload, PipelineStage::Upload);
        assert_ne!(PipelineStage::Upload, PipelineStage::ImageCleaning);
    }
}
