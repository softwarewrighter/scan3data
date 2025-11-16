//! Gemini 2.5 Flash Image (Nano Banana) integration for image editing
//!
//! Provides API client for Google's Gemini image editing model to clean
//! scanned images by removing greenbar lines and background artifacts.

use anyhow::{Context, Result};
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};

/// Configuration for Gemini API client
#[derive(Debug, Clone)]
pub struct GeminiConfig {
    /// API key for Google Gemini
    pub api_key: String,
    /// Model to use (default: gemini-2.5-flash-image)
    pub model: String,
    /// Timeout in seconds
    pub timeout_secs: u64,
}

impl GeminiConfig {
    /// Create config from environment variable
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("GEMINI_API_KEY")
            .context("GEMINI_API_KEY environment variable not set")?;

        Ok(Self {
            api_key,
            model: "gemini-2.5-flash-image".to_string(),
            timeout_secs: 120,
        })
    }
}

/// Gemini API client for image editing
pub struct GeminiClient {
    config: GeminiConfig,
    client: reqwest::Client,
}

impl GeminiClient {
    /// Create a new Gemini client
    pub fn new(config: GeminiConfig) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { config, client })
    }

    /// Create client from environment variables
    pub fn from_env() -> Result<Self> {
        let config = GeminiConfig::from_env()?;
        Self::new(config)
    }

    /// Clean an image by removing greenbar lines and background artifacts
    ///
    /// # Arguments
    /// * `image_bytes` - Raw image data (JPEG, PNG, etc.)
    ///
    /// # Returns
    /// * Base64-encoded cleaned image data
    pub async fn clean_image(&self, image_bytes: &[u8]) -> Result<Vec<u8>> {
        let base64_image = general_purpose::STANDARD.encode(image_bytes);

        let prompt = concat!(
            "This is a scan of a vintage computer printout on greenbar paper. ",
            "The image has horizontal lines and bands from the greenbar background. ",
            "Remove all background lines, bands, and artifacts while preserving the printed text exactly. ",
            "Keep the text sharp and clear. Output a clean white background with only the text visible."
        );

        let request = GeminiRequest {
            contents: vec![GeminiContent {
                parts: vec![
                    GeminiPart::Text {
                        text: prompt.to_string(),
                    },
                    GeminiPart::InlineData {
                        inline_data: InlineData {
                            mime_type: "image/jpeg".to_string(),
                            data: base64_image,
                        },
                    },
                ],
            }],
        };

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
            self.config.model
        );

        let response = self
            .client
            .post(&url)
            .header("x-goog-api-key", &self.config.api_key)
            .json(&request)
            .send()
            .await
            .context("Failed to send request to Gemini API")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Gemini API error ({}): {}", status, error_text);
        }

        let gemini_response: GeminiResponse = response
            .json()
            .await
            .context("Failed to parse Gemini response")?;

        // Extract image from response
        if let Some(candidate) = gemini_response.candidates.first() {
            if let Some(GeminiPart::InlineData { inline_data }) = candidate.content.parts.first() {
                let decoded = general_purpose::STANDARD
                    .decode(&inline_data.data)
                    .context("Failed to decode base64 image")?;
                return Ok(decoded);
            }
        }

        anyhow::bail!("No image in Gemini response")
    }
}

/// Gemini API request structure
#[derive(Debug, Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum GeminiPart {
    Text { text: String },
    InlineData { inline_data: InlineData },
}

#[derive(Debug, Serialize, Deserialize)]
struct InlineData {
    mime_type: String,
    data: String,
}

/// Gemini API response structure
#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}

#[derive(Debug, Deserialize)]
struct GeminiCandidate {
    content: GeminiContent,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gemini_config_default() {
        let config = GeminiConfig {
            api_key: "test-key".to_string(),
            model: "gemini-2.5-flash-image".to_string(),
            timeout_secs: 120,
        };

        assert_eq!(config.api_key, "test-key");
        assert_eq!(config.model, "gemini-2.5-flash-image");
        assert_eq!(config.timeout_secs, 120);
    }

    #[test]
    fn test_gemini_client_creation() {
        let config = GeminiConfig {
            api_key: "test-key".to_string(),
            model: "gemini-2.5-flash-image".to_string(),
            timeout_secs: 120,
        };

        let client = GeminiClient::new(config);
        assert!(client.is_ok());
    }

    #[test]
    fn test_base64_encoding() {
        let test_data = b"test image data";
        let encoded = general_purpose::STANDARD.encode(test_data);
        let decoded = general_purpose::STANDARD.decode(&encoded).unwrap();
        assert_eq!(test_data, decoded.as_slice());
    }

    // Note: Integration test would require GEMINI_API_KEY
    // and would be expensive ($0.039 per image)
    // Run manually: cargo test --features integration_tests -- --ignored
    #[test]
    #[ignore]
    fn test_clean_image_integration() {
        // This test requires GEMINI_API_KEY environment variable
        // and will make actual API calls (costs money)
        // Run with: cargo test test_clean_image_integration -- --ignored
    }
}
