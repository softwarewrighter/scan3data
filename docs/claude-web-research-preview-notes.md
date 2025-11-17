# Claude Web Research Preview Notes - scan3data

**Session Date**: 2025-11-16
**Branch**: `claude/document-research-preview-notes-01D1haQ1u8tfYiZfK2PAWKbd`
**Project**: scan3data v0.1.0

## Executive Summary

scan3data is a three-phase pipeline for processing scanned IBM 1130 punch cards and computer listings into emulator-ready structured data. The project is built as a multi-crate Rust workspace with strong emphasis on Test-Driven Development, zero-warning code quality, and a Rust-first philosophy (Yew/WASM frontend, Axum backend).

**Current State**: Phase 1 implementation is substantially complete with working CLI ingest/analyze/export pipeline. Phase 2 (LLM integration) has initial Gemini API integration for image cleaning and a functional 4-stage web UI pipeline visualization.

## Recent Achievements (Last 5 Commits)

1. **c720f49** - Yew UI pipeline visualization with port 7214
   - 4-stage pipeline UI (Upload -> Image Cleaning -> OCR -> Validation)
   - Integrated Gemini 2.5 Flash Image API for greenbar removal
   - Side-by-side original vs cleaned image comparison

2. **1373853** - Comprehensive CLI metadata and sw-checklist compliance
   - Added build metadata (version, git commit, build date)
   - Enhanced `--version` output with detailed build info

3. **691c2cf** - Gemini 2.5 Flash Image (Nano Banana) greenbar removal
   - AI-powered image cleaning to remove greenbar artifacts
   - REST API endpoint `/api/clean-image` for frontend integration

4. **f6704db** - Yew UI pipeline for IBM 1130 OCR processing (TDD)
   - Pipeline component with stage progression
   - Upload component with file handling

5. **61be23a** - HTML side-by-side comparison viewer for OCR verification
   - Visual comparison of original vs enhanced images

## Current Capabilities

### Core Pipeline (Phase 1 - Completed)

#### CLI Commands
- **ingest**: Import scans, detect duplicates (SHA-256), create scan sets
- **analyze**: Run Tesseract OCR, classify artifacts, extract text
- **export**: Generate emulator-ready JSON output
- **serve**: Launch web UI (SPA or API mode)

#### Image Processing
- Grayscale conversion
- Deskewing (Hough transform)
- Adaptive thresholding
- Morphological operations
- Duplicate detection via SHA-256 hashing

#### OCR Integration
- Tesseract OCR with IBM 1130 character whitelist
- 80-column punch card text extraction
- Layout preservation
- IBM 1130 object deck parsing (basic)

### LLM Integration (Phase 2 - Partial)

#### Gemini API Integration
- Gemini 2.5 Flash Image for greenbar artifact removal
- REST endpoint: `POST /api/clean-image`
- Base64 image encoding/decoding
- Cost: $0.039 per image

#### Ollama Vision Models (Scaffolded)
- llama3.2-vision, Qwen2.5-VL support
- Vision model wrapper in `llm_bridge` crate
- Prompts for IBM 1130-specific OCR correction
- Layout-preserving OCR validation

### Web UI (Yew/WASM)

#### 4-Stage Pipeline Visualization
1. **Upload** - File selection via drag-drop or file input
2. **Image Cleaning** - Gemini API integration, original vs cleaned comparison
3. **OCR Extraction** - Placeholder (needs Tesseract WASM integration)
4. **Validation** - IBM 1130-specific error detection (planned)

#### Current Features
- File upload with base64 encoding
- Async API calls to backend (gloo-net)
- Stage progression with visual indicators
- Image preview (original and cleaned)

### Backend (Axum Server)

#### Endpoints
- `GET /health` - Health check
- `POST /api/scan_sets` - Create scan set (placeholder)
- `POST /api/scan_sets/:id/upload` - Upload images (placeholder)
- `GET /api/scan_sets/:id/artifacts` - Get artifacts (placeholder)
- `POST /api/clean-image` - Clean image via Gemini API (working)

#### Configuration
- Port: 7214 (unified for API and static files)
- CORS: Permissive (for development)
- Static file serving: `dist/` directory for Yew frontend
- Tracing: Enabled for HTTP requests

