# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**scan3data** processes scanned images of IBM 1130 punch cards and computer listings, converting them into structured data for emulator consumption.

### Why "scan3data"?

The **3** represents our **three-phase processing pipeline**:

1. **Scan** - Ingest and digitize (image acquisition, duplicate detection, preprocessing)
2. **Classify & Correct** - Analyze and refine (OCR, LLM classification, ordering, gap detection)
3. **Convert** - Transform to structured output (emulator formats, reconstruction, export)

This isn't just a simple scanner - it's a complete three-stage transformation pipeline from messy historical scans to pristine emulator-ready data.

### Architecture

This is a **multi-crate Cargo workspace** with 5 crates:

1. **core_pipeline** - Core types and processing logic (no networking)
   - Canonical Intermediate Representation (CIR)
   - Image preprocessing (deskew, threshold, noise removal)
   - OCR integration (Tesseract via leptess)
   - IBM 1130 object deck decoder and disassembler

2. **llm_bridge** - Ollama LLM integration (Phase 2)
   - Vision models (Qwen2.5-VL, Phi-Vision) for image classification
   - Text models (Qwen2.5, Phi-4) for refinement and ordering
   - HTTP client for Ollama API

3. **cli** - Command-line interface (binary: scan3data)
   - Commands: ingest, analyze, export, serve (three-phase pipeline)
   - Batch processing
   - Can serve either SPA or API mode

4. **server** - REST API backend (binary: scan3data-server)
   - Axum-based HTTP server
   - Endpoints for scan set management, uploads, processing
   - Job queue and status tracking

5. **yew_frontend** - Browser UI (compiled to WASM)
   - File upload interface
   - Page/card ordering (drag-drop)
   - Reconstruction visualization
   - Two modes: standalone SPA or API client

### Rust-First Philosophy

**CRITICAL**: This project is Rust-focused. ALL business logic and presentation must be in Rust.

- Frontend: Yew (Rust -> WASM), NOT JavaScript frameworks
- Backend: Axum (Rust), NOT Node.js/Python
- Image processing: Rust crates, NOT Python libraries
- Only acceptable non-Rust: minimal HTML/CSS, calling external binaries when unavoidable

### Duplicate Detection

The `./test-data` directory contains many duplicate scans with different filenames.

**Requirement**: During ingest, detect duplicates and combine them:
1. Compute SHA-256 hash of image contents
2. Store only ONE copy of duplicate images
3. Store ALL filenames in metadata array
4. Use filenames as context hints for LLMs

Example: `forth-p1.jpg` + `moore-1130-page1.jpg` both hash to same value:
- Store one image
- Metadata: `original_filenames: ["forth-p1.jpg", "moore-1130-page1.jpg"]`
- LLM gets both names as context

## Build and Test Commands

### Build All
```bash
# Build entire workspace (all crates)
./scripts/build-all.sh

# Or manually:
cargo build --workspace --exclude yew_frontend  # Rust crates
./scripts/build-wasm.sh                          # WASM frontend
```

### Build Individual Components
```bash
# CLI only (faster)
./scripts/build-cli.sh
# Output: target/release/scan3data

# Server only
./scripts/build-server.sh
# Output: target/release/scan3data-server

# WASM frontend only
./scripts/build-wasm.sh
# Output: dist/
```

### Development
```bash
# Watch and auto-rebuild WASM (with live reload)
./scripts/dev-wasm.sh

# Build single crate during development
cargo build -p core_pipeline
cargo build -p llm_bridge
cargo build -p scan3data-cli
cargo build -p scan3data-server
```

### Testing
```bash
# Run all tests
cargo test --workspace

# Run tests for specific crate
cargo test -p core_pipeline
cargo test -p llm_bridge

# Run specific test
cargo test test_name

# Run tests with output visible
cargo test -- --nocapture
```

### Linting and Formatting
```bash
# Lint (strict mode - zero warnings allowed)
cargo clippy --all-targets --all-features -- -D warnings

# Format code
cargo fmt --all

# Check formatting without modifying
cargo fmt --check

# Generate and view documentation
cargo doc --open --workspace
```

### Serving

```bash
# SPA mode (standalone, all processing in browser)
./scripts/serve-spa.sh [PORT]
# Default: http://localhost:8080

# API mode (backend + frontend)
./scripts/serve-api.sh
# Backend: http://localhost:3000

# Or use CLI directly
cargo run -p scan3data-cli -- serve --mode spa --port 8080
cargo run -p scan3data-cli -- serve --mode api --port 8080
```

## Mandatory Pre-Commit Process

**CRITICAL**: Before every commit, complete ALL steps in order. No exceptions, no deferrals, no skipping.

### Step 1: Run Tests
```bash
cargo test
```
- ALL tests must pass
- Fix failing tests immediately, never disable or skip

