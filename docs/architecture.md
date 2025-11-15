# Architecture Overview

## Why "scan3data"?

The **3** represents our **three-phase processing pipeline**:

1. **Scan** - Ingest and digitize (image acquisition, duplicate detection, preprocessing)
2. **Classify & Correct** - Analyze and refine (OCR, LLM classification, ordering, gap detection)
3. **Convert** - Transform to structured output (emulator formats, reconstruction, export)

This isn't just a simple scanner - it's a complete three-stage transformation pipeline from messy historical scans to pristine emulator-ready data.

## Multi-Crate Workspace Design

scan3data is organized as a Cargo workspace with 5 interconnected crates:

```
scan3data (workspace root)
+
+-- crates/
    |
    +-- core_pipeline/      [Library crate - no networking]
    |   +-- types.rs        Canonical Intermediate Representation (CIR)
    |   +-- preprocess.rs   Image preprocessing (classical CV)
    |   +-- ocr.rs          OCR integration (Tesseract)
    |   +-- decoder.rs      IBM 1130 object deck parser
    |
    +-- llm_bridge/         [Library crate - Ollama integration]
    |   +-- ollama.rs       HTTP client for Ollama API
    |   +-- vision.rs       Vision model wrapper (Qwen2.5-VL)
    |   +-- text.rs         Text model wrapper (Qwen2.5)
    |
    +-- cli/                [Binary crate - scan2data]
    |   +-- main.rs         Commands: ingest, analyze, export, serve
    |
    +-- server/             [Binary crate - scan2data-server]
    |   +-- main.rs         Axum REST API server
    |
    +-- yew_frontend/       [Library crate - WASM target]
        +-- lib.rs          Yew app entry point
        +-- app.rs          Main application component
        +-- components/     UI components
```

## Data Flow

### Phase 1: Non-LLM Baseline

```
Raw Scans (Images)
    |
    v
[Ingest] (CLI)
    |
    +-- Compute SHA-256 hash (duplicate detection)
    +-- Store image files
    +-- Create ScanSetId
    v
ScanSet (Directory with CIR JSON)
    |
    v
[Analyze] (CLI)
    |
    +-- Preprocess images (deskew, threshold)
    +-- Run Tesseract OCR
    +-- Classify artifacts (basic heuristics)
    +-- Extract 80-column text (cards)
    +-- Parse object decks (binary cards)
    v
Analyzed ScanSet (CIR with artifacts)
    |
    v
[Export] (CLI)
    |
    +-- Generate EmulatorOutput
    +-- Write JSON/text files
    v
Emulator-Ready Files
```

### Phase 2: LLM-Enhanced (Future)

```
Preprocessed Images
    |
    v
[Vision LLM] (llm_bridge)
    |
    +-- Classify: card vs listing, text vs binary
    +-- Extract metadata: page numbers, headers
    +-- OCR validation and refinement
    v
Classified Artifacts (higher confidence)
    |
    v
[Text LLM] (llm_bridge)
    |
    +-- Refine language detection (ASM, FORTRAN, Forth)
    +-- Suggest ordering (page/card sequences)
    +-- Detect gaps and anomalies
    +-- Propose reconstructions
    v
Refined CIR
    |
    v
Export to Emulator
```

## Canonical Intermediate Representation (CIR)

The CIR is the central data structure that flows through the pipeline:

```rust
ScanSet
    +-- ScanSetId (UUID)
    +-- Pages: Vec<PageArtifact>
    +-- Cards: Vec<CardArtifact>

PageArtifact
    +-- PageId (UUID)
    +-- raw_image_path: PathBuf
    +-- processed_image_path: Option<PathBuf>
    +-- layout_label: ArtifactKind
    +-- content_text: Option<String>
    +-- metadata: PageMetadata
        +-- page_number: Option<u32>
        +-- header/footer: Option<String>
        +-- original_filenames: Vec<String>  // Duplicate detection
        +-- notes: Vec<String>
        +-- confidence: f32

CardArtifact
    +-- CardId (UUID)
    +-- raw_image_path: PathBuf
    +-- processed_image_path: Option<PathBuf>
    +-- layout_label: ArtifactKind
    +-- text_80col: Option<String>          // For text decks
    +-- binary_80col: Option<Vec<u8>>       // For object decks
    +-- metadata: CardMetadata
        +-- sequence_number: Option<String>
        +-- deck_name: Option<String>
        +-- original_filenames: Vec<String>
        +-- notes: Vec<String>
        +-- confidence: f32

HighLevelArtifact (After reconstruction)
    +-- SourceListing
    +-- ObjectDeck
    +-- RunListing
    +-- Mixed (unresolved)
```

## Deployment Modes

### Mode 1: CLI-Only (Batch Processing)

Three-phase pipeline via CLI commands:

