# LLM Bridge

The **llm_bridge** crate handles all integration with external AI services: Gemini API (cloud) and Ollama (local). It provides clean abstraction layers for vision and text models.

## Overview

**Crate Name:** `llm_bridge`
**Type:** Library
**Dependencies:** reqwest, base64, serde_json, anyhow
**External Services:** Gemini 2.5 Flash Image, Ollama (llama3.2-vision, Qwen2.5-VL)

### Responsibilities

1. Gemini API client for image cleaning
2. Ollama API client for vision and text models
3. Prompt template management for IBM 1130-specific tasks
4. Response parsing and validation
5. Error handling and retry logic

## Module Structure

```
llm_bridge/src/
├── lib.rs          # Public API exports
├── gemini.rs       # Gemini API client
├── ollama.rs       # Ollama API client
├── vision.rs       # Vision model abstraction
├── text.rs         # Text model abstraction
└── prompts.rs      # Prompt templates
```

## Gemini API Integration

### Image Cleaning

```rust
pub struct GeminiClient {
    api_key: String,
    base_url: String,
}

impl GeminiClient {
    pub async fn clean_image(&self, image_bytes: &[u8]) -> Result<Vec<u8>> {
        let base64_image = base64::encode(image_bytes);

        let request = json!({
            "contents": [{
                "parts": [
                    {"text": GREENBAR_REMOVAL_PROMPT},
                    {
                        "inline_data": {
                            "mime_type": "image/jpeg",
                            "data": base64_image
                        }
                    }
                ]
            }]
        });

        let response = self.client
            .post(&format!("{}/v1/models/gemini-2.5-flash-image:generateContent", self.base_url))
            .header("x-goog-api-key", &self.api_key)
            .json(&request)
            .send()
            .await?;

        // Parse and decode cleaned image
        let cleaned_base64 = extract_image_from_response(&response).await?;
        let cleaned_bytes = base64::decode(&cleaned_base64)?;

        Ok(cleaned_bytes)
    }
}
```

**Prompt:** "Remove all greenbar artifacts and background patterns from this scanned IBM 1130 computer listing while preserving all text. Output only the cleaned image with white background and black text."

**Cost:** $0.039 per image
**Performance:** ~2-4 seconds per image

## Ollama API Integration

### Vision Model Classification

```rust
pub struct OllamaClient {
    base_url: String,
}

impl OllamaClient {
    pub async fn classify_image(&self, image_bytes: &[u8], model: &str) -> Result<ArtifactKind> {
        let base64_image = base64::encode(image_bytes);

        let request = json!({
            "model": model,
            "prompt": IBM1130_CLASSIFICATION_PROMPT,
            "images": [base64_image],
            "stream": false
        });

        let response = self.client
            .post(&format!("{}/api/generate", self.base_url))
            .json(&request)
            .send()
            .await?;

        // Parse classification from response
        let classification_text = response.json::<OllamaResponse>().await?.response;
        parse_artifact_kind(&classification_text)
    }
}
```

**Models:**
- `llama3.2-vision` - Fast, good for basic classification
- `qwen2.5-vl:7b` - Better accuracy, IBM 1130-specific tuning

### Text Model Refinement

```rust
pub async fn refine_ocr_text(&self, ocr_text: &str, context: &str) -> Result<String> {
    let prompt = format!(
        "{}\n\nContext: {}\n\nOCR Text:\n{}",
        OCR_CORRECTION_PROMPT, context, ocr_text
    );

    let request = json!({
        "model": "qwen2.5:3b",
        "prompt": prompt,
        "stream": false
    });

    let response = self.client
        .post(&format!("{}/api/generate", self.base_url))
        .json(&request)
        .send()
        .await?;

    Ok(response.json::<OllamaResponse>().await?.response)
}
```

## Prompt Templates

### Greenbar Removal Prompt

```text
You are analyzing a scanned IBM 1130 computer listing on greenbar paper.

Task: Remove all greenbar artifacts (alternating green/white bands) and background noise while perfectly preserving all text.

Requirements:
- Output a cleaned image with pure white background
- Preserve all text in high-contrast black
- Maintain exact text positioning and alignment
- Remove paper texture, creases, and stains
- Preserve column alignment (critical for punch card data)

Output only the cleaned image, no explanations.
```

### Classification Prompt

