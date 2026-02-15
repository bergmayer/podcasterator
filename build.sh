#!/bin/bash
# Build script for Podcasterator
#
# Usage:
#   ./build.sh              Build raw binary only
#   ./build.sh --makebundle Build platform-specific distributable bundle

set -e

MAKEBUNDLE=false
for arg in "$@"; do
    case "$arg" in
        --makebundle) MAKEBUNDLE=true ;;
        *) echo "Unknown option: $arg"; echo "Usage: ./build.sh [--makebundle]"; exit 1 ;;
    esac
done

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

echo "Installing npm dependencies..."
npm install

if [ "$MAKEBUNDLE" = true ]; then
    OS="$(uname)"
    case "$OS" in
        Darwin)
            echo "Building universal macOS bundle..."
            rustup target add aarch64-apple-darwin x86_64-apple-darwin 2>/dev/null || true
            npm run tauri build -- --target universal-apple-darwin --bundles app,dmg
            echo ""
            echo "Build complete!"
            echo ""
            echo "Bundle located at:"
            echo "  src-tauri/target/universal-apple-darwin/release/bundle/dmg/"
            ;;
        Linux)
            echo "Building Linux AppImage..."
            npm run tauri build -- --bundles appimage
            echo ""
            echo "Build complete!"
            echo ""
            echo "Bundles located at:"
            echo "  src-tauri/target/release/bundle/"
            ;;
        MINGW*|MSYS*|CYGWIN*)
            echo "Building Windows bundles..."
            npm run tauri build -- --bundles nsis,msi
            echo ""
            echo "Build complete!"
            echo ""
            echo "Bundles located at:"
            echo "  src-tauri/target/release/bundle/"
            ;;
        *)
            echo "Unknown platform: $OS. Building with default bundles..."
            npm run tauri build
            ;;
    esac
else
    echo "Building Podcasterator (binary only)..."
    npm run tauri build -- --no-bundle
    echo ""
    echo "Build complete!"
    echo ""
    echo "The binary is located at:"
    echo "  src-tauri/target/release/podcasterator"
fi