### Architecture

#### Multi-Crate Workspace (5 crates)
1. **core_pipeline** - Core types, image preprocessing, OCR, decoder
2. **llm_bridge** - Gemini API client, Ollama client (partial)
3. **cli** - Command-line interface (scan3data binary)
4. **server** - REST API backend (scan3data-server binary)
5. **yew_frontend** - Browser UI compiled to WASM

#### Canonical Intermediate Representation (CIR)
- `ScanSet` with `PageArtifact` and `CardArtifact`
- Metadata includes: filenames (for deduplication), page numbers, confidence scores
- Support for text and binary card formats
- Extensible `ArtifactKind` enum

## Known Limitations & TODOs

### High Priority

#### OCR Stage (Web UI)
- **TODO**: Integrate Tesseract WASM or call backend OCR endpoint
- **TODO**: Implement editable textarea for manual text correction
- **TODO**: Add "Re-run OCR" functionality

#### Validation Stage (Web UI)
- **TODO**: Implement IBM 1130-specific validation rules
  - Hex sequence detection
  - Character pattern validation
  - Column alignment checks
- **TODO**: Highlight errors with suggestions
- **TODO**: Add "Accept" and "Export" actions

#### Backend API Implementation
- **TODO**: Implement actual scan set storage (currently placeholder)
- **TODO**: Add multipart/form-data upload handling
- **TODO**: Implement job queue for long-running operations
- **TODO**: Add WebSocket support for progress updates

#### Testing
- **TODO**: Write unit tests for Yew components (wasm-bindgen-test)
- **TODO**: Add integration tests for API endpoints
- **TODO**: Create end-to-end pipeline tests with test-data

### Medium Priority

#### IBM 1130 Object Deck Decoder
- **TODO**: Complete object deck format parsing
- **TODO**: Implement checksum validation
- **TODO**: Add disassembler support

#### Ollama Vision Integration
- **TODO**: Complete vision model OCR correction pipeline
- **TODO**: Add confidence scoring
- **TODO**: Implement fallback to Tesseract-only mode

#### UI Enhancements
- **TODO**: Drag-and-drop page/card ordering
- **TODO**: Batch processing (multiple images)
- **TODO**: Export controls (format selection, download)
- **TODO**: Progress indicators for long operations

#### Documentation
- **TODO**: Add API documentation (OpenAPI/Swagger)
- **TODO**: Create user guide with screenshots
- **TODO**: Add deployment guide

### Low Priority

#### Performance Optimization
- **TODO**: Add caching for LLM responses
- **TODO**: Implement parallel image processing (rayon)
- **TODO**: Optimize WASM bundle size (wasm-opt)

#### Advanced Features
- **TODO**: Custom IBM 1130 character recognition models
- **TODO**: Forth-specific syntax analysis
- **TODO**: Reverse engineering from object code
- **TODO**: Multi-user support with authentication

## Recommended Next Steps

### Immediate Actions (Next Session)

1. **Complete OCR Stage in Web UI** (High Impact)
   - Add backend endpoint: `POST /api/ocr-extract`
   - Call Tesseract OCR from backend (already working in CLI)
   - Return OCR text to frontend
   - Implement editable textarea with save functionality
   - **Estimated effort**: 2-3 hours

2. **Implement Basic Validation Stage** (High Impact)
   - Create validation rules for IBM 1130 format
   - Highlight common OCR errors (O vs 0, I vs 1, etc.)
   - Add character whitelist validation
   - **Estimated effort**: 2-4 hours

3. **End-to-End Pipeline Test** (Critical)
   - Upload real test image from `test-data/`
   - Verify full pipeline: Upload -> Clean -> OCR -> Validate
   - Document any issues or improvements needed
   - **Estimated effort**: 1-2 hours

### Short-Term Goals (1-2 Weeks)

4. **Backend Scan Set Storage**
   - Implement file system-based scan set persistence
   - Store uploaded images and metadata
   - Connect to core_pipeline for processing
   - **Estimated effort**: 4-6 hours

5. **Batch Processing Support**
   - Allow multiple image uploads
   - Process images in parallel (backend)
   - Show progress for batch operations
   - **Estimated effort**: 4-6 hours

