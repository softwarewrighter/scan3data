#!/bin/bash
set -euo pipefail

# Development mode for WASM frontend (auto-rebuild on changes)

echo "=== Starting Yew development server ==="
echo "Watches for changes and auto-rebuilds"
echo ""

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

# Start development server with live reload
trunk serve --open
