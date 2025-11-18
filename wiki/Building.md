# Building scan3data

Complete build instructions for all components of the scan3data project.

## Prerequisites

### Rust Toolchain

```bash
# Install Rust (latest stable)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add WASM target for frontend
rustup target add wasm32-unknown-unknown

# Verify installation
rustc --version  # Should be 1.75.0 or newer
```

### WASM Build Tools

```bash
# Install Trunk (WASM bundler)
cargo install trunk

# Verify installation
trunk --version
```

### System Dependencies

#### macOS

```bash
# Tesseract OCR and dependencies
brew install tesseract pkgconf

# Optional: Ollama for local LLM
brew install ollama
```

#### Linux (Debian/Ubuntu)

```bash
# Tesseract OCR and dependencies
sudo apt-get update
sudo apt-get install tesseract-ocr pkg-config libleptonica-dev libtesseract-dev

# Build essentials
sudo apt-get install build-essential

# Optional: Ollama
curl https://ollama.ai/install.sh | sh
```

#### Linux (Fedora/RHEL)

```bash
# Tesseract OCR and dependencies
sudo dnf install tesseract tesseract-devel leptonica-devel pkgconf

# Build essentials
sudo dnf install gcc gcc-c++ make
```

## Quick Build (All Components)

```bash
# Build everything at once
./scripts/build-all.sh
```

**Outputs:**
- `target/release/scan3data` - CLI binary
- `target/release/scan3data-server` - Server binary
- `dist/` - WASM frontend (static files)

## Component-Specific Builds

### CLI Only

```bash
./scripts/build-cli.sh

# Or manually:
cargo build -p scan3data-cli --release
```

**Output:** `target/release/scan3data`

**Test:**
```bash
./target/release/scan3data --version
```

### Server Only

```bash
./scripts/build-server.sh

# Or manually:
cargo build -p scan3data-server --release
```

**Output:** `target/release/scan3data-server`

**Test:**
```bash
./target/release/scan3data-server &
curl http://localhost:7214/health
```

### Frontend Only (WASM)

```bash
./scripts/build-wasm.sh

# Or manually:
cd crates/yew_frontend
trunk build --release
```

**Output:** `dist/` directory with:
- `index.html`
- `scan3data-*.wasm`
- `scan3data-*.js`

**Test:**
```bash
cd dist
python3 -m http.server 8080
# Open http://localhost:8080
```

### Library Crates

```bash
# Build core_pipeline
cargo build -p core_pipeline --release

# Build llm_bridge
cargo build -p llm_bridge --release
```

**Note:** Library crates don't produce binaries, only `.rlib` files.

## Development Builds

### Fast Iteration (Debug Mode)

```bash
# Build without optimizations (faster compile)
cargo build --workspace --exclude yew_frontend

# Build specific crate
cargo build -p scan3data-cli
```

**Debug binaries:** `target/debug/scan3data`

### Watch Mode (Auto-Rebuild)

#### WASM Frontend

```bash
./scripts/dev-wasm.sh

# Or manually:
cd crates/yew_frontend
trunk serve --port 8080
```

**Features:**
- Auto-rebuild on file changes
- Live reload in browser
- Hot module replacement

#### Rust Code (cargo-watch)

```bash
# Install cargo-watch
cargo install cargo-watch

# Watch and rebuild CLI
cargo watch -x 'build -p scan3data-cli'

# Watch and run tests
cargo watch -x 'test -p core_pipeline'
```

## Build Scripts

Located in `./scripts/` directory:

### build-all.sh

Builds all components (CLI, server, frontend).

```bash
#!/usr/bin/env bash
set -euo pipefail

echo "Building all components..."

# Build Rust crates (excluding WASM)
cargo build --workspace --exclude yew_frontend --release

# Build WASM frontend
./scripts/build-wasm.sh

echo "Build complete!"
echo "  CLI: target/release/scan3data"
echo "  Server: target/release/scan3data-server"
echo "  Frontend: dist/"
```

### build-cli.sh

```bash
#!/usr/bin/env bash
set -euo pipefail

cargo build -p scan3data-cli --release
echo "CLI built: target/release/scan3data"
```

### build-server.sh

```bash
#!/usr/bin/env bash
set -euo pipefail

cargo build -p scan3data-server --release
echo "Server built: target/release/scan3data-server"
```

