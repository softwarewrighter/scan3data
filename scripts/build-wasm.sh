#!/bin/bash
set -euo pipefail

# Build Yew frontend to WASM

echo "=== Building Yew frontend ==="

cd crates/yew_frontend

# Check if trunk is installed
if ! command -v trunk &> /dev/null; then
    echo "trunk is not installed. Installing..."
    cargo install trunk
fi

# Check if wasm target is installed
if ! rustup target list | grep -q "wasm32-unknown-unknown (installed)"; then
    echo "Installing wasm32-unknown-unknown target..."
    rustup target add wasm32-unknown-unknown
fi

# Build with trunk
trunk build --release

# Copy output to top-level dist directory
cd ../..
mkdir -p dist
rm -rf dist/*
cp -r crates/yew_frontend/dist/* dist/

echo "=== WASM build complete ==="
echo "Output: dist/"
