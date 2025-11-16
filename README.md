# scan3data

Process scanned images of IBM 1130 punch cards and computer listings into structured data for emulator consumption.

## Why "scan3data"?

The **3** represents our three-phase processing pipeline:

1. **Scan** - Ingest and digitize (image acquisition, duplicate detection, preprocessing)
2. **Classify & Correct** - Analyze and refine (OCR, LLM classification, ordering, gap detection)
3. **Convert** - Transform to structured output (emulator formats, reconstruction, export)

This isn't just a simple scanner - it's a complete **three-stage transformation pipeline** from messy historical scans to pristine emulator-ready data.

## Rust-First Philosophy

**This project is Rust-focused.** All business logic, data processing, and presentation code must be written in Rust.

- Frontend: Yew (Rust compiled to WebAssembly)
- Backend: Axum (pure Rust HTTP server)
- Image Processing: Rust crates (image, imageproc)
- OCR: Rust bindings (leptess for Tesseract)
- CLI: Rust (clap)

The only acceptable non-Rust code:
- Minimal HTML/CSS for web UI structure
- Calling external binaries via `Command::new()` when no Rust alternative exists

**No JavaScript. No TypeScript. No Python.**

## Project Structure

This is a multi-crate Cargo workspace:

```
scan3data/
+-- crates/
    +-- core_pipeline/    # Core processing logic (no networking)
    +-- llm_bridge/       # Ollama LLM integration
    +-- cli/              # Command-line interface
    +-- server/           # REST API backend
    +-- yew_frontend/     # Browser UI (Yew/WASM)
+-- scripts/              # Build and serve scripts
+-- docs/                 # Documentation
+-- test-data/            # Test scans (gitignored)
```

## Features

### Phase 1 (Current) - Non-LLM Baseline
- Classical image preprocessing (deskew, threshold, denoise)
- Tesseract OCR integration
- IBM 1130 object deck parsing
- 80-column punch card text extraction
- Duplicate detection (SHA-256 based)
- Export to emulator formats (JSON)

### Phase 2 (Planned) - LLM Integration
- Vision LLM (Qwen2.5-VL, Phi-Vision) for classification
- Text LLM (Qwen2.5, Phi-4) for refinement
- Automatic page/card ordering
- Gap detection and reconstruction

### Phase 3 (Future)
- Document denoising/super-resolution
- Reverse engineering from object decks
- Fine-tuned models for IBM 1130 specifics

## Quick Start

### Prerequisites

```bash
# Rust (latest stable)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# WASM target
rustup target add wasm32-unknown-unknown

# Trunk (for WASM builds)
cargo install trunk

# Tesseract (for OCR) and dependencies
# macOS:
brew install tesseract pkgconf

# Linux:
sudo apt-get install tesseract-ocr pkg-config libleptonica-dev libtesseract-dev
```

**Note**: The `leptess` crate requires Tesseract and Leptonica libraries. On macOS, `pkgconf` is needed for the build process.

### Build

```bash
# Build everything
./scripts/build-all.sh

# Or build components individually
./scripts/build-cli.sh      # CLI only
./scripts/build-server.sh   # Server only
./scripts/build-wasm.sh     # Frontend only
```

### Run

```bash
# Serve standalone SPA (all processing in browser)
./scripts/serve-spa.sh 8080

# Serve with backend API
./scripts/serve-api.sh

# Development mode (auto-rebuild)
./scripts/dev-wasm.sh

# CLI commands (three-phase pipeline)
cargo run -p scan3data-cli -- ingest -i ./scans -o ./output
cargo run -p scan3data-cli -- analyze -s ./output --use-llm
cargo run -p scan3data-cli -- export -s ./output -o deck.json
```

## Architecture

### Core Pipeline (core_pipeline)

Defines the **Canonical Intermediate Representation (CIR)**:

- `PageArtifact` - Scanned page metadata
- `CardArtifact` - Scanned card metadata
- `ArtifactKind` - Classification (text/object card, listing, etc.)
- `EmulatorOutput` - Final output format for IBM 1130 emulator

Provides:
- Image preprocessing (deskew, threshold, morphology)
- OCR baseline (Tesseract integration)
- IBM 1130 object deck decoder
- Disassembler

### LLM Bridge (llm_bridge)

Integrates with local Ollama for:
- Vision model calls (image -> classification)
- Text model calls (text -> refinement)
- Prompt templates

### CLI (cli)

Commands (three-phase pipeline):
- `ingest` - **Phase 1: Scan** - Import scans, detect duplicates, create scan set
- `analyze` - **Phase 2: Classify & Correct** - Classify artifacts, extract text, refine with LLM
- `export` - **Phase 3: Convert** - Generate emulator-ready output
- `serve` - Start web UI (SPA or API mode)

### Server (server)

REST API endpoints:
- `POST /api/scan_sets` - Create new scan set
- `POST /api/scan_sets/:id/upload` - Upload images
- `GET /api/scan_sets/:id/artifacts` - Get processed artifacts

### Frontend (yew_frontend)

Browser UI:
- File upload
- Image preview
- Page/card ordering (drag-drop)
- Manual classification correction
- Export controls

Two modes:
1. **Standalone SPA** - All processing in WASM (no backend)
2. **API Client** - Offload heavy processing to backend

## Duplicate Detection

The `test-data/` directory contains many duplicate scans with different filenames.

**How it works:**
1. During ingest, compute SHA-256 hash of each image
2. Identical images get same hash
3. Store only ONE copy of the image
4. Store ALL filenames in metadata

**Why this matters:**
- Saves storage space
- Filenames provide context for LLMs
- Example: `forth-p1.jpg` + `moore-1130-page1.jpg` -> "This is likely Chuck Moore's Forth"

## Development

### Workspace Commands

```bash
# Build all crates
cargo build --workspace

# Test all crates
cargo test --workspace

# Lint all crates (zero warnings)
cargo clippy --all-targets --all-features -- -D warnings

# Format all code
cargo fmt --all

# Generate docs
cargo doc --open --workspace
```

### Build Individual Crates

```bash
cargo build -p core_pipeline
cargo build -p llm_bridge
cargo build -p scan3data-cli
cargo build -p scan3data-server
cargo build -p yew_frontend  # Note: Use trunk for WASM
```

### Testing

```bash
# All tests
cargo test --workspace

# Specific crate
cargo test -p core_pipeline

# Specific test
cargo test test_duplicate_detection

# With output
cargo test -- --nocapture
```

## Contributing

See `docs/process.md` for:
- TDD workflow (Red/Green/Refactor)
- Pre-commit quality gates
- Code standards
- Tech debt limits

Key points:
- Zero clippy warnings (`-D warnings`)
- All code formatted with `cargo fmt`
- Files under 500 lines
- Functions under 50 lines
- Max 3 TODOs per file

## Documentation

- `CLAUDE.md` - Guide for Claude Code
- `docs/starting-prompt.txt` - Original requirements
- `docs/research.txt` - Research and design discussion
- `docs/notes.txt` - Project notes and requirements
- `docs/ai_agent_instructions.md` - AI agent guidelines
- `docs/process.md` - Development process
- `docs/tools.md` - Development tools

## License

MIT License - Copyright (c) 2025 Michael A Wright

See [LICENSE](LICENSE) file for details.