```bash
# Phase 1: Scan
scan3data ingest -i ./scans -o ./scan_set_001

# Phase 2: Classify & Correct
scan3data analyze -s ./scan_set_001

# Phase 3: Convert
scan3data export -s ./scan_set_001 -o output.json
```

Use case: Batch processing large collections, CI/CD pipelines

### Mode 2: Standalone SPA

```bash
./scripts/serve-spa.sh 8080
# Open http://localhost:8080
```

All processing happens in browser WASM:
- File upload via File API
- Image preprocessing in WASM (Photon)
- Limited OCR (or call backend selectively)
- Export files locally

Use case: Offline usage, no server setup required

### Mode 3: Backend + Frontend (Full Stack)

```bash
./scripts/serve-api.sh
# Backend: http://localhost:3000
# Frontend: http://localhost:3000/static
```

Heavy processing on server:
- Full Tesseract OCR
- Ollama LLM calls
- Job queue for long operations
- WebSocket progress updates

Use case: Production deployment, LLM integration, multiple users

## API Design (Mode 3)

REST endpoints:

```
POST   /api/scan_sets
    -> { id: UUID }

POST   /api/scan_sets/:id/upload
    Body: multipart/form-data with images
    -> { artifact_ids: [UUID] }

GET    /api/scan_sets/:id/artifacts
    -> { artifacts: [ArtifactInfo] }

POST   /api/scan_sets/:id/analyze
    Body: { use_llm: bool }
    -> { job_id: UUID }

GET    /api/jobs/:id
    -> { status: "pending|running|completed|failed", progress: 0.5 }

POST   /api/scan_sets/:id/export
    Body: { format: "card_deck|listing" }
    -> EmulatorOutput JSON
```

WebSocket for real-time updates:

```
WS     /ws/jobs/:id
    <- { event: "progress", value: 0.75 }
    <- { event: "complete", result: {...} }
```

## Extensibility Points

### 1. Add New Image Processing Step

```rust
// In core_pipeline/src/preprocess.rs
pub fn enhance_contrast(input: &GrayImage) -> Result<GrayImage> {
    // Implementation
}
```

### 2. Add New LLM Model

```rust
// In llm_bridge/src/vision.rs
impl VisionModel {
    pub fn with_model(model_name: String) -> Self {
        // Switch models
    }
}
```

### 3. Add New Artifact Type

```rust
// In core_pipeline/src/types.rs
pub enum ArtifactKind {
    // ... existing variants
    ForthSource,  // New type
}
```

### 4. Add New Export Format

```rust
// In core_pipeline/src/types.rs
pub enum EmulatorOutput {
    // ... existing variants
    BinaryDisk {
        sectors: Vec<Sector>,
    },
}
```

## Testing Strategy

### Unit Tests
- In each module's `#[cfg(test)]` section
- Test pure functions (no I/O)
- Use synthetic data

### Integration Tests
- `tests/` directory in each crate
- Test module interactions
- Use `tempfile` for filesystem ops

### End-to-End Tests
- In workspace root `tests/`
- Use `test-data/` samples
- Verify full pipeline

### WASM Tests
- Use `wasm-bindgen-test`
- Test in headless browser (CI)
- Verify WASM-specific code paths

## Dependencies

### Core Dependencies
- `image` + `imageproc` - Image processing
- `serde` + `serde_json` - Serialization
- `uuid` - Unique identifiers
- `anyhow` + `thiserror` - Error handling

### LLM Integration
- `reqwest` - HTTP client (Ollama API)
- `base64` - Image encoding

### Backend
- `axum` - HTTP server
- `tokio` - Async runtime
- `tower` + `tower-http` - Middleware

### Frontend
- `yew` - Reactive framework
- `wasm-bindgen` - JS interop
- `web-sys` - Web APIs
- `gloo` - WASM utilities

### CLI
- `clap` - Argument parsing
- `tracing` - Logging

## Performance Considerations

### Image Processing
- Use multi-threading (rayon) for batch operations
- Cache preprocessed images
- Support incremental processing

### LLM Calls
- Batch similar images (cards from same deck)
- Cache LLM responses (hash-based)
- Support local-only mode (no Ollama)

### WASM Size
- Use `wasm-opt` for optimization
- Code splitting for large apps
- Lazy-load components

## Security

### Input Validation
- Validate image formats
- Limit file sizes
- Sanitize filenames

### LLM Safety
- Never let LLM modify binary data directly
- Always validate LLM JSON outputs
- Flag low-confidence results

### API Security
- Rate limiting
- CORS configuration
- Input sanitization

## Future Enhancements

### Phase 3 Features
1. Document super-resolution (Python sidecar)
2. Custom IBM 1130 character recognition model
3. Forth-specific syntax analysis
4. Disassembly reconstruction from object code
5. Web-based collaborative editing
6. Export to multiple emulator formats
7. Automated test generation from listings
