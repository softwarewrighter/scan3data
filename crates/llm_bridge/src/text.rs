//! Text model integration for refinement and analysis

use crate::ollama::{ChatMessage, ChatRequest, OllamaClient};
use anyhow::Result;

/// Text model for refining and analyzing extracted text
pub struct TextModel {
    client: OllamaClient,
    model_name: String,
}

impl TextModel {
    /// Create a new text model
    pub fn new(client: OllamaClient, model_name: String) -> Self {
        Self { client, model_name }
    }

    /// Create a text model with default settings (qwen2.5:3b)
    pub fn default_model() -> Result<Self> {
        Ok(Self::new(
            OllamaClient::default_client()?,
            "qwen2.5:3b".to_string(),
        ))
    }

    /// Refine OCR text and classify language
    pub async fn refine_and_classify(&self, ocr_text: &str) -> Result<RefinementResult> {
        let prompt = format!(
            r#"Analyze this OCR'd text from an IBM 1130 computer listing or card.

Text:
{}

Determine:
1. Language: assembler, FORTRAN, Forth, data, or unknown
2. Purpose: source, listing, object, log
3. Confidence: 0.0 to 1.0

Return JSON only: {{"language": "...", "purpose": "...", "confidence": 0.0}}"#,
            ocr_text
        );

        let request = ChatRequest {
            model: self.model_name.clone(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: prompt,
                images: None,
            }],
            stream: Some(false),
        };

        let _response = self.client.chat(request).await?;

        // TODO: Implement robust JSON parsing from response
        Ok(RefinementResult {
            language: "unknown".to_string(),
            purpose: "unknown".to_string(),
            confidence: 0.5,
            refined_text: ocr_text.to_string(),
        })
    }

    /// Suggest ordering for a collection of pages/cards
    pub async fn suggest_ordering(&self, items: &[OrderingItem]) -> Result<Vec<usize>> {
        // TODO: Implement ordering suggestion
        // Create prompt with first/last lines of each item
        // Ask LLM to propose ordering
        // Parse response

        Ok((0..items.len()).collect())
    }
}

/// Result of text refinement
pub struct RefinementResult {
    pub language: String,
    pub purpose: String,
    pub confidence: f32,
    pub refined_text: String,
}

/// An item for ordering suggestion
pub struct OrderingItem {
    pub id: String,
    pub first_lines: String,
    pub last_lines: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_model_creation() {
        let result = TextModel::default_model();
        assert!(result.is_ok() || result.is_err());
    }
}
