#!/bin/bash
set -euo pipefail

# Build just the CLI (faster than full workspace build)

echo "=== Building CLI ==="

cargo build --package scan2data-cli --release

echo "=== CLI build complete ==="
echo "Binary: target/release/scan2data"
