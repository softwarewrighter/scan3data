//! OCR module
//!
//! Provides baseline OCR capabilities using Tesseract (via leptess).
//! This is the non-LLM approach for text extraction.

use anyhow::{Context, Result};
use image::GrayImage;
use leptess::LepTess;

/// Extract text from an image using Tesseract OCR
///
/// Uses Tesseract without character whitelist (permissive mode).
/// Post-processing should validate characters based on expected IBM 1130 character sets.
///
/// # Arguments
/// * `input` - Grayscale image to extract text from
///
/// # Returns
/// * Extracted text as a string, preserving layout
///
/// # Errors
/// * Returns error if Tesseract is not installed or OCR fails
pub fn extract_text_tesseract(input: &GrayImage) -> Result<String> {
    // Initialize Tesseract
    let mut tesseract = LepTess::new(None, "eng")
        .context("Failed to initialize Tesseract. Is Tesseract installed?")?;

    // Convert GrayImage to PNG bytes for leptess
    // leptess requires image data in a standard format (PNG, JPEG, etc.)
    let mut png_bytes = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut png_bytes);
    input
        .write_to(&mut cursor, image::ImageFormat::Png)
        .context("Failed to encode image as PNG")?;

    // Set image in Tesseract
    tesseract
        .set_image_from_mem(&png_bytes)
        .context("Failed to load image into Tesseract")?;

    // Extract text
    let text = tesseract
        .get_utf8_text()
        .context("Failed to extract text from image")?;

    Ok(text)
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
    fn test_extract_text_returns_string() {
        // Simple test: black image should return empty or whitespace
        let img = ImageBuffer::from_pixel(100, 100, Luma([0u8]));
        let result = extract_text_tesseract(&img);
        assert!(result.is_ok());
        // Result should be a string (even if empty)
        let text = result.unwrap();
        assert!(text.is_empty() || text.trim().is_empty());
    }

    #[test]
    fn test_extract_text_white_image() {
        // White image (no text) should return empty string
        let img = ImageBuffer::from_pixel(100, 100, Luma([255u8]));
        let result = extract_text_tesseract(&img);
        assert!(result.is_ok());
        let text = result.unwrap();
        assert!(text.is_empty() || text.trim().is_empty());
    }

    #[test]
    fn test_extract_text_handles_tesseract_not_installed() {
        // If Tesseract is not installed, should return meaningful error
        // This test documents expected behavior, implementation will determine actual behavior
        let img = ImageBuffer::from_pixel(100, 100, Luma([0u8]));
        let result = extract_text_tesseract(&img);
        // For now, we expect it to work if Tesseract is installed
        // or fail gracefully if not
        match result {
            Ok(_) => {} // Tesseract is installed
            Err(e) => {
                // Error message should mention Tesseract
                let msg = e.to_string().to_lowercase();
                assert!(msg.contains("tesseract") || msg.contains("leptess"));
            }
        }
    }

    #[test]
    fn test_extract_card_text_length() {
        let img = ImageBuffer::from_pixel(100, 100, Luma([0u8]));
        let result = extract_card_text(&img).unwrap();
        assert_eq!(result.len(), 80);
    }
}