```text
You are analyzing a scanned image from an IBM 1130 computer system (circa 1960s-1970s).

Classify this image as ONE of:
- TEXT_CARD: 80-column punch card with text/source code
- OBJECT_CARD: 80-column punch card with binary object code
- SOURCE_LISTING: Computer printout of source code
- RUN_LISTING: Computer printout of program output
- MIXED: Multiple types or unclear
- UNKNOWN: Cannot determine

Also extract:
- Page number (if visible in header/footer)
- Deck name or program name (if visible)
- Any sequence numbers

Respond in JSON format:
{
  "classification": "TEXT_CARD",
  "page_number": 1,
  "deck_name": "FORTH",
  "confidence": 0.95
}
```

### OCR Correction Prompt

```text
You are correcting OCR text from an IBM 1130 computer listing.

Common OCR errors to fix:
- O (letter) vs 0 (zero)
- I (letter) vs 1 (one)
- S vs 5
- Z vs 2
- B vs 8

IBM 1130 context:
- Uppercase only (no lowercase)
- FORTRAN or assembly language syntax
- Column-aligned code (preserve spacing!)
- Columns 73-80 may contain sequence numbers

Correct obvious OCR errors while preserving:
- Exact column positions
- Line breaks
- Sequence numbers

Output ONLY the corrected text, no explanations.
```

## Error Handling

### Retry Logic with Exponential Backoff

```rust
async fn call_with_retry<F, T>(f: F, max_retries: u32) -> Result<T>
where
    F: Fn() -> Future<Output = Result<T>>,
{
    let mut delay = Duration::from_secs(2);

    for attempt in 0..max_retries {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) if is_retryable(&e) => {
                if attempt < max_retries - 1 {
                    tokio::time::sleep(delay).await;
                    delay *= 2; // Exponential backoff
                } else {
                    return Err(e);
                }
            }
            Err(e) => return Err(e),
        }
    }

    unreachable!()
}

fn is_retryable(error: &Error) -> bool {
    matches!(
        error,
        Error::Network(_) | Error::RateLimit(_) | Error::Timeout(_)
    )
}
```

**Retry Strategy:**
- Initial delay: 2s
- Backoff multiplier: 2x (2s, 4s, 8s, 16s)
- Max retries: 4
- Only retry: Network errors, rate limits, timeouts
- Never retry: Authentication errors, invalid requests

## API Reference

### Gemini Client

```rust
// Create client
pub fn GeminiClient::new(api_key: String) -> Self;

// Clean greenbar artifacts
pub async fn clean_image(&self, image_bytes: &[u8]) -> Result<Vec<u8>>;
```

### Ollama Client

```rust
// Create client
pub fn OllamaClient::new(base_url: String) -> Self;
pub fn OllamaClient::default() -> Self; // Uses http://localhost:11434

// Vision model operations
pub async fn classify_image(&self, image_bytes: &[u8], model: &str) -> Result<ArtifactKind>;
pub async fn extract_metadata(&self, image_bytes: &[u8]) -> Result<PageMetadata>;

// Text model operations
pub async fn refine_ocr_text(&self, ocr_text: &str, context: &str) -> Result<String>;
pub async fn detect_language(&self, code: &str) -> Result<Language>;
```

## Usage Examples

### Clean Image with Gemini

```rust
use llm_bridge::GeminiClient;

#[tokio::main]
async fn main() -> Result<()> {
    let client = GeminiClient::new(std::env::var("GEMINI_API_KEY")?);

    let image_bytes = std::fs::read("greenbar-listing.jpg")?;
    let cleaned_bytes = client.clean_image(&image_bytes).await?;

    std::fs::write("cleaned-listing.jpg", cleaned_bytes)?;
    Ok(())
}
```

### Classify with Ollama Vision

```rust
use llm_bridge::OllamaClient;

#[tokio::main]
async fn main() -> Result<()> {
    let client = OllamaClient::default();

    let image_bytes = std::fs::read("card.jpg")?;
    let classification = client.classify_image(&image_bytes, "llama3.2-vision").await?;

    println!("Classification: {:?}", classification);
    Ok(())
}
```

## Related Pages

- [Core Pipeline](Core-Pipeline) - Image preprocessing before LLM calls
- [REST API](REST-API) - API endpoints that use llm_bridge
- [Data Flow](Data-Flow) - Sequence diagrams showing LLM integration

---

**Last Updated:** 2025-11-16
