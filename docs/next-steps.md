# Next Steps for scan3data

## Immediate Priorities (Phase 1 Implementation)

### 1. Implement Duplicate Detection (High Priority)
**Why**: Critical for test-data directory with many duplicate scans

**Tasks**:
- [ ] Add SHA-256 hashing to `core_pipeline/src/preprocess.rs`
- [ ] Create `ImageHash` type in `core_pipeline/src/types.rs`
- [ ] Update `PageMetadata` and `CardMetadata` with `original_filenames: Vec<String>`
- [ ] Implement deduplication logic in CLI `ingest` command
- [ ] Write tests with synthetic duplicate images
- [ ] Test with actual test-data directory

**Crates to use**:
```toml
sha2 = "0.10"
```

**Implementation sketch**:
```rust
use sha2::{Sha256, Digest};

pub fn compute_image_hash(image_bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(image_bytes);
    format!("{:x}", hasher.finalize())
}
```

### 2. Integrate Tesseract OCR (High Priority)
**Why**: Foundation for Phase 1 baseline text extraction

**Tasks**:
- [ ] Add `leptess` dependency to `core_pipeline`
- [ ] Implement `extract_text_tesseract()` in `core_pipeline/src/ocr.rs`
- [ ] Configure Tesseract for IBM 1130 character set
- [ ] Handle fixed-width column layouts
- [ ] Add error handling for missing Tesseract installation
- [ ] Write tests with synthetic punch card images

**Crates to use**:
```toml
leptess = "0.14"
```

**Documentation needed**:
- Add Tesseract installation to README.md prerequisites
- Document character set configuration

### 3. Implement CLI Ingest Command (High Priority)
**Why**: Entry point for Phase 1: Scan

**Tasks**:
- [ ] Implement file/directory traversal in CLI
- [ ] Support multiple image formats (JPEG, PNG, TIFF, PDF)
- [ ] Create scan set directory structure
- [ ] Generate unique ScanSetId
- [ ] Copy images with deduplication
- [ ] Write CIR JSON files
- [ ] Add progress reporting
- [ ] Write integration tests

**Directory structure**:
```
scan_set_001/
+-- manifest.json          # ScanSetId, metadata
+-- images/
    +-- <hash>.jpg         # Original images (deduplicated)
+-- processed/             # Preprocessed images (created by analyze)
+-- artifacts.json         # PageArtifact and CardArtifact records
```

### 4. Basic Image Preprocessing (Medium Priority)
**Why**: Improve OCR accuracy

