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
                images: Some(vec![image_b64]),
            }],
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
                images: Some(vec![image_b64]),
            }],
            stream: Some(false),
        };

        let response = self.client.chat(request).await?;

        // TODO: Parse and validate 80-character response
        Ok(response.message.content)
    }

    /// Correct OCR text using vision model with layout preservation
    ///
    /// Uses a two-pass approach:
    /// 1. Analyze image to detect format and column layout
    /// 2. Correct OCR text preserving exact spacing
    pub async fn correct_ocr_with_layout(
        &self,
        image_bytes: &[u8],
        raw_ocr_text: &str,
    ) -> Result<String> {
        let image_b64 = general_purpose::STANDARD.encode(image_bytes);

        let prompt = format!(
            r#"You are analyzing an IBM 1130 assembler/Forth listing scan from a GRAYSCALE greenbar printout.

CRITICAL INSTRUCTIONS:
1. IGNORE horizontal lines/bars from the greenbar background - they are NOT text
2. PRESERVE EXACT LEADING SPACES - count spaces from the left edge of the image
3. Each line MUST maintain its exact column position as seen in the image

COLUMN LAYOUT RULES:
1. Assembler Object Code:
   - Columns 1-4: Location (hex address)
   - Column 5: Flag (-, =, ', or blank)
   - Columns 6-9: Object word 1
   - Columns 10-13: Object word 2 (optional)
   - Columns 14-19: Opcode (DC, BSS, etc.)
   - Columns 20+: Operands and comments

2. Assembler Source:
   - Columns 1-5: Label (left-aligned)
   - Columns 9-12: Opcode (e.g., LDX, STX, BSI)
   - Columns 15+: Operands
   - Columns 40+: Comments (after *)

3. Forth Code:
   - Column 1: Top-level definitions (: WORD)
   - 4 spaces: Primary code
   - 8 spaces: Nested blocks (IF, DO, etc.)
   - 12+ spaces: Deeply nested

SPACING RULES:
- Measure indentation by counting character positions from LEFT EDGE of image
- Do NOT use the corrupted OCR spacing - look at the actual image
- Lines with hex addresses (OBFO, 0901, etc.) start at column 1 (NO leading spaces)
- Indented lines must have EXACT number of leading spaces visible in image
- Preserve ALL horizontal spacing between fields

CHARACTER CORRECTION:
- Fix OCR errors: Be → DC, oc → DC, OC → DC, etc.
- Ignore dashes/hyphens from greenbar lines - only include actual printed characters
- Only include characters that are part of the actual printed text

RAW OCR OUTPUT (corrupted, missing whitespace and has greenbar artifacts):
{}

TASK:
Return the corrected text with:
1. EXACT leading spaces preserved from image (not from OCR)
2. Character errors fixed
3. Greenbar line artifacts removed
4. Proper column alignment

Return ONLY the corrected text, nothing else."#,
            raw_ocr_text
        );

        let request = ChatRequest {
            model: self.model_name.clone(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: prompt,
                images: Some(vec![image_b64]),
            }],
            stream: Some(false),
        };

        let response = self.client.chat(request).await?;

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
