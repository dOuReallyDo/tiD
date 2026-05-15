#!/usr/bin/env bash
#
# build_release.sh — Local macOS development build script for tiD
#
# Usage:
#   ./scripts/build_release.sh           # Debug build
#   ./scripts/build_release.sh release   # Release build (optimized)
#
set -euo pipefail

MODE="${1:-debug}"
PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN_NAME="tid"
BUILD_DIR="${PROJECT_ROOT}/target"

echo "================================"
echo " tiD Build Script (macOS)"
echo " Mode: ${MODE}"
echo "================================"

# Build
if [ "$MODE" = "release" ]; then
  echo "Building release binary..."
  cargo build --release
  BINARY="${BUILD_DIR}/release/${BIN_NAME}"
else
  echo "Building debug binary..."
  cargo build
  BINARY="${BUILD_DIR}/debug/${BIN_NAME}"
fi

# Verify binary exists
if [ ! -f "$BINARY" ]; then
  echo "ERROR: Binary not found at ${BINARY}"
  exit 1
fi

echo "Binary built: ${BINARY}"
echo "Size: $(du -h "$BINARY" | cut -f1)"

# Package distribution
DIST_DIR="${PROJECT_DIR:-${PROJECT_ROOT}}/dist_local/tiD"
echo "Packaging into ${DIST_DIR}..."

rm -rf "${DIST_DIR}"
mkdir -p "${DIST_DIR}/frontend/dist"
mkdir -p "${DIST_DIR}/data/sources"
mkdir -p "${DIST_DIR}/data/inputs"
mkdir -p "${DIST_DIR}/data/exports"

# Copy binary
cp "${BINARY}" "${DIST_DIR}/tiD"

# Copy frontend (if built)
if [ -d "${PROJECT_ROOT}/frontend/dist" ] && [ "$(ls -A "${PROJECT_ROOT}/frontend/dist" 2>/dev/null)" ]; then
  echo "Including frontend..."
  cp -r "${PROJECT_ROOT}/frontend/dist/"* "${DIST_DIR}/frontend/dist/"
else
  echo "WARNING: frontend/dist/ is empty or missing — frontend not included"
fi

# Copy data sources (factory Excel files)
if [ -d "${PROJECT_ROOT}/data/sources" ] && [ "$(ls -A "${PROJECT_ROOT}/data/sources" 2>/dev/null)" ]; then
  echo "Including data sources..."
  cp -r "${PROJECT_ROOT}/data/sources/"* "${DIST_DIR}/data/sources/"
else
  echo "WARNING: data/sources/ is empty or missing — Excel factory files not included"
fi

# Copy startup scripts and readme
if [ -f "${PROJECT_ROOT}/START_tiD.command" ]; then
  cp "${PROJECT_ROOT}/START_tiD.command" "${DIST_DIR}/START_tiD.command"
  chmod +x "${DIST_DIR}/START_tiD.command"
fi

if [ -f "${PROJECT_ROOT}/START_tiD.bat" ]; then
  cp "${PROJECT_ROOT}/START_tiD.bat" "${DIST_DIR}/START_tiD.bat"
fi

if [ -f "${PROJECT_ROOT}/README.md" ]; then
  cp "${PROJECT_ROOT}/README.md" "${DIST_DIR}/README.md"
fi

echo ""
echo "================================"
echo " Build complete!"
echo " Distribution: ${DIST_DIR}"
echo "================================"
echo ""
echo "To run locally:"
echo "  cd ${DIST_DIR}"
echo "  ./tiD serve"