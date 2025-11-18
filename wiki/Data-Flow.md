# Data Flow & Sequence Diagrams

This page details the data flows through scan3data's three-phase pipeline, including sequence diagrams for key operations.

## Table of Contents

- [Three-Phase Pipeline Overview](#three-phase-pipeline-overview)
- [Phase 1: Scan (Ingest)](#phase-1-scan-ingest)
- [Phase 2: Classify & Correct (Analyze)](#phase-2-classify--correct-analyze)
- [Phase 3: Convert (Export)](#phase-3-convert-export)
- [Web UI Workflows](#web-ui-workflows)
- [Duplicate Detection Flow](#duplicate-detection-flow)
- [Error Handling Flow](#error-handling-flow)

## Three-Phase Pipeline Overview

```mermaid
stateDiagram-v2
    direction LR
    [*] --> Phase1: Raw Scans
    Phase1 --> Phase2: ScanSet + Images
    Phase2 --> Phase3: Analyzed Artifacts
    Phase3 --> [*]: Emulator Files

    state "Phase 1: Scan" as Phase1 {
        [*] --> Ingest
        Ingest --> DuplicateDetect: SHA-256 Hash
        DuplicateDetect --> StoreCIR: Deduplicated
        StoreCIR --> [*]
    }

    state "Phase 2: Classify & Correct" as Phase2 {
        [*] --> Preprocess
        Preprocess --> ImageClean: Gemini API
        ImageClean --> OCR: Tesseract
        OCR --> LLMClassify: Vision Models
        LLMClassify --> Validate: IBM 1130 Rules
        Validate --> [*]
    }

    state "Phase 3: Convert" as Phase3 {
        [*] --> LoadCIR
        LoadCIR --> GenerateOutput: CardDeck/Listing
        GenerateOutput --> WriteFiles: JSON/Text
        WriteFiles --> [*]
    }
```

## Phase 1: Scan (Ingest)

The ingest phase loads raw scans, detects duplicates, and creates the initial ScanSet.

### CLI Ingest Sequence

```mermaid
sequenceDiagram
    participant User
    participant CLI as scan3data ingest
    participant FS as File System
    participant Core as Core Pipeline
    participant Hash as SHA-256 Hasher

    User->>CLI: ingest -i ./scans -o ./scan_set
    CLI->>FS: List image files
    FS-->>CLI: File paths

    loop For each image file
        CLI->>FS: Read image bytes
        FS-->>CLI: Image data
        CLI->>Hash: Compute SHA-256
        Hash-->>CLI: Hash digest
        CLI->>CLI: Check if hash exists

        alt New hash (unique image)
            CLI->>FS: Copy to scan_set/images/<hash>.jpg
            CLI->>Core: Create PageArtifact/CardArtifact
            Core-->>CLI: Artifact with original_filename
        else Duplicate hash
            CLI->>Core: Append filename to existing artifact
            Note over CLI,Core: Multiple filenames stored for same image hash
        end
    end

    CLI->>Core: Build ScanSet
    Core-->>CLI: ScanSet structure
    CLI->>FS: Write manifest.json
    CLI->>FS: Write artifacts.json
    CLI-->>User: ScanSet created at ./scan_set
```

### Ingest Data Transformations

```mermaid
graph LR
    A[Raw Image Files] -->|Read| B[Image Bytes]
    B -->|SHA-256| C{Hash Lookup}
    C -->|New| D[Store Image]
    C -->|Duplicate| E[Skip Storage]
    D --> F[Create Artifact]
    E --> G[Update Artifact Metadata]
    F --> H[PageArtifact/CardArtifact]
    G --> H
    H --> I[ScanSet]
    I -->|Serialize| J[artifacts.json]
```

### Ingest Output Structure

```
scan_set_001/
├── manifest.json          # ScanSetId, created_at, etc.
├── images/
│   ├── a1b2c3d4...jpg    # SHA-256 hash as filename
│   ├── e5f6g7h8...jpg    # Each file is unique
│   └── ...
└── artifacts.json         # All PageArtifact and CardArtifact records
```

## Phase 2: Classify & Correct (Analyze)

The analyze phase runs OCR, classifies artifacts, and optionally uses LLMs for enhancement.

### CLI Analyze Sequence (Baseline - No LLM)

```mermaid
sequenceDiagram
    participant User
    participant CLI as scan3data analyze
    participant FS as File System
    participant Core as Core Pipeline
    participant Tess as Tesseract OCR

    User->>CLI: analyze -s ./scan_set
    CLI->>FS: Read artifacts.json
    FS-->>CLI: ScanSet data

    loop For each artifact
        CLI->>FS: Read raw image
        FS-->>CLI: Image data

        CLI->>Core: preprocess_image()
        Core->>Core: Grayscale conversion
        Core->>Core: Deskew (Hough transform)
        Core->>Core: Adaptive threshold
        Core->>Core: Morphological ops
        Core-->>CLI: Preprocessed image

        CLI->>FS: Save processed image
        CLI->>Tess: Extract text (IBM 1130 whitelist)
        Tess-->>CLI: OCR text

        CLI->>Core: classify_artifact()
        Core->>Core: Apply heuristics (card vs listing, text vs binary)
        Core-->>CLI: ArtifactKind

        CLI->>Core: Update artifact metadata
    end

    CLI->>FS: Write updated artifacts.json
    CLI-->>User: Analysis complete
```

### CLI Analyze Sequence (LLM-Enhanced)

```mermaid
sequenceDiagram
    participant User
    participant CLI as scan3data analyze --use-llm
    participant Core as Core Pipeline
    participant LLM as LLM Bridge
    participant Gemini as Gemini API
    participant Ollama as Ollama Vision
    participant Tess as Tesseract

    User->>CLI: analyze -s ./scan_set --use-llm
    CLI->>Core: Load ScanSet

    loop For each artifact
        CLI->>Core: preprocess_image()
        Core-->>CLI: Preprocessed image

        alt Greenbar listing detected
            CLI->>LLM: clean_image()
            LLM->>Gemini: POST /v1/models/gemini-2.5-flash-image
            Gemini-->>LLM: Cleaned image
            LLM-->>CLI: Enhanced image
        end

        CLI->>Tess: Extract text baseline
        Tess-->>CLI: Raw OCR text

        CLI->>LLM: classify_with_vision()
        LLM->>Ollama: POST /api/generate (llama3.2-vision)
        Ollama-->>LLM: Classification + metadata
        LLM-->>CLI: ArtifactKind, confidence, page_number

        CLI->>LLM: refine_ocr_text()
        LLM->>Ollama: POST /api/generate (qwen2.5-vl)
        Ollama-->>LLM: Corrected OCR text
        LLM-->>CLI: Layout-preserved text

        CLI->>Core: Update artifact with LLM results
    end

    CLI->>Core: Save analyzed ScanSet
    CLI-->>User: Analysis complete (LLM-enhanced)
```

### Analyze Data Transformations

```mermaid
graph TB
    A[Raw Image] -->|Preprocess| B[Grayscale]
    B --> C[Deskewed]
    C --> D[Thresholded]
    D --> E[Denoised]
    E -->|Optional| F[Gemini Clean]
    E -->|Baseline| G[Tesseract OCR]
    F --> G
    G --> H{Use LLM?}
    H -->|Yes| I[Vision Model]
    H -->|No| J[Heuristic Classifier]
    I --> K[Refined OCR Text]
    J --> L[Basic Classification]
    K --> M[Updated Artifact]
    L --> M
    M --> N[artifacts.json]
```

## Phase 3: Convert (Export)

The export phase generates emulator-ready output from analyzed artifacts.

### CLI Export Sequence

```mermaid
sequenceDiagram
    participant User
    participant CLI as scan3data export
    participant FS as File System
    participant Core as Core Pipeline

    User->>CLI: export -s ./scan_set -o output.json
    CLI->>FS: Read artifacts.json
    FS-->>CLI: Analyzed ScanSet

    CLI->>Core: generate_emulator_output()

    alt Export as CardDeck
        Core->>Core: Collect text_80col from CardArtifacts
        Core->>Core: Order by sequence_number
        Core->>Core: Format as emulator card deck
        Core-->>CLI: EmulatorOutput::CardDeck
    else Export as Listing
        Core->>Core: Collect content_text from PageArtifacts
        Core->>Core: Order by page_number
        Core->>Core: Concatenate with headers
        Core-->>CLI: EmulatorOutput::Listing
    else Export as ObjectCode
        Core->>Core: Decode binary_80col from CardArtifacts
        Core->>Core: Parse IBM 1130 object deck format
        Core->>Core: Validate checksums
        Core-->>CLI: EmulatorOutput::ObjectCode
    end

    CLI->>FS: Write output file (JSON/text)
    CLI-->>User: Export complete: output.json
```

### Export Format Examples

#### CardDeck Format (JSON)

```json
{
  "format": "card_deck",
  "deck_name": "forth-1970",
  "cards": [
    {
      "sequence": "00010",
      "text": "       LATEST @ CFA NFA DUP C@ 31 AND"
    },
    {
      "sequence": "00020",
      "text": "       OVER 2+ + 1 MIN NEGATE AND"
    }
  ]
}
```

#### Listing Format (Text)

```
IBM 1130 SOURCE LISTING
Page 1 of 15
Date: 1970-06-15

00010        LATEST @ CFA NFA DUP C@ 31 AND
00020        OVER 2+ + 1 MIN NEGATE AND
00030        DUP >R - CMOVE R> LATEST !
```

## Web UI Workflows

The web UI implements a 4-stage pipeline with visual feedback and user interaction.

### Web UI Upload & Clean Sequence

```mermaid
sequenceDiagram
    participant User
    participant UI as Yew Frontend
    participant API as REST API
    participant LLM as LLM Bridge
    participant Gemini as Gemini API

    User->>UI: Select image file
    UI->>UI: Read file as bytes (File API)
    UI->>UI: Convert to base64
    UI->>UI: Display original image
    UI->>UI: Update stage: Upload → ImageCleaning

    User->>UI: Click "Clean Image"
    UI->>API: POST /api/clean-image { image_data: base64 }
    API->>LLM: clean_image(bytes)
    LLM->>LLM: Encode to base64
    LLM->>Gemini: POST /v1/models/gemini-2.5-flash-image Prompt: "Remove greenbar artifacts"
    Gemini-->>LLM: Cleaned image (base64)
    LLM-->>API: Cleaned image data
    API-->>UI: { cleaned_image_data: base64 }

    UI->>UI: Display cleaned image
    UI->>UI: Show side-by-side comparison
    UI->>UI: Update stage: ImageCleaning → OcrExtraction
    UI-->>User: Original vs Cleaned comparison
```

### Web UI OCR & Validation Sequence (Planned)

```mermaid
sequenceDiagram
    participant User
    participant UI as Yew Frontend
    participant API as REST API
    participant Core as Core Pipeline
    participant Tess as Tesseract

    User->>UI: Click "Run OCR"
    UI->>API: POST /api/ocr-extract { image_data: base64, use_cleaned: true }
    API->>Core: run_ocr(image_bytes)
    Core->>Core: Preprocess image
    Core->>Tess: Extract text (IBM 1130 whitelist)
    Tess-->>Core: Raw OCR text
    Core-->>API: { ocr_text: string }
    API-->>UI: OCR results

    UI->>UI: Display in editable textarea
    UI->>UI: Update stage: OcrExtraction → Validation
    User->>UI: Edit OCR text (manual corrections)

    User->>UI: Click "Validate"
    UI->>UI: Apply IBM 1130 validation rules

    alt Validation errors found
        UI->>UI: Highlight errors (O vs 0, I vs 1)
        UI->>UI: Show suggestions
        UI-->>User: Errors highlighted with fixes
    else No errors
        UI->>UI: Mark as validated
        UI->>UI: Enable export button
        UI-->>User: Ready to export
    end
```

### Web UI State Machine

```mermaid
stateDiagram-v2
    [*] --> Upload: Page Load
    Upload --> ImageCleaning: File Selected
    ImageCleaning --> ImageCleaning: Cleaning in Progress
    ImageCleaning --> OcrExtraction: Clean Complete
    OcrExtraction --> OcrExtraction: OCR in Progress
    OcrExtraction --> Validation: OCR Complete
    Validation --> Validation: Validating
    Validation --> Export: Validated
    Export --> [*]: Download Complete

    note right of Upload
        User selects image file
        Display original
    end note

    note right of ImageCleaning
        Call Gemini API
        Side-by-side comparison
    end note

    note right of OcrExtraction
        Call Tesseract OCR
        Editable textarea
    end note

    note right of Validation
        IBM 1130 rules
        Error highlighting
    end note
```

## Duplicate Detection Flow

Detailed flow for SHA-256-based duplicate detection during ingest.

### Duplicate Detection Algorithm

```mermaid
flowchart TD
    Start([Start: Process Image File]) --> ReadFile[Read image file bytes]
    ReadFile --> ComputeHash[Compute SHA-256 hash]
    ComputeHash --> CheckHash{Hash in hash_map?}

    CheckHash -->|No - New image| StoreImage[Store image as hash.jpg]
    StoreImage --> CreateArtifact[Create new PageArtifact/CardArtifact]
    CreateArtifact --> AddFilename[Add original_filename to metadata]
    AddFilename --> UpdateMap[Add hash to hash_map]
    UpdateMap --> Continue([Continue to next file])

    CheckHash -->|Yes - Duplicate| LookupArtifact[Lookup existing artifact by hash]
    LookupArtifact --> AppendFilename[Append filename to original_filenames array]
    AppendFilename --> Continue

    Continue --> MoreFiles{More files?}
    MoreFiles -->|Yes| ReadFile
    MoreFiles -->|No| SaveCIR[Save artifacts.json]
    SaveCIR --> End([End])
```

### Duplicate Detection Example

**Input Files:**
```
scans/
├── forth-p1.jpg          # 512 KB
├── moore-1130-page1.jpg  # 510 KB (same image, different filename)
└── forth-p2.jpg          # 498 KB (different image)
```

**After Ingest:**
```
scan_set/
├── images/
│   ├── a1b2c3d4e5f6...jpg  # forth-p1.jpg and moore-1130-page1.jpg (same hash)
│   └── 7g8h9i0j1k2l...jpg  # forth-p2.jpg
└── artifacts.json
```

**artifacts.json:**
```json
{
  "pages": [
    {
      "page_id": "uuid-001",
      "raw_image_path": "images/a1b2c3d4e5f6...jpg",
      "metadata": {
        "original_filenames": [
          "forth-p1.jpg",
          "moore-1130-page1.jpg"
        ],
        "notes": ["Duplicate detected - same hash"]
      }
    },
    {
      "page_id": "uuid-002",
      "raw_image_path": "images/7g8h9i0j1k2l...jpg",
      "metadata": {
        "original_filenames": ["forth-p2.jpg"]
      }
    }
  ]
}
```

**Why This Matters:**
- Saves disk space (one copy instead of two)
- Filenames provide context hints for LLMs
- Example: "forth" + "moore" + "1130" → High confidence this is Chuck Moore's Forth

## Error Handling Flow

### Error Propagation Through Pipeline

```mermaid
sequenceDiagram
    participant User
    participant CLI
    participant Core
    participant External as External Service (Tesseract, Gemini, etc.)

    User->>CLI: Run command
    CLI->>Core: Process operation
    Core->>External: Call external service

    alt Success Path
        External-->>Core: Success result
        Core-->>CLI: Ok(data)
        CLI-->>User: Success message
    else Error Path
        External--xCore: Error (network, API, etc.)
        Core->>Core: Wrap error (anyhow::Context)
        Core--xCLI: Err(error)
        CLI->>CLI: Log error (tracing)
        CLI->>CLI: Format user-friendly message
        CLI-->>User: Error message + suggestion
    end
```

### Error Handling Strategy

```mermaid
graph TB
    Error[Error Occurs] --> Classify{Error Type?}

    Classify -->|I/O Error| IO[File not found, permission denied]
    Classify -->|Network Error| NET[Connection failed, timeout]
    Classify -->|OCR Error| OCR[Tesseract failed, bad image]
    Classify -->|LLM Error| LLM[API key invalid, rate limit]
    Classify -->|Validation Error| VAL[Invalid format, bad data]

    IO --> Retry{Retryable?}
    NET --> Retry
    OCR --> Fallback{Fallback available?}
    LLM --> Fallback
    VAL --> Log[Log error]

    Retry -->|Yes| RetryOp[Retry with exponential backoff]
    Retry -->|No| Log
    RetryOp -->|Success| Continue[Continue]
    RetryOp -->|Failed| Log

    Fallback -->|Yes| UseFallback[Use baseline method]
    Fallback -->|No| Log
    UseFallback --> Continue

    Log --> Report[Report to user]
    Report --> Decision{Critical?}
    Decision -->|Yes| Abort[Abort operation]
    Decision -->|No| Continue

    Continue --> End([Continue processing])
    Abort --> End
```

### Example Error Flows

#### OCR Failure with Fallback

```mermaid
sequenceDiagram
    participant CLI
    participant Core
    participant Tess as Tesseract
    participant LLM as Vision LLM

    CLI->>Core: run_ocr(image)
    Core->>Tess: Extract text
    Tess--xCore: Error: Image quality too low
    Core->>Core: Log warning
    Core->>LLM: Try vision model fallback

    alt Vision model succeeds
        LLM-->>Core: OCR text (lower confidence)
        Core-->>CLI: Ok(text) with warning
        CLI-->>CLI: Log: "Used fallback vision model"
    else Vision model also fails
        LLM--xCore: Error: All OCR methods failed
        Core-->>CLI: Err("Could not extract text")
        CLI-->>CLI: Log error, skip artifact
    end
```

#### Gemini API Rate Limit

```mermaid
sequenceDiagram
    participant UI
    participant API
    participant LLM
    participant Gemini

    UI->>API: POST /api/clean-image
    API->>LLM: clean_image()
    LLM->>Gemini: POST request
    Gemini--xLLM: 429 Too Many Requests Retry-After: 60s
    LLM->>LLM: Exponential backoff retry (2s, 4s, 8s, 16s)

    alt Retry succeeds
        Gemini-->>LLM: 200 OK (cleaned image)
        LLM-->>API: Success
        API-->>UI: { cleaned_image_data }
    else All retries fail
        LLM--xAPI: Error: Rate limit exceeded
        API-->>UI: 503 Service Unavailable { error: "Try again in 60s" }
        UI->>UI: Show error toast
    end
```

## Related Pages

- [Architecture](Architecture) - System architecture and component diagrams
- [Core Pipeline](Core-Pipeline) - Detailed image processing flows
- [LLM Bridge](LLM-Bridge) - API integration details
- [REST API](REST-API) - API endpoint specifications
- [Web UI](Web-UI) - Frontend component interactions
- [CLI](CLI) - Command-line usage and workflows

---

**Last Updated:** 2025-11-16
**Flow Diagrams Version:** 1.0
