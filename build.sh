#!/bin/bash
# Build script for Podcasterator

set -e

echo "Installing npm dependencies..."
npm install

echo "Building Podcasterator for $(uname -s)..."
npm run tauri build

echo ""
echo "Build complete!"
echo "Binary: src-tauri/target/release/podcasterator"
echo "Packages: src-tauri/target/release/bundle/"
