# CLI (Command-Line Interface)

The **cli** crate provides the `scan3data` command-line tool for batch processing IBM 1130 scans through the three-phase pipeline.

## Overview

**Crate Name:** `cli` (binary: scan3data)
**Framework:** clap 4.0 (derive API)
**Commands:** ingest, analyze, export, serve
**Dependencies:** clap, core_pipeline, llm_bridge, tokio

## Commands

### ingest - Phase 1: Scan

Import raw scans, detect duplicates, create scan set.

```bash
scan3data ingest -i <INPUT_DIR> -o <OUTPUT_DIR>
```

**Options:**
- `-i, --input <DIR>` - Directory containing scanned images (required)
- `-o, --output <DIR>` - Output directory for scan set (required)
- `-v, --verbose` - Enable verbose logging

**Example:**
```bash
scan3data ingest -i ./test-data -o ./scan_set_001
```

**Output Structure:**
```
scan_set_001/
├── manifest.json
├── images/
│   ├── a1b2c3d4e5f6...jpg
│   └── ...
└── artifacts.json
```

**What It Does:**
1. Recursively scans input directory for images (JPEG, PNG, TIFF)
2. Computes SHA-256 hash for each image
3. Detects duplicates (same hash)
4. Stores unique images in `images/` directory
5. Creates PageArtifact/CardArtifact with all original filenames
6. Writes `manifest.json` and `artifacts.json`

### analyze - Phase 2: Classify & Correct

Run OCR, classify artifacts, optionally use LLMs.

```bash
scan3data analyze -s <SCAN_SET> [OPTIONS]
```

**Options:**
- `-s, --scan-set <DIR>` - Scan set directory (required)
- `--use-llm` - Enable LLM enhancement (Gemini, Ollama)
- `-v, --verbose` - Enable verbose logging

**Example (Baseline - No LLM):**
```bash
scan3data analyze -s ./scan_set_001
```

**Example (LLM-Enhanced):**
```bash
scan3data analyze -s ./scan_set_001 --use-llm
```

**What It Does (Baseline):**
1. Loads scan set from `artifacts.json`
2. For each artifact:
   - Preprocess image (deskew, threshold, denoise)
   - Run Tesseract OCR with IBM 1130 whitelist
   - Classify artifact (heuristics: card vs listing, text vs binary)
   - Extract 80-column text (for cards)
3. Updates `artifacts.json` with OCR results and classifications

**What It Does (LLM-Enhanced):**
1. All baseline steps, plus:
2. Call Gemini API to remove greenbar artifacts
3. Call Ollama vision model for better classification
4. Call Ollama text model to refine OCR results
5. Extract metadata (page numbers, deck names)
6. Store LLM results with confidence scores

### export - Phase 3: Convert

Generate emulator-ready output from analyzed artifacts.

```bash
scan3data export -s <SCAN_SET> -o <OUTPUT_FILE> [OPTIONS]
```

**Options:**
- `-s, --scan-set <DIR>` - Scan set directory (required)
- `-o, --output <FILE>` - Output file path (required)
- `-f, --format <FORMAT>` - Output format: `card-deck`, `listing`, `object-code` (default: `card-deck`)
- `-v, --verbose` - Enable verbose logging

**Example (Card Deck):**
```bash
scan3data export -s ./scan_set_001 -o forth-deck.json -f card-deck
```

**Example (Listing):**
```bash
scan3data export -s ./scan_set_001 -o listing.txt -f listing
```

**Output Formats:**

**card-deck (JSON):**
```json
{
  "format": "card_deck",
  "deck_name": "forth-1970",
  "cards": [
    {"sequence": "00010", "text": "       LATEST @ CFA NFA DUP C@ 31 AND"},
    {"sequence": "00020", "text": "       OVER 2+ + 1 MIN NEGATE AND"}
  ]
}
```

**listing (Text):**
```
IBM 1130 SOURCE LISTING
Date: 1970-06-15

00010        LATEST @ CFA NFA DUP C@ 31 AND
00020        OVER 2+ + 1 MIN NEGATE AND
```

### serve - Launch Web UI

Start the web server (REST API + Yew frontend).

```bash
scan3data serve [OPTIONS]
```

**Options:**
- `-p, --port <PORT>` - Port to listen on (default: 7214)
- `--mode <MODE>` - Server mode: `spa`, `api` (default: `api`)
- `-v, --verbose` - Enable verbose logging

**Example (API mode):**
```bash
scan3data serve --port 7214 --mode api
```

**Example (SPA mode):**
```bash
scan3data serve --port 8080 --mode spa
```

**Modes:**
- `api`: Full backend + frontend (recommended)
- `spa`: Serve static files only (frontend-only processing)

## Global Options

```bash
scan3data [OPTIONS] <COMMAND>
```

**Options:**
- `-h, --help` - Print help information
- `-V, --version` - Print version information (includes build metadata)
- `-v, --verbose` - Enable verbose logging (can be used with any command)

**Version Output:**
```
scan3data 0.1.0
Git Commit: c720f49
Build Date: 2025-11-16
```

## Implementation

