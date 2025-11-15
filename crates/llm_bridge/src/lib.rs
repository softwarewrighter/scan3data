//! LLM Bridge for Ollama integration
//!
//! Provides integration with local LLMs via Ollama HTTP API.
//! Supports both vision models (for image analysis) and text models
//! (for classification and refinement).

pub mod ollama;
pub mod text;
pub mod vision;

pub use ollama::{OllamaClient, OllamaConfig};
pub use text::TextModel;
pub use vision::VisionModel;