6. **Comprehensive Testing**
   - Unit tests for all components
   - Integration tests for API
   - End-to-end tests with test-data
   - **Estimated effort**: 8-10 hours

7. **Export Functionality**
   - Allow downloading OCR results (text, JSON)
   - Support multiple export formats
   - Generate emulator-ready card decks
   - **Estimated effort**: 3-4 hours

### Medium-Term Goals (1 Month)

8. **Ollama Vision Model Integration**
   - Complete vision model pipeline
   - Implement OCR correction with layout preservation
   - Add confidence scoring and manual review
   - **Estimated effort**: 10-15 hours

9. **IBM 1130 Object Deck Decoder**
   - Research and implement full decoder
   - Add disassembler support
   - Create comprehensive tests
   - **Estimated effort**: 15-20 hours

10. **Deployment and Documentation**
    - Create deployment guide (Docker, systemd, etc.)
    - Write user manual with screenshots
    - Add API documentation
    - Create video tutorials
    - **Estimated effort**: 10-12 hours

## Development Guidelines

### Quality Standards (Mandatory)

#### Pre-Commit Checklist
1. Run tests: `cargo test --workspace` (all must pass)
2. Fix linting: `cargo clippy --all-targets --all-features -- -D warnings` (zero warnings)
3. Format code: `cargo fmt --all`
4. Validate markdown: `markdown-checker -f "**/*.md"` (if docs changed)
5. Update documentation: `docs/learnings.md`, README.md, CLAUDE.md (if needed)

#### Code Quality Metrics
- Source files: < 500 lines (prefer 200-300)
- Functions: < 50 lines (prefer 10-30)
- TODO comments: max 3 per file
- Cyclomatic complexity: < 10
- Zero clippy warnings
- Zero test failures

### Test-Driven Development (TDD)

Follow strict Red/Green/Refactor cycle:
1. **RED**: Write failing test defining desired behavior
2. **GREEN**: Write minimal code to pass test
3. **REFACTOR**: Improve code while keeping tests green
4. **REPEAT**: Continue for next functionality

### Build Commands

```bash
# Build all
./scripts/build-all.sh

# Build individual components
./scripts/build-cli.sh      # CLI only
./scripts/build-server.sh   # Server only
./scripts/build-wasm.sh     # Frontend only

# Development
./scripts/dev-wasm.sh       # Watch and auto-rebuild WASM

# Testing
cargo test --workspace      # All tests
cargo test -p core_pipeline # Specific crate

# Linting
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all
```

### Serving Modes

```bash
# SPA mode (standalone, browser-only processing)
./scripts/serve-spa.sh [PORT]

# API mode (backend + frontend, full LLM integration)
./scripts/serve-api.sh

# Manual
cargo run -p scan3data-cli -- serve --mode spa --port 8080
cargo run -p scan3data-server  # Port 7214
```

## Technical Debt & Issues

### Current Technical Debt

1. **Backend API Placeholders**
   - Most endpoints return placeholder data
   - No actual scan set storage or job queue
   - Needs proper implementation before production use

2. **Error Handling in Frontend**
   - Basic error logging to console
   - No user-visible error messages
   - Should add proper error UI components

3. **WASM OCR Integration**
   - OCR stage currently has placeholder text
   - Need to decide: WASM Tesseract vs backend API calls
   - Performance implications need benchmarking

4. **Test Coverage**
   - Limited unit tests for new components
   - No integration tests for API endpoints
   - No end-to-end pipeline tests

5. **Documentation Gaps**
   - API documentation incomplete
   - User guide needs screenshots and examples
   - Deployment guide missing

### Known Issues

1. **CORS Configuration**
   - Currently using permissive CORS for development
   - Needs proper configuration for production

2. **Image Size Limits**
   - No validation or size limits on uploads
   - Could cause memory issues with large images

3. **Gemini API Error Handling**
   - Basic error handling, needs retry logic
   - No rate limiting or quota management

4. **Port Hardcoding**
   - Frontend hardcodes `localhost:7214` for API calls
   - Should use environment variable or config

## Environment Setup

### Required Tools

```bash
# Rust (latest stable)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# WASM target
rustup target add wasm32-unknown-unknown

# Trunk (WASM build tool)
cargo install trunk

# Tesseract (OCR engine)
# macOS:
brew install tesseract pkgconf

# Linux:
sudo apt-get install tesseract-ocr pkg-config libleptonica-dev libtesseract-dev
```