### Command Structure

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "scan3data")]
#[command(about = "IBM 1130 scan processing pipeline")]
#[command(version = VERSION_INFO)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    Ingest {
        #[arg(short, long)]
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
    },
    Analyze {
        #[arg(short, long)]
        scan_set: PathBuf,
        #[arg(long)]
        use_llm: bool,
    },
    Export {
        #[arg(short, long)]
        scan_set: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(short, long, default_value = "card-deck")]
        format: String,
    },
    Serve {
        #[arg(short, long, default_value = "7214")]
        port: u16,
        #[arg(long, default_value = "api")]
        mode: String,
    },
}
```

### Ingest Implementation

```rust
async fn ingest(input: PathBuf, output: PathBuf) -> Result<()> {
    info!("Ingesting from {:?} to {:?}", input, output);

    let mut scan_set = ScanSet::new();
    let mut hash_map = HashMap::new();

    for entry in WalkDir::new(&input) {
        let entry = entry?;
        if !is_image_file(&entry) {
            continue;
        }

        let bytes = std::fs::read(entry.path())?;
        let hash = compute_sha256(&bytes);

        if let Some(artifact_id) = hash_map.get(&hash) {
            // Duplicate: append filename
            scan_set.append_filename(*artifact_id, entry.path())?;
        } else {
            // New: create artifact
            let artifact = create_artifact(entry.path(), &hash)?;
            hash_map.insert(hash, artifact.id());
            scan_set.add_artifact(artifact);

            // Copy image
            copy_image(&bytes, &output, &hash)?;
        }
    }

    scan_set.save(&output)?;
    Ok(())
}
```

## Environment Variables

### GEMINI_API_KEY

Required for LLM-enhanced analysis (`--use-llm`).

```bash
export GEMINI_API_KEY="your-api-key-here"
scan3data analyze -s ./scan_set --use-llm
```

Get API key from: https://ai.google.dev/

### OLLAMA_BASE_URL

Optional. Defaults to `http://localhost:11434`.

```bash
export OLLAMA_BASE_URL="http://192.168.1.100:11434"
scan3data analyze -s ./scan_set --use-llm
```

### RUST_LOG

Control logging verbosity.

```bash
export RUST_LOG=scan3data=debug
scan3data ingest -i ./scans -o ./scan_set
```

**Levels:** error, warn, info, debug, trace

## Complete Workflow Example

### Step 1: Ingest Scans

```bash
# Import scans, detect duplicates
scan3data ingest -i ./test-data -o ./forth_scan_set

# Output:
# Processed 15 files
# Found 3 duplicates
# Created scan set: forth_scan_set
# Unique images: 12
```

### Step 2: Analyze (Baseline)

```bash
# Run OCR without LLMs
scan3data analyze -s ./forth_scan_set

# Output:
# Analyzing 12 artifacts...
# [1/12] Processing page-001.jpg... OCR complete (0.89 confidence)
# [2/12] Processing page-002.jpg... OCR complete (0.92 confidence)
# ...
# Analysis complete: 12 artifacts processed
```

### Step 3: Analyze (LLM-Enhanced)

```bash
# Run with Gemini + Ollama
export GEMINI_API_KEY="your-key"
scan3data analyze -s ./forth_scan_set --use-llm

# Output:
# Analyzing 12 artifacts with LLM enhancement...
# [1/12] Cleaning image... Gemini API complete
# [1/12] Running OCR... Tesseract complete
# [1/12] Classifying... Ollama vision complete (SourceListing, 0.97)
# [1/12] Refining text... Ollama text complete
# ...
# Analysis complete: 12 artifacts processed
# Average confidence: 0.94
```

### Step 4: Export Results

```bash
# Export as card deck
scan3data export -s ./forth_scan_set -o forth-deck.json -f card-deck

# Export as listing
scan3data export -s ./forth_scan_set -o forth-listing.txt -f listing

# Output:
# Exported 12 artifacts to forth-deck.json
# Format: card-deck
```

## Error Handling

### Common Errors

**Missing input directory:**
```
Error: Input directory does not exist: ./nonexistent
```

**Missing Gemini API key (with --use-llm):**
```
Error: GEMINI_API_KEY environment variable not set
Hint: Get an API key from https://ai.google.dev/
```

**Ollama not running (with --use-llm):**
```
Error: Failed to connect to Ollama at http://localhost:11434
Hint: Start Ollama with: ollama serve
```

**Invalid image file:**
```
Warning: Skipping invalid image: ./scans/corrupt.jpg
Error: Image decode failed
```

## Testing

### Manual Testing

```bash
# Test ingest
scan3data ingest -i ./test-data -o /tmp/test_scan_set

# Verify output
ls /tmp/test_scan_set/images/
cat /tmp/test_scan_set/artifacts.json | jq .

# Test analyze
scan3data analyze -s /tmp/test_scan_set

# Test export
scan3data export -s /tmp/test_scan_set -o /tmp/output.json
```

### Integration Tests

```rust
#[tokio::test]
async fn test_ingest_command() {
    let temp_dir = tempfile::tempdir().unwrap();
    let output_dir = temp_dir.path().join("scan_set");

    let result = ingest("./test-data".into(), output_dir.clone()).await;
    assert!(result.is_ok());

    // Verify output
    assert!(output_dir.join("manifest.json").exists());
    assert!(output_dir.join("artifacts.json").exists());
}
```

## Related Pages

- [Core Pipeline](Core-Pipeline) - Core processing used by CLI
- [LLM Bridge](LLM-Bridge) - LLM integration used with --use-llm
- [Data Flow](Data-Flow) - CLI workflow sequence diagrams
- [Building](Building) - Build instructions

---

**Last Updated:** 2025-11-16
