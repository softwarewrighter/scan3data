#!/bin/bash
set -euo pipefail

# Serve standalone SPA mode (no backend, all processing in browser)

PORT="${1:-8080}"

echo "=== Serving SPA mode on port ${PORT} ==="
echo "All processing happens in the browser (WASM)"
echo "No backend API calls"
echo ""
echo "Open: http://localhost:${PORT}"
echo ""

# Check if dist exists
if [ ! -d "dist" ]; then
    echo "dist/ directory not found. Building WASM frontend..."
    ./scripts/build-wasm.sh
fi

# Serve using simple HTTP server
if command -v python3 &> /dev/null; then
    cd dist
    python3 -m http.server "${PORT}"
elif command -v python &> /dev/null; then
    cd dist
    python -m SimpleHTTPServer "${PORT}"
else
    echo "Error: Python not found. Install Python or use another HTTP server."
    exit 1
fi