### build-wasm.sh

```bash
#!/usr/bin/env bash
set -euo pipefail

cd crates/yew_frontend
trunk build --release
echo "Frontend built: dist/"
```

### dev-wasm.sh

```bash
#!/usr/bin/env bash
set -euo pipefail

cd crates/yew_frontend
trunk serve --port 8080
```

## Build Configuration

### Cargo.toml (Workspace)

```toml
[workspace]
members = [
    "crates/core_pipeline",
    "crates/llm_bridge",
    "crates/cli",
    "crates/server",
    "crates/yew_frontend",
]

[workspace.package]
version = "0.1.0"
edition = "2021"

[profile.release]
opt-level = 3
lto = true          # Link-Time Optimization
codegen-units = 1   # Better optimization, slower compile
strip = true        # Strip symbols for smaller binaries
```

### Trunk.toml (WASM)

```toml
[build]
target = "index.html"
dist = "dist"

[watch]
ignore = ["dist"]

[serve]
port = 8080
address = "127.0.0.1"
```

## Optimization

### Release Build Flags

```bash
# Full optimizations (slow compile, fast runtime)
cargo build --release

# Size-optimized build
cargo build --profile release-small

# Custom profile (in Cargo.toml):
[profile.release-small]
inherits = "release"
opt-level = "z"      # Optimize for size
lto = true
codegen-units = 1
strip = true
```

### WASM Optimization

```bash
# Install wasm-opt
cargo install wasm-opt

# Optimize WASM bundle
wasm-opt -Oz -o dist/optimized.wasm dist/scan3data-*.wasm
```

**Size reduction:** ~30-40%

## Troubleshooting

### Tesseract Not Found

**Error:**
```
error: linking with `cc` failed: exit status: 1
note: ld: library not found for -ltesseract
```

**Solution (macOS):**
```bash
brew install tesseract pkgconf
export PKG_CONFIG_PATH="/usr/local/lib/pkgconfig"
```

**Solution (Linux):**
```bash
sudo apt-get install libtesseract-dev libleptonica-dev
```

### WASM Target Not Installed

**Error:**
```
error: target 'wasm32-unknown-unknown' not found
```

**Solution:**
```bash
rustup target add wasm32-unknown-unknown
```

### Trunk Not Found

**Error:**
```
trunk: command not found
```

**Solution:**
```bash
cargo install trunk
```

### Out of Memory During Build

**Solution:**
```bash
# Reduce parallel jobs
cargo build --release -j 2

# Or set environment variable
export CARGO_BUILD_JOBS=2
cargo build --release
```

### Slow Compile Times

**Solutions:**

1. **Use sccache (caching compiler)**
```bash
cargo install sccache
export RUSTC_WRAPPER=sccache
```

2. **Incremental compilation (debug builds)**
```bash
export CARGO_INCREMENTAL=1
```

3. **Link with mold (faster linker)**
```bash
cargo install mold
export RUSTFLAGS="-C link-arg=-fuse-ld=mold"
```

## Build Artifacts

### Directory Structure

```
scan3data/
├── target/
│   ├── debug/              # Debug builds
│   │   ├── scan3data
│   │   └── scan3data-server
│   ├── release/            # Release builds
│   │   ├── scan3data
│   │   └── scan3data-server
│   └── wasm32-unknown-unknown/
│       └── release/        # WASM builds
├── dist/                   # Trunk output (WASM frontend)
│   ├── index.html
│   ├── scan3data-*.wasm
│   └── scan3data-*.js
└── crates/                 # Source code
```

### Clean Builds

```bash
# Clean all build artifacts
cargo clean

# Clean only release builds
cargo clean --release

# Clean WASM builds
rm -rf dist/
```

## Continuous Integration

### GitHub Actions (Example)

```yaml
name: Build

on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Install dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y tesseract-ocr libtesseract-dev
      - name: Build
        run: cargo build --workspace --exclude yew_frontend --release
      - name: Test
        run: cargo test --workspace
```

## Related Pages

- [Testing](Testing) - Running tests after building
- [CLI](CLI) - Using the built CLI binary
- [REST API](REST-API) - Running the server binary

---

**Last Updated:** 2025-11-16
