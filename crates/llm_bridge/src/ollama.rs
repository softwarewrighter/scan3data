//! Ollama HTTP API client

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Configuration for Ollama client
#[derive(Debug, Clone)]
pub struct OllamaConfig {
    /// Base URL for Ollama API (default: http://localhost:11434)
    pub base_url: String,
    /// Timeout in seconds (default: 120)
    pub timeout_secs: u64,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:11434".to_string(),
            timeout_secs: 120,
        }
    }
}

/// Ollama API client
pub struct OllamaClient {
    config: OllamaConfig,
    client: reqwest::Client,
}

impl OllamaClient {
    /// Create a new Ollama client
    pub fn new(config: OllamaConfig) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .build()?;

        Ok(Self { config, client })
    }

    /// Create a client with default configuration
    pub fn default_client() -> Result<Self> {
        Self::new(OllamaConfig::default())
    }

    /// Send a chat request to Ollama
    pub async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let url = format!("{}/api/chat", self.config.base_url);

        let response = self.client.post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            anyhow::bail!("Ollama API error: {}", response.status());
        }

        let chat_response: ChatResponse = response.json().await?;
        Ok(chat_response)
    }
}

/// Chat request to Ollama
#[derive(Debug, Clone, Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

/// A chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// Chat response from Ollama
#[derive(Debug, Clone, Deserialize)]
pub struct ChatResponse {
    pub model: String,
    pub message: ChatMessage,
    pub done: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ollama_config_default() {
        let config = OllamaConfig::default();
        assert_eq!(config.base_url, "http://localhost:11434");
        assert_eq!(config.timeout_secs, 120);
    }

    #[test]
    fn test_chat_request_serialization() {
        let request = ChatRequest {
            model: "qwen2.5vl:7b".to_string(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
            }],
            images: None,
            stream: Some(false),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("qwen2.5vl:7b"));
    }
}
