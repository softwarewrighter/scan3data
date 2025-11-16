# OCR Correction and Improvement Plan

## Problem Statement

Current Tesseract OCR extraction has several issues:
1. **Missing leading whitespace** - Critical for punch card column alignment
2. **Character corruption** - OCR errors like "Be" vs "DC", "oc" vs "DC"
3. **No layout preservation** - Can't distinguish column positions
4. **No context awareness** - Doesn't know IBM 1130 opcodes/syntax

## Three-Track Solution Strategy

### Track 1: Enhanced OCR (Immediate - Tesseract Improvements)

**Goal**: Preserve layout and improve baseline OCR quality

**Steps**:
1. **Enable Tesseract PSM (Page Segmentation Mode)**
   - Use PSM 6 (uniform block of text) or PSM 4 (single column)
   - Preserves spatial layout and leading whitespace
   - Code change in `extract_text_tesseract()`:
     ```rust
     tesseract.set_variable("tessedit_pageseg_mode", "6")?;
     ```

2. **Use hOCR output format**
   - Captures bounding boxes and confidence per word
   - Preserves exact column positions
   - Can reconstruct punch card columns (1-80)
   - API: `tesseract.get_hocr_text()`

3. **Character whitelist for IBM 1130**
   - Restrict to valid punch card characters:
     - A-Z, 0-9, special chars: `+-*/=().,;:$#@'&`
     - No lowercase (punch cards are uppercase)
   - Code: `tesseract.set_variable("tessedit_char_whitelist", "ABC...")?;`

4. **Increase DPI for preprocessing**
   - Resample images to 300 DPI before OCR
   - Better recognition of small/dense text

**Implementation Priority**: HIGH (1-2 days)

---

### Track 2: Reference Database (Medium-term - Context Building)

**Goal**: Build corpus of known-good IBM 1130 code for correlation

**Data Sources**:

1. **Bitsavers Archive** (http://bitsavers.org/bits/IBM/1130/)
   - Known .job files, assembler listings, Forth decks
   - Download entire IBM 1130 collection
   - Parse and index by content type

2. **IBM 1130 Archives**:
   - http://ibm1130.org/ - Community archives
   - GitHub repos with 1130 code
   - Academic/museum collections

3. **Structure Reference Database**:
   ```
   reference_db/
   ├── card_decks/
   │   ├── forth/
   │   │   ├── chuck_moore_forth.txt
   │   │   ├── metadata.json (source, date, format)
   │   ├── assembler/
   │   ├── fortran/
   ├── listings/
   │   ├── object_code/
   │   ├── source/
   └── index/
       ├── opcode_db.json      # All known 1130 opcodes
       ├── forth_words.json    # All known Forth words
       ├── ngrams.json         # Common code patterns
   ```

4. **Build Similarity Search**:
   - Use embedding models (sentence-transformers)
   - Index all reference decks
   - For each OCR artifact, find top-K similar known decks
   - Use as context for correction

**Implementation Priority**: MEDIUM (1 week)

---

### Track 3: LLM/Diffusion Correction (Advanced - AI Enhancement)

**Goal**: Use AI models to correct OCR errors using context

**Approach A: GPT-4 Vision + Text (Recommended)**

**Workflow**:
1. **Two-pass correction**:
   - **Pass 1 - Vision**: Send preprocessed image + OCR text to GPT-4V
   - **Pass 2 - Text**: Use text-only GPT-4 with reference context

2. **Pass 1: GPT-4 Vision Prompt**:
   ```
   You are an expert in IBM 1130 punch card OCR correction.

   Image: [preprocessed card scan]
   OCR Text: [tesseract output with layout]

   Task: Correct OCR errors while preserving exact column alignment.

   Context:
   - This is an IBM 1130 punch card (80 columns)
   - Valid characters: A-Z 0-9 +-*/=().,;:$#@'&
   - Common opcodes: LDX, STX, BSI, MDX, ADD, SUB, etc.
   - Preserve all leading/trailing whitespace

   Output format:
   {
     "corrected_text": "...",
     "corrections": [
       {"position": "col 12", "original": "Be", "corrected": "DC", "confidence": 0.95}
     ],
     "card_type": "object_deck" | "source_text" | "data"
   }
   ```

3. **Pass 2: GPT-4 Text with Reference Context**:
   ```
   You are correcting IBM 1130 Forth code OCR errors.

   OCR Text (with errors):
   [pass 1 output]

   Reference Context (top-3 similar known decks):
   [retrieved from reference DB]

   Known Forth Words:
   [from forth_words.json]

   Task: Final correction using reference context.
   Preserve exact formatting and column positions.
   ```

**Approach B: Text Diffusion Model (Alternative)**

**Use Case**: For purely text-based correction (no vision)

1. **Model**: Use fine-tuned diffusion model for text correction
   - Base: Stable Diffusion for text (e.g., TextDiffusion)
   - Fine-tune on: Clean IBM 1130 code + synthetic OCR errors

2. **Training Data Generation**:
   - Take reference decks
   - Synthetically add OCR errors (simulate corruption)
   - Train model to denoise back to clean text

3. **Limitations**:
   - Doesn't see original image
   - Relies purely on learned patterns
   - May hallucinate if no strong reference match

**Implementation Priority**: HIGH for GPT-4V, LOW for diffusion

---

## Recommended Implementation Roadmap

### Phase 1: Quick Wins (Week 1)
- [ ] Fix Tesseract PSM mode to preserve whitespace
- [ ] Add character whitelist for IBM 1130
- [ ] Output hOCR format with bounding boxes
- [ ] Re-run analyze on test_scan_set
- [ ] Validate column alignment in text-dump

### Phase 2: Reference Database (Week 2)
- [ ] Scrape bitsavers.org IBM 1130 collection
- [ ] Build opcode/Forth word reference databases
- [ ] Implement embedding-based similarity search
- [ ] Add `--use-reference` flag to analyze command

### Phase 3: GPT-4 Vision Correction (Week 3)
- [ ] Integrate OpenAI API (or local Ollama with vision model)
- [ ] Implement two-pass correction workflow
- [ ] Add `--correct-with-llm` flag to analyze
- [ ] Store corrections in artifact metadata
- [ ] Generate before/after diff reports

### Phase 4: Validation & Iteration (Week 4)
- [ ] Human review of corrected output
- [ ] Calculate correction accuracy metrics
- [ ] Refine prompts based on errors
- [ ] Build correction confidence scoring

---

## Technical Architecture

### Enhanced OCR Module (core_pipeline/src/ocr.rs)

```rust
pub struct OcrConfig {
    pub preserve_layout: bool,
    pub page_segmentation_mode: u8,  // 4 or 6
    pub char_whitelist: Option<String>,
    pub output_format: OcrOutputFormat,
}

pub enum OcrOutputFormat {
    PlainText,
    Hocr,  // HTML with bounding boxes
    Tsv,   // Tab-separated values with coordinates
}

pub struct OcrResult {
    pub text: String,
    pub layout: Option<LayoutInfo>,
    pub confidence: f32,
    pub words: Vec<OcrWord>,
}

pub struct OcrWord {
    pub text: String,
    pub bbox: BoundingBox,
    pub confidence: f32,
    pub column: u8,  // For punch cards: 1-80
}
```

### Reference Database Module (new: core_pipeline/src/reference.rs)

```rust
pub struct ReferenceDatabase {
    pub card_decks: Vec<ReferenceDeck>,
    pub opcodes: HashMap<String, OpcodeInfo>,
    pub forth_words: HashSet<String>,
    pub embeddings: Vec<(String, Vec<f32>)>,
}

pub struct ReferenceDeck {
    pub id: String,
    pub source: String,
    pub content: String,
    pub metadata: DeckMetadata,
    pub deck_type: DeckType,
}

pub fn find_similar_decks(query: &str, k: usize) -> Vec<ReferenceDeck>;
```

### LLM Correction Module (new: llm_bridge/src/correction.rs)

```rust
pub struct CorrectionRequest {
    pub image: Option<DynamicImage>,
    pub ocr_text: String,
    pub reference_context: Vec<ReferenceDeck>,
    pub config: CorrectionConfig,
}

pub struct CorrectionResult {
    pub corrected_text: String,
    pub corrections: Vec<Correction>,
    pub confidence: f32,
    pub card_type: ArtifactKind,
}

pub struct Correction {
    pub position: String,
    pub original: String,
    pub corrected: String,
    pub confidence: f32,
    pub reasoning: String,
}

pub async fn correct_with_gpt4v(req: CorrectionRequest) -> Result<CorrectionResult>;
pub async fn correct_with_ollama(req: CorrectionRequest) -> Result<CorrectionResult>;
```

---

## Data Flow

```
┌─────────────────┐
│ Scanned Image   │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Preprocess      │  (grayscale, deskew, threshold)
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Enhanced OCR    │  (Tesseract + PSM + whitelist + hOCR)
│ - Preserves     │
│   whitespace    │
│ - Column pos    │
│ - Confidence    │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Reference DB    │  Find similar known decks
│ Similarity      │  (embedding search)
│ Search          │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ LLM Correction  │
│ (GPT-4V or      │  Two-pass correction
│  Ollama Qwen2VL)│  - Vision pass
│                 │  - Text pass w/ context
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Corrected Text  │
│ + Metadata      │  Store corrections, confidence
│ + Diff Report   │  Human review
└─────────────────┘
```

---

## Example: Before vs After

### Current OCR Output (broken):
```
cy PAGE 10
OBFO-0-—0078 Be 123-——-NUMBER ;
OBF1 0 004C oc 76 LESS THAN
```

### Target Output (corrected with layout):
```
                                                              PAGE 10
OBF0    0       0078    DC      123             *NUMBER
OBF1    0       004C    DC      76              *LESS THAN
```

### Correction Metadata:
```json
{
  "corrections": [
    {"position": "col 1-4", "original": "OBFO", "corrected": "OBF0", "confidence": 0.98},
    {"position": "col 21-22", "original": "Be", "corrected": "DC", "confidence": 0.95},
    {"position": "col 33-40", "original": "123-——-NUMBER", "corrected": "*NUMBER", "confidence": 0.92}
  ],
  "card_type": "ListingObject",
  "reference_match": "chuck_moore_forth.txt (similarity: 0.87)"
}
```

---

## Cost Estimates

### GPT-4 Vision API
- Cost: ~$0.01 per image (1024x1024)
- 16 artifacts = $0.16
- Full production (1000s cards) = $10-50

### Ollama (Local - Free)
- Models: Qwen2.5-VL 7B, Phi-3.5-Vision
- Hardware: Mac with 16GB+ RAM
- Cost: $0 (local inference)
- Speed: 2-5 sec per card

### Recommendation
Start with Ollama for cost-free iteration, then optionally use GPT-4V for final high-accuracy pass.

---

## Success Metrics

1. **Column Alignment**: 100% of cards preserve exact 80-column layout
2. **OCR Accuracy**: >95% character accuracy (measured against reference)
3. **Correction Accuracy**: >90% of LLM corrections are valid
4. **Coverage**: >80% of cards match to known reference decks
5. **Human Review**: <10% of cards need manual correction

---

## Next Immediate Action

**Implement Phase 1 - Enhanced Tesseract OCR** (1-2 days):
1. Add PSM mode configuration
2. Enable hOCR output
3. Add IBM 1130 character whitelist
4. Re-run analyze on test data
5. Validate whitespace preservation

This will immediately fix the leading whitespace issue and give you properly formatted output for manual review.
