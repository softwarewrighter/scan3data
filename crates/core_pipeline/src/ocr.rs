//! OCR module
//!
//! Provides baseline OCR capabilities using Tesseract (via leptess).
//! This is the non-LLM approach for text extraction.

use anyhow::Result;
use image::GrayImage;

/// Extract text from an image using Tesseract OCR
///
/// Note: Requires Tesseract to be installed on the system.
/// In Phase 1, this is a placeholder that returns a TODO message.
pub fn extract_text_tesseract(_input: &GrayImage) -> Result<String> {
    // TODO: Integrate leptess crate
    // TODO: Configure for IBM 1130 character set
    // TODO: Handle fixed-width layout

    Ok("TODO: Implement Tesseract OCR integration".to_string())
}

/// Extract 80-column card text from a card image
pub fn extract_card_text(_input: &GrayImage) -> Result<String> {
    // TODO: Implement card-specific OCR
    // - Use column templates
    // - Extract exactly 80 characters
    // - Handle sequence columns (73-80)

    Ok(" ".repeat(80))
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Luma};

    #[test]
    fn test_extract_text_placeholder() {
        let img = ImageBuffer::from_pixel(100, 100, Luma([0u8]));
        let result = extract_text_tesseract(&img);
        assert!(result.is_ok());
    }

    #[test]
    fn test_extract_card_text_length() {
        let img = ImageBuffer::from_pixel(100, 100, Luma([0u8]));
        let result = extract_card_text(&img).unwrap();
        assert_eq!(result.len(), 80);
    }
}
