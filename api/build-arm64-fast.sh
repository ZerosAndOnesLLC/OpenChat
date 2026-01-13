#!/bin/bash

# Fast build script - builds on host first, then creates minimal Docker image

set -e

echo "=========================================="
echo "Fast ARM64 Build Process"
echo "=========================================="

# Step 1: Build with cargo on host (uses incremental compilation)
echo ""
echo "[1/3] Building with cargo (incremental)..."
cargo build --release

# Step 2: Remove old Docker image
echo ""
echo "[2/3] Removing old Docker image..."
docker rmi openchat-api 2>/dev/null || true

# Step 3: Build minimal Docker image with pre-built binary
echo ""
echo "[3/3] Creating Docker image..."
docker build --provenance=false -f Dockerfile.arm64.fast -t openchat-api .

echo ""
echo "=========================================="
echo "Build complete!"
echo "Image tagged as: openchat-api"
echo "=========================================="