### Step 2: Fix Linting (Zero Warnings)
```bash
cargo clippy --all-targets --all-features -- -D warnings
```
- ZERO warnings allowed
- Fix all warnings, never use `#[allow(...)]` to suppress
- Re-run until completely clean

### Step 3: Format Code
```bash
cargo fmt --all
```
- Verify with `cargo fmt --check`

### Step 4: Validate Markdown (if docs changed)
```bash
markdown-checker -f "**/*.md"
```
- Ensure ASCII-only characters (no Unicode)
- Use `--fix` for auto-fixable issues (tree symbols)
- Manually fix emojis and arrows

### Step 5: Update Documentation
If any issues were found in previous steps:
- Update `docs/learnings.md` with root cause analysis
- Update README.md if features changed
- Update CLAUDE.md if development patterns changed

## Architecture Guidelines

### Test-Driven Development (TDD)

Follow strict **Red/Green/Refactor** cycle:

1. **RED**: Write a failing test that defines desired behavior
2. **GREEN**: Write minimal code to make the test pass
3. **REFACTOR**: Improve code while keeping tests green
4. **REPEAT**: Continue for next piece of functionality

### Code Quality Standards

**Rust 2024 Edition Idioms**:
- Inline format arguments: `format!("{name}")` not `format!("{}", name)`
- Inner doc comments for modules: `//!` not `///` + empty line
- Use `let-else` patterns for error handling where appropriate

**File Size Limits**:
- Keep source files under 500 lines (prefer 200-300)
- Split large files into logical modules
- Refactor immediately if exceeding 500 lines

**Function Complexity**:
- Functions under 50 lines (prefer 10-30)
- Cyclomatic complexity below 10
- Deeply nested code (>3 levels) needs refactoring

**TODO Management**:
- Maximum 3 TODO comments per file
- Address TODOs within 2 development sessions
- Convert persistent TODOs to GitHub issues
- Never commit FIXMEs - resolve immediately

**Dependencies**:
- Audit regularly, remove unused ones
- Prefer well-maintained, popular crates
- Minimize dependency tree depth

### Documentation Requirements

- Public APIs must have `///` documentation comments
- Module-level docs use `//!` comments
- Include examples in doc comments where helpful
- Keep README.md up-to-date

## Development Tools

Tools are located in `~/.local/softwarewrighter/bin/`:

### proact - AI Agent Documentation Generator
```bash
# Generate AI agent documentation
proact .

# Regenerate if project structure changes
proact . --verbose
```
Generates comprehensive AI coding agent guidelines.

### markdown-checker - Markdown Validator
```bash
# Validate all markdown (REQUIRED before commit)
markdown-checker -f "**/*.md"

# Auto-fix tree symbols
markdown-checker --fix

# Verbose output
markdown-checker -v
```
Ensures ASCII-only markdown for portability.

### ask - Command-Line LLM Query
```bash
# Quick queries (uses Ollama by default)
ask "How do I parse JSON in Rust?"

# Use specific model
ask -s sonnet "Explain async/await"
```

All tools support `--help` for detailed usage.

## Git Workflow

### Commit Message Format
```
type: Short summary (50 chars max)

Detailed explanation of what changed and why.
Include context, rationale, and trade-offs.

If bugs were fixed:
- Root cause: <why the bug occurred>
- Prevention: <what process change prevents this>
- Updated learnings.md with <specific section>

[AI] Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>
```

**Types**: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

### Branch and Push Policy
- Currently: Direct commits to `main`
- Always push immediately after commit
- Never force push to main
- Never skip pre-commit checks

## Continuous Improvement

### Update docs/learnings.md When:
- Clippy warnings found (document the pattern)
- Tests failed (document root cause)
- Bug fixed (document prevention strategy)

### Root Cause Analysis Required:
1. What went wrong?
2. Why wasn't it caught sooner?
3. What process change prevents this?
4. Update proactive checklist if needed

## Scripting Guidelines

**Use Bash for**:
- Simple automation (< 100 lines)
- Always start with `set -euo pipefail`
- Quote all variables: `"${var}"` not `$var`
- Validate with shellcheck

**Switch to Python when**:
- Script exceeds 100 lines
- Complex data structures needed
- JSON/YAML parsing required
- Cross-platform compatibility needed
- Error handling becomes complex

**Python Scripts**:
- Use type hints even in scripts
- Include `#!/usr/bin/env python3` shebang
- Use `argparse` for CLI arguments
- Keep focused - one purpose per script

## Project-Specific Notes

This project was generated with comprehensive AI agent documentation in `docs/`:
- `docs/ai_agent_instructions.md` - Full development guidelines
- `docs/process.md` - Detailed development workflow (TDD, pre-commit process)
- `docs/tools.md` - Tool documentation and usage

Refer to these documents for expanded guidance on:
- Checkpoint workflows
- Quality gates
- Testing strategies
- Playwright MCP setup
- WASM development (if applicable)
- Continuous improvement processes