### Environment Variables

```bash
# Required for Gemini image cleaning
export GEMINI_API_KEY="your-gemini-api-key"

# Optional: For Ollama (runs locally)
# Default: http://localhost:11434
```

### Development Tools (in ~/.local/softwarewrighter/bin/)

- `proact` - AI agent documentation generator
- `markdown-checker` - Markdown validator (ASCII-only enforcement)
- `ask` - Command-line LLM query tool

## Project Philosophy

### Rust-First Approach
- **Frontend**: Yew (Rust -> WASM), NOT JavaScript
- **Backend**: Axum (Rust), NOT Node.js/Python
- **Image Processing**: Rust crates, NOT Python libraries
- **Only acceptable non-Rust**: minimal HTML/CSS, external binaries when unavoidable

### Three-Phase Pipeline
1. **Scan** - Ingest and digitize (acquisition, deduplication, preprocessing)
2. **Classify & Correct** - Analyze and refine (OCR, LLM, ordering, gap detection)
3. **Convert** - Transform to structured output (emulator formats, export)

### Quality Over Speed
- Zero warnings policy
- All tests must pass
- TDD workflow strictly followed
- Documentation updated with every change
- Root cause analysis for all bugs (docs/learnings.md)

## Success Metrics

### Phase 1 Baseline (Achieved)
- [x] CLI ingest command with duplicate detection
- [x] CLI analyze command with Tesseract OCR
- [x] CLI export command for emulator formats
- [x] SHA-256 duplicate detection working
- [x] IBM 1130 character whitelist configured

### Phase 2 LLM Integration (In Progress)
- [x] Gemini 2.5 Flash Image integration for greenbar removal
- [x] Web UI with 4-stage pipeline visualization
- [x] Image upload and cleaning working end-to-end
- [ ] OCR stage integrated with backend
- [ ] Validation stage with IBM 1130 rules
- [ ] Ollama vision model integration complete
- [ ] Manual text editing and correction

### Phase 3 Advanced Features (Future)
- [ ] IBM 1130 disassembler working
- [ ] Forth-specific syntax analysis
- [ ] Custom character recognition models
- [ ] Multi-user collaborative editing
- [ ] Automated test generation from listings

## Resources

### Documentation
- `CLAUDE.md` - Development guidelines for Claude Code
- `docs/ai_agent_instructions.md` - Full AI agent development guide
- `docs/process.md` - TDD workflow and pre-commit process
- `docs/architecture.md` - System architecture and design
- `docs/next-steps.md` - Detailed roadmap with tasks
- `docs/tools.md` - Development tools documentation

### External Resources
- IBM 1130 documentation (various online sources)
- Tesseract OCR documentation
- Yew framework guide: https://yew.rs
- Axum web framework: https://docs.rs/axum
- Gemini API: https://ai.google.dev/

### Test Data
- `test-data/` directory (gitignored)
- Contains real IBM 1130 scans
- Includes duplicates with different filenames
- Useful for end-to-end testing

## Next Session Recommendations

### Priority 1: Complete Web UI Pipeline
Focus on making the 4-stage pipeline fully functional:
1. Implement OCR endpoint and frontend integration
2. Add basic validation rules
3. Test with real images from test-data
4. Document any issues or improvements

### Priority 2: Testing Infrastructure
Add comprehensive tests to prevent regressions:
1. Unit tests for Yew components
2. Integration tests for API endpoints
3. End-to-end pipeline tests
4. Update CI/CD if applicable

### Priority 3: Documentation
Keep documentation in sync with implementation:
1. Update README.md with current UI screenshots
2. Document API endpoints (current state)
3. Update next-steps.md to reflect completion
4. Add learnings to docs/learnings.md

### Long-Term Vision
The ultimate goal is a production-ready tool for the IBM 1130 preservation community:
- Process Chuck Moore's Forth scans
- Extract and validate historical source code
- Generate emulator-ready datasets
- Enable reverse engineering from object decks
- Preserve computing history

---

**Document Version**: 1.0
**Last Updated**: 2025-11-16
**Next Review**: After OCR stage completion