**Tasks**:
- [ ] Implement grayscale conversion (already done)
- [ ] Add contrast stretching/histogram equalization
- [ ] Implement adaptive thresholding (Otsu's method)
- [ ] Add morphological operations (opening/closing)
- [ ] Implement basic deskewing (Hough transform)
- [ ] Write unit tests with synthetic images
- [ ] Benchmark performance

**Crates to use**:
```toml
imageproc = { workspace = true }  # Already added
opencv = { version = "0.92", optional = true }  # For advanced ops
```

## Short-Term Goals (1-2 Weeks)

### 5. CLI Analyze Command
**Tasks**:
- [ ] Load scan set from disk
- [ ] Iterate through images
- [ ] Run preprocessing pipeline
- [ ] Run Tesseract OCR
- [ ] Classify artifacts (basic heuristics)
- [ ] Update CIR with results
- [ ] Save artifacts.json
- [ ] Add --use-llm flag (placeholder for Phase 2)

### 6. CLI Export Command
**Tasks**:
- [ ] Load analyzed scan set
- [ ] Generate `EmulatorOutput::CardDeck` format
- [ ] Generate `EmulatorOutput::Listing` format
- [ ] Write JSON output files
- [ ] Validate output against schema
- [ ] Add export format selection

### 7. Basic Web UI (Yew)
**Tasks**:
- [ ] Implement file upload component (already started)
- [ ] Add drag-and-drop support
- [ ] Display uploaded files list
- [ ] Show image previews
- [ ] Add processing progress indicator
- [ ] Display results (text extraction)
- [ ] Implement standalone mode (process in WASM)

### 8. Testing Infrastructure
**Tasks**:
- [ ] Create synthetic test images (punch cards, listings)
- [ ] Generate test card decks with known content
- [ ] Create test listings with various formats
- [ ] Add property-based tests (round-trip validation)
- [ ] Set up test data fixtures
- [ ] Document test data generation

## Medium-Term Goals (1 Month)

### 9. IBM 1130 Object Deck Decoder
**Tasks**:
- [ ] Research IBM 1130 object deck format
- [ ] Implement card type detection
- [ ] Parse compressed label columns
- [ ] Extract address fields
- [ ] Extract binary data
- [ ] Implement checksum validation
- [ ] Write comprehensive tests

**Resources needed**:
- IBM 1130 documentation
- Example object decks
- Emulator reference implementations

### 10. REST API Implementation
**Tasks**:
- [ ] Implement scan set creation endpoint
- [ ] Add file upload handler (multipart/form-data)
- [ ] Create job queue system
- [ ] Add WebSocket support for progress updates
- [ ] Implement artifact retrieval endpoints
- [ ] Add export endpoint
- [ ] Write API integration tests

### 11. Server Mode Integration
**Tasks**:
- [ ] Connect server to core_pipeline
- [ ] Implement async processing
- [ ] Add job persistence (SQLite or sled)
- [ ] Implement static file serving for frontend
- [ ] Add CORS configuration
- [ ] Write end-to-end tests

## Phase 2: LLM Integration (1-2 Months)

### 12. Ollama Setup and Testing
**Tasks**:
- [ ] Document Ollama installation
- [ ] Test Qwen2.5-VL 7B model
- [ ] Test Phi-3.5-Vision model
- [ ] Test text models (Qwen2.5 3B, Phi-4)
- [ ] Create prompt templates
- [ ] Implement response parsing
- [ ] Add LLM result caching

### 13. Vision Model Integration
**Tasks**:
- [ ] Implement image classification (already scaffolded)
- [ ] Add card vs listing detection
- [ ] Implement text vs binary card detection
- [ ] Extract page numbers from headers
- [ ] Parse metadata from images
- [ ] Cross-validate with OCR results
- [ ] Add confidence scoring

### 14. Text Model Integration
**Tasks**:
- [ ] Implement language detection (ASM, FORTRAN, Forth)
- [ ] Add OCR refinement
- [ ] Implement ordering suggestions
- [ ] Detect gaps and missing sections
- [ ] Generate reconstruction proposals
- [ ] Add human-in-the-loop approval

### 15. Advanced UI Features
**Tasks**:
- [ ] Implement page/card ordering (drag-drop)
- [ ] Add manual classification correction
- [ ] Show LLM confidence scores
- [ ] Highlight uncertain regions
- [ ] Implement split/merge operations
- [ ] Add annotation tools

## Phase 3: Advanced Features (3+ Months)

### 16. IBM 1130 Disassembler
**Tasks**:
- [ ] Research IBM 1130 instruction set
- [ ] Implement opcode decoder
- [ ] Add operand formatting
- [ ] Generate labels for branch targets
- [ ] Create symbol table
- [ ] Add comments/annotations
- [ ] Write disassembler tests

### 17. Reverse Engineering
**Tasks**:
- [ ] Implement control flow analysis
- [ ] Detect subroutine boundaries
- [ ] Identify data vs code sections
- [ ] Generate call graphs
- [ ] Propose source reconstructions
- [ ] Add Forth-specific analysis

### 18. Document Enhancement
**Tasks**:
- [ ] Research super-resolution models
- [ ] Implement Python sidecar (if needed)
- [ ] Add denoising pipeline
- [ ] Test with low-quality scans
- [ ] Compare original vs enhanced
- [ ] Add manual review step

## Infrastructure and DevOps

### 19. CI/CD Pipeline
**Tasks**:
- [ ] Set up GitHub Actions workflow
- [ ] Add automated testing on push
- [ ] Add clippy and fmt checks
- [ ] Add markdown validation
- [ ] Set up automated releases
- [ ] Add code coverage reporting

### 20. Documentation
**Tasks**:
- [ ] Create user guide
- [ ] Add CLI command examples
- [ ] Document API endpoints
- [ ] Create video tutorials
- [ ] Add troubleshooting guide
- [ ] Document deployment options

### 21. Performance Optimization
**Tasks**:
- [ ] Add benchmarking suite
- [ ] Profile image processing
- [ ] Optimize WASM bundle size
- [ ] Implement parallel processing (rayon)
- [ ] Add result caching
- [ ] Optimize memory usage

## Research and Exploration

### 22. Custom Models
**Tasks**:
- [ ] Collect training data (IBM 1130 specific)
- [ ] Fine-tune Phi-3.5-Vision
- [ ] Train custom character recognition
- [ ] Evaluate model performance
- [ ] Document model training process

### 23. Additional Export Formats
**Tasks**:
- [ ] Research other emulator formats
- [ ] Add binary disk image export
- [ ] Support tape image formats
- [ ] Add assembler source output
- [ ] Create format documentation

## Community and Collaboration

### 24. Open Source Preparation
**Tasks**:
- [ ] Write CONTRIBUTING.md
- [ ] Add CODE_OF_CONDUCT.md
- [ ] Create issue templates
- [ ] Add pull request template
- [ ] Document project governance
- [ ] Set up Discord/discussion forum

### 25. Example Projects
**Tasks**:
- [ ] Process Chuck Moore's Forth scans
- [ ] Document reconstruction process
- [ ] Create before/after comparisons
- [ ] Share results with IBM 1130 community
- [ ] Gather feedback for improvements

## Current State Summary

**Completed**:
- [x] Multi-crate workspace architecture
- [x] Core types and CIR definition
- [x] LLM bridge scaffolding (Ollama client)
- [x] CLI structure with command definitions
- [x] REST API server scaffolding
- [x] Yew frontend basic structure
- [x] Build and serve scripts
- [x] Comprehensive documentation
- [x] MIT license and copyright
- [x] Git repository initialized
- [x] Duplicate detection implementation (SHA-256 based)

**In Progress**:
- None (ready to continue Phase 1 implementation)

**Blocked**:
- None (no blockers)

## Recommended Starting Point

**Start here**: Implement duplicate detection (#1) and Tesseract integration (#2) in parallel, then move to CLI ingest command (#3). This gives you a working baseline for the "Scan" phase.

**Test with**: Use test-data directory to validate deduplication and OCR accuracy.

**Success criteria**:
```bash
scan3data ingest -i ./test-data -o ./test_scan_set
# Should create scan set with deduplicated images and metadata

scan3data analyze -s ./test_scan_set
# Should run OCR and classify artifacts

scan3data export -s ./test_scan_set -o output.json
# Should generate emulator-ready JSON
```

Once this baseline works, you have a solid foundation for Phase 2 LLM integration.
