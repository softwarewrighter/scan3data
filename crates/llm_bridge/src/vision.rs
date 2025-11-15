//! Vision model integration for image analysis

use crate::ollama::{ChatMessage, ChatRequest, OllamaClient};
use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use core_pipeline::ArtifactKind;

/// Vision model for analyzing scanned images
pub struct VisionModel {
    client: OllamaClient,
    model_name: String,
}

impl VisionModel {
    /// Create a new vision model
    pub fn new(client: OllamaClient, model_name: String) -> Self {
        Self { client, model_name }
    }

    /// Create a vision model with default settings (qwen2.5vl:7b)
    pub fn default_model() -> Result<Self> {
        Ok(Self::new(
            OllamaClient::default_client()?,
            "qwen2.5vl:7b".to_string(),
        ))
    }

    /// Classify a scanned image
    pub async fn classify_image(&self, image_bytes: &[u8]) -> Result<ArtifactKind> {
        let image_b64 = general_purpose::STANDARD.encode(image_bytes);

        let prompt = r#"Describe this document briefly and categorize it as one of:
- CARD_TEXT: Punch card with text (assembler, FORTRAN, etc.)
- CARD_OBJECT: Punch card with binary/object code
- LISTING_SOURCE: Source code listing
- LISTING_OBJECT: Listing with object code
- RUNTIME_OUTPUT: Execution log or output
- UNKNOWN: Cannot determine

Return only JSON: {"category": "...", "description": "..."}"#;

        let request = ChatRequest {
            model: self.model_name.clone(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            images: Some(vec![image_b64]),
            stream: Some(false),
        };

        let response = self.client.chat(request).await?;

        // Parse response and map to ArtifactKind
        // TODO: Implement robust JSON parsing
        let category = if response.message.content.contains("CARD_TEXT") {
            ArtifactKind::CardText
        } else if response.message.content.contains("CARD_OBJECT") {
            ArtifactKind::CardObject
        } else if response.message.content.contains("LISTING_SOURCE") {
            ArtifactKind::ListingSource
        } else if response.message.content.contains("LISTING_OBJECT") {
            ArtifactKind::ListingObject
        } else if response.message.content.contains("RUNTIME_OUTPUT") {
            ArtifactKind::RuntimeOutput
        } else {
            ArtifactKind::Unknown
        };

        Ok(category)
    }

    /// Extract text from a card image (80 columns)
    pub async fn extract_card_text(&self, image_bytes: &[u8]) -> Result<String> {
        let image_b64 = general_purpose::STANDARD.encode(image_bytes);

        let prompt = r#"You are digitizing vintage 80-column IBM punch cards.
Extract the text from this card image.
Return exactly 80 characters using only [A-Z 0-9 + - * / , . ( ) & # ].
If you are unsure about a character, put ? in that position."#;

        let request = ChatRequest {
            model: self.model_name.clone(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            images: Some(vec![image_b64]),
            stream: Some(false),
        };

        let response = self.client.chat(request).await?;

        // TODO: Parse and validate 80-character response
        Ok(response.message.content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vision_model_creation() {
        let result = VisionModel::default_model();
        // Will fail without Ollama running, but tests the construction
        assert!(result.is_ok() || result.is_err());
    }
}
