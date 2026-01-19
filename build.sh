#!/bin/bash
# Build script for Podcasterator

set -e

echo "Installing npm dependencies..."
npm install

echo "Building Podcasterator..."
npm run tauri build

echo ""
echo "Build complete!"
echo ""
echo "The binary is located at:"
echo "  src-tauri/target/release/podcasterator"
echo ""
echo "You can copy this binary to any location you prefer."
