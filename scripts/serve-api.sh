#!/bin/bash
set -euo pipefail

# Serve API mode (backend + frontend)

PORT="${1:-3000}"

echo "=== Serving API mode ==="
echo "Backend API: http://localhost:${PORT}"
echo "Frontend: Served by backend at /static"
echo ""
echo "This mode supports:"
echo "  - REST API endpoints"
echo "  - Backend image processing"
echo "  - Ollama LLM integration"
echo ""

# Check if server binary exists
if [ ! -f "target/release/scan3data-server" ]; then
    echo "Server binary not found. Building..."
    ./scripts/build-server.sh
fi

# Check if dist exists for static serving
if [ ! -d "dist" ]; then
    echo "WASM frontend not built. Building..."
    ./scripts/build-wasm.sh
fi

# Run the server
export RUST_LOG=info
./target/release/scan3data-server
