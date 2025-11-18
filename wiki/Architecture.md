# Architecture Overview

scan3data is built as a multi-crate Rust workspace with strong separation of concerns and pluggable deployment modes.

## Table of Contents

- [Multi-Crate Workspace](#multi-crate-workspace)
- [Component Architecture](#component-architecture)
- [Canonical Intermediate Representation (CIR)](#canonical-intermediate-representation-cir)
- [Deployment Modes](#deployment-modes)
- [Technology Stack](#technology-stack)
- [Design Principles](#design-principles)

## Multi-Crate Workspace

scan3data is organized as 5 interconnected Rust crates within a Cargo workspace:

```mermaid
graph TB
    subgraph "Workspace: scan3data"
        subgraph "Library Crates"
            CORE[core_pipeline<br/>Image processing, OCR, CIR types<br/>No networking]
            LLM[llm_bridge<br/>Gemini & Ollama API clients<br/>HTTP-based]
            YEW[yew_frontend<br/>Yew/WASM UI components<br/>Browser target]
        end

        subgraph "Binary Crates"
            CLI[cli<br/>scan3data binary<br/>Commands: ingest, analyze, export, serve]
            SRV[server<br/>scan3data-server binary<br/>Axum REST API]
        end
    end

    CLI --> CORE
    CLI --> LLM
    SRV --> CORE
    SRV --> LLM
    YEW --> SRV
    YEW -.->|"(SPA mode)"| CORE
```

### Crate Responsibilities

| Crate | Type | Responsibility | Key Dependencies |
|-------|------|---------------|------------------|
| **core_pipeline** | Library | Core types (CIR), image preprocessing, OCR integration, IBM 1130 decoder | image, imageproc, leptess, serde |
| **llm_bridge** | Library | Gemini API client, Ollama API client, prompt templates | reqwest, base64, serde_json |
| **cli** | Binary | Command-line interface with ingest/analyze/export/serve commands | clap, core_pipeline, llm_bridge |
| **server** | Binary | Axum REST API server, static file serving | axum, tokio, tower-http, core_pipeline |
| **yew_frontend** | Library | Yew/WASM UI with 4-stage pipeline visualization | yew, wasm-bindgen, gloo, web-sys |

## Component Architecture

### High-Level Component Diagram

```mermaid
graph TB
    subgraph "Presentation Layer"
        UI[Yew Web UI<br/>WASM]
        CLICMD[CLI Commands]
    end

    subgraph "API Layer"
        REST[REST API<br/>Axum/Tokio]
        SERVE[Static File Server]
    end

    subgraph "Business Logic Layer"
        INGEST[Ingest Pipeline<br/>Duplicate Detection]
        ANALYZE[Analyze Pipeline<br/>OCR, Classification]
        EXPORT[Export Pipeline<br/>Emulator Formats]
    end

    subgraph "Core Processing"
        PREPROC[Image Preprocessing<br/>Deskew, Threshold]
        OCR[OCR Engine<br/>Tesseract Integration]
        DECODER[IBM 1130 Decoder<br/>Object Deck Parser]
        CIR[CIR Types<br/>ScanSet, Artifacts]
    end

    subgraph "External Integration"
        GEMINI[Gemini API<br/>Image Cleaning]
        OLLAMA[Ollama API<br/>Vision Models]
        TESS[Tesseract<br/>OCR Binary]
    end

    subgraph "Storage"
        FS[File System<br/>Images, JSON, CIR]
    end

    UI --> REST
    UI --> SERVE
    CLICMD --> INGEST
    CLICMD --> ANALYZE
    CLICMD --> EXPORT
    REST --> INGEST
    REST --> ANALYZE
    REST --> EXPORT

    INGEST --> PREPROC
    INGEST --> CIR
    ANALYZE --> OCR
    ANALYZE --> DECODER
    ANALYZE --> GEMINI
    ANALYZE --> OLLAMA
    EXPORT --> CIR

    PREPROC --> CIR
    OCR --> TESS
    OCR --> CIR
    DECODER --> CIR

    INGEST --> FS
    ANALYZE --> FS
    EXPORT --> FS
    CIR --> FS
```

### Component Interaction Details

```mermaid
sequenceDiagram
    participant User
    participant UI as Yew UI
    participant API as REST API
    participant Core as Core Pipeline
    participant LLM as LLM Bridge
    participant FS as File System

    User->>UI: Upload Image
    UI->>API: POST /api/clean-image
    API->>LLM: clean_image()
    LLM->>LLM: Call Gemini API
    LLM-->>API: Cleaned Image
    API-->>UI: Base64 Image Data
    UI->>UI: Display Original vs Cleaned
    User->>UI: Run OCR
    UI->>API: POST /api/ocr-extract
    API->>Core: run_tesseract_ocr()
    Core->>Core: Preprocess Image
    Core->>Core: Extract Text
    Core-->>API: OCR Text
    API-->>UI: OCR Results
    UI->>UI: Show Editable Text
    User->>UI: Validate & Export
    UI->>API: POST /api/export
    API->>Core: generate_emulator_output()
    Core->>FS: Write JSON/Text
    Core-->>API: Export Path
    API-->>UI: Download Link
```

## Canonical Intermediate Representation (CIR)

The CIR is the central data structure that flows through the pipeline. It provides a unified representation for all processing stages.

### CIR Type Hierarchy

```mermaid
classDiagram
    class ScanSet {
        +ScanSetId: UUID
        +created_at: DateTime
        +pages: Vec~PageArtifact~
        +cards: Vec~CardArtifact~
        +metadata: ScanSetMetadata
    }

    class PageArtifact {
        +page_id: UUID
        +raw_image_path: PathBuf
        +processed_image_path: Option~PathBuf~
        +layout_label: ArtifactKind
        +content_text: Option~String~
        +metadata: PageMetadata
    }

    class CardArtifact {
        +card_id: UUID
        +raw_image_path: PathBuf
        +processed_image_path: Option~PathBuf~
        +layout_label: ArtifactKind
        +text_80col: Option~String~
        +binary_80col: Option~Vec~u8~~
        +metadata: CardMetadata
    }

    class PageMetadata {
        +page_number: Option~u32~
        +header: Option~String~
        +footer: Option~String~
        +original_filenames: Vec~String~
        +notes: Vec~String~
        +confidence: f32
    }

    class CardMetadata {
        +sequence_number: Option~String~
        +deck_name: Option~String~
        +original_filenames: Vec~String~
        +notes: Vec~String~
        +confidence: f32
    }

    class ArtifactKind {
        <<enumeration>>
        TextCard
        ObjectCard
        SourceListing
        RunListing
        Mixed
        Unknown
    }

    class EmulatorOutput {
        <<enumeration>>
        CardDeck
        Listing
        ObjectCode
    }

    ScanSet "1" --> "*" PageArtifact
    ScanSet "1" --> "*" CardArtifact
    PageArtifact "1" --> "1" PageMetadata
    CardArtifact "1" --> "1" CardMetadata
    PageArtifact --> ArtifactKind
    CardArtifact --> ArtifactKind
```

### CIR Data Flow

```mermaid
stateDiagram-v2
    [*] --> RawImages: Scan/Upload
    RawImages --> ScanSet: Ingest (Duplicate Detection)
    ScanSet --> Preprocessed: Image Preprocessing
    Preprocessed --> OCRExtracted: Tesseract OCR
    OCRExtracted --> LLMCleaned: Gemini Image Cleaning
    LLMCleaned --> Classified: Vision Model Classification
    Classified --> Validated: IBM 1130 Validation
    Validated --> Ordered: Page/Card Ordering
    Ordered --> EmulatorOutput: Export
    EmulatorOutput --> [*]

    note right of ScanSet
        CIR stored as JSON
        SHA-256 deduplication
    end note

    note right of Classified
        ArtifactKind assigned
        Confidence scored
    end note

    note right of EmulatorOutput
        CardDeck / Listing
        JSON or Text format
    end note
```

## Deployment Modes

scan3data supports three deployment modes to suit different use cases:

### Mode 1: CLI-Only (Batch Processing)

```mermaid
graph LR
    INPUT[Raw Scans] --> INGEST[scan3data ingest]
    INGEST --> SCANSET[ScanSet Directory]
    SCANSET --> ANALYZE[scan3data analyze]
    ANALYZE --> ANALYZED[Analyzed ScanSet]
    ANALYZED --> EXPORT[scan3data export]
    EXPORT --> OUTPUT[Emulator Files]
```

**Use Cases:**
- Batch processing large collections
- CI/CD pipelines
- Server-side automation
- No GUI needed

**Commands:**
```bash
scan3data ingest -i ./scans -o ./scan_set_001
scan3data analyze -s ./scan_set_001
scan3data export -s ./scan_set_001 -o output.json
```

### Mode 2: Standalone SPA

```mermaid
graph TB
    USER[User Browser] --> UI[Yew/WASM UI]
    UI --> WASM[WASM Processing]
    WASM --> FILE[File API Upload]
    WASM --> PROC[Image Processing]
    WASM --> DL[Download Results]

    style WASM fill:#e1f5fe
    style UI fill:#b3e5fc
```

**Use Cases:**
- Offline usage
- No server setup required
- Privacy-sensitive (all processing local)
- Demos and prototyping

**Serving:**
```bash
./scripts/serve-spa.sh 8080
# Open http://localhost:8080
```

**Note:** Currently limited - full implementation planned for Phase 3.

### Mode 3: Backend + Frontend (Full Stack)

```mermaid
graph TB
    USER[User Browser] --> UI[Yew UI<br/>Port 7214]
    UI <--> API[REST API<br/>Axum Server]
    API --> CORE[Core Pipeline]
    API --> LLM[LLM Bridge]
    API --> QUEUE[Job Queue]
    API --> FS[File System]

    LLM --> GEMINI[Gemini API]
    LLM --> OLLAMA[Ollama API]
    CORE --> TESS[Tesseract]

    style API fill:#fff9c4
    style UI fill:#b3e5fc
    style CORE fill:#c8e6c9
    style LLM fill:#f8bbd0
```

**Use Cases:**
- Production deployment
- LLM integration (Gemini, Ollama)
- Multiple concurrent users
- Heavy processing workloads
- WebSocket progress updates

**Serving:**
```bash
./target/release/scan3data-server
# Backend + UI: http://localhost:7214
```

**Current Implementation:**
- ✅ Static file serving (Yew UI)
- ✅ Gemini API integration (/api/clean-image)
- 🚧 Job queue (planned)
- 🚧 WebSocket progress (planned)
- 🚧 Scan set storage (placeholder)

## Technology Stack

### Frontend Stack

```mermaid
graph TB
    subgraph "Frontend (Rust -> WASM)"
        YEW[Yew 0.21<br/>Reactive Framework]
        WASM[wasm-bindgen<br/>JS Interop]
        WEB[web-sys<br/>Web APIs]
        GLOO[gloo<br/>WASM Utilities]
        NET[gloo-net<br/>HTTP Client]
    end

    YEW --> WASM
    YEW --> WEB
    YEW --> GLOO
    YEW --> NET
```

**Build Tool:** Trunk (WASM bundler)
**Target:** wasm32-unknown-unknown

### Backend Stack

```mermaid
graph TB
    subgraph "Backend (Rust)"
        AXUM[Axum 0.7<br/>HTTP Framework]
        TOKIO[Tokio 1.0<br/>Async Runtime]
        TOWER[Tower<br/>Middleware]
        SERDE[Serde<br/>Serialization]
    end

    AXUM --> TOKIO
    AXUM --> TOWER
    AXUM --> SERDE
```

**Runtime:** Tokio async
**Server:** Axum with tower-http

### Core Processing Stack

```mermaid
graph TB
    subgraph "Image Processing"
        IMG[image 0.25<br/>Image I/O]
        PROC[imageproc 0.25<br/>Processing Ops]
        LEPT[leptess 0.14<br/>Tesseract Binding]
    end

    subgraph "LLM Integration"
        REQ[reqwest 0.12<br/>HTTP Client]
        B64[base64 0.22<br/>Encoding]
    end

    subgraph "Core Types"
        SER[serde + serde_json<br/>Serialization]
        UUID[uuid 1.0<br/>Identifiers]
        ERR[anyhow + thiserror<br/>Error Handling]
    end
```

**OCR Engine:** Tesseract (external binary)
**LLM APIs:** Gemini (cloud), Ollama (local)

## Design Principles

### 1. Rust-First Philosophy

**All** business logic, data processing, and presentation code is written in Rust:
- Frontend: Yew (compiles to WASM)
- Backend: Axum (pure Rust HTTP)
- Image Processing: Rust crates
- CLI: Rust (clap)

**Only acceptable non-Rust:**
- Minimal HTML/CSS for UI structure
- External binaries called via `Command::new()` (Tesseract, Ollama)

**Never acceptable:**
- JavaScript/TypeScript for business logic
- Python for image processing
- Node.js for backend

### 2. Separation of Concerns

Each crate has a single, well-defined responsibility:

| Concern | Crate | Dependencies |
|---------|-------|-------------|
| Data types & core processing | core_pipeline | No networking, pure logic |
| External API integration | llm_bridge | HTTP clients only |
| Command-line interface | cli | Uses core_pipeline + llm_bridge |
| Web API | server | Uses core_pipeline + llm_bridge |
| Web UI | yew_frontend | Browser-only, no backend coupling |

### 3. Pluggable Deployment

The architecture supports multiple deployment modes without code changes:
- CLI-only: No web server needed
- SPA: No backend needed (future)
- Full-stack: Complete feature set

This is achieved through:
- Core logic in libraries (not binaries)
- Clean API boundaries between crates
- Configuration-based mode selection

### 4. Test-Driven Development

Every component is designed for testability:
- Pure functions in core_pipeline (no I/O)
- Mock APIs in llm_bridge tests
- Integration tests for CLI commands
- WASM tests for frontend components

**Quality Standards:**
- Zero clippy warnings (`-D warnings`)
- All tests pass before commit
- Red/Green/Refactor TDD cycle

### 5. Canonical Intermediate Representation

All pipeline stages operate on the same CIR:
- Consistent data model across phases
- Easy to serialize/deserialize (JSON)
- Extensible via enums (ArtifactKind, EmulatorOutput)
- Backward-compatible changes

This enables:
- Pipeline checkpointing (save/resume)
- Manual intervention at any stage
- Incremental processing
- Easy debugging

### 6. Progressive Enhancement

The system works in stages of increasing sophistication:

**Phase 1: Baseline (No LLM)**
- Classical image preprocessing
- Tesseract OCR
- Basic heuristics for classification
- ✅ **Status: Complete**

**Phase 2: LLM-Enhanced**
- Gemini image cleaning
- Vision model OCR correction
- Text model classification and ordering
- 🚧 **Status: In Progress**

**Phase 3: Advanced AI**
- Custom IBM 1130 models
- Reverse engineering
- Automated reconstruction
- 📋 **Status: Planned**

Each phase builds on the previous, ensuring a working baseline at all times.

## Extensibility Points

The architecture is designed for easy extension:

### 1. Adding New Image Processing

```rust
// In core_pipeline/src/preprocess.rs
pub fn super_resolution(input: &GrayImage) -> Result<GrayImage> {
    // New preprocessing step
}
```

### 2. Adding New LLM Provider

```rust
// In llm_bridge/src/anthropic.rs
pub struct AnthropicClient {
    api_key: String,
}

impl AnthropicClient {
    pub async fn classify_image(&self, image: &[u8]) -> Result<ArtifactKind> {
        // Call Claude API
    }
}
```

### 3. Adding New Artifact Type

```rust
// In core_pipeline/src/types.rs
pub enum ArtifactKind {
    // Existing variants...
    ForthSource,      // New variant
    FortranListing,   // Another new variant
}
```

### 4. Adding New Export Format

```rust
// In core_pipeline/src/types.rs
pub enum EmulatorOutput {
    // Existing variants...
    BinaryDiskImage {
        tracks: Vec<Track>,
    },
}
```

## Performance Considerations

### Image Processing
- **Multi-threading:** Use rayon for parallel batch operations
- **Caching:** Store preprocessed images to avoid recomputation
- **Incremental:** Process only changed/new images

### LLM Calls
- **Batching:** Group similar images (same deck) for efficiency
- **Caching:** Hash-based cache for LLM responses
- **Fallback:** Local-only mode (Tesseract) when LLMs unavailable

### WASM Size
- **Optimization:** Use wasm-opt for size reduction
- **Code Splitting:** Lazy-load components (future)
- **Selective Features:** Only include needed functionality

### API Performance
- **Async:** Tokio for non-blocking I/O
- **Streaming:** Stream large responses
- **Job Queue:** Offload heavy work to background workers (planned)

## Security Considerations

### Input Validation
- ✅ Image format validation (PNG, JPEG, TIFF)
- 🚧 File size limits (planned)
- 🚧 Filename sanitization (planned)

### LLM Safety
- ✅ Never allow LLM to modify binary data directly
- ✅ Always validate LLM JSON outputs
- ✅ Flag low-confidence results for review

### API Security
- 🚧 Rate limiting (planned)
- ✅ CORS configuration (permissive for dev, needs prod config)
- 🚧 Authentication (planned for multi-user)

## Related Pages

- [Data Flow](Data-Flow) - Detailed pipeline flows and sequence diagrams
- [Core Pipeline](Core-Pipeline) - Image processing and OCR details
- [LLM Bridge](LLM-Bridge) - Gemini and Ollama integration
- [REST API](REST-API) - API endpoint documentation
- [Web UI](Web-UI) - Yew frontend architecture
- [CLI](CLI) - Command-line interface documentation

---

**Last Updated:** 2025-11-16
**Architecture Version:** 1.0
