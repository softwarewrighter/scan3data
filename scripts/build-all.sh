#!/bin/bash
set -euo pipefail

# Build all components of scan2data

echo "=== Building scan2data workspace ==="

# Build Rust workspace (all crates except WASM)
echo "Building Rust workspace..."
cargo build --workspace --exclude yew_frontend

# Build WASM frontend
echo "Building WASM frontend..."
./scripts/build-wasm.sh

echo "=== Build complete ==="
echo "CLI binary: target/debug/scan2data"
echo "Server binary: target/debug/scan2data-server"
echo "WASM output: dist/"
