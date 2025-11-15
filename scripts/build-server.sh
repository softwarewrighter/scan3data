#!/bin/bash
set -euo pipefail

# Build just the server (faster than full workspace build)

echo "=== Building Server ==="

cargo build --package scan3data-server --release

echo "=== Server build complete ==="
echo "Binary: target/release/scan3data-server"
