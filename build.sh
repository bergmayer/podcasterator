#!/bin/bash
# Build script for Podcasterator

set -e

show_help() {
    cat << 'HELP'
Usage: ./build.sh [OPTION]

Options:
  (none)          Build raw binary only
  --makebundle    Build portable distributable (AppImage, .app)
  --makepackage   Build system installer/package
  --check         Check build dependencies only
  --clean         Remove all build artifacts
  --help          Show this help message

--makepackage supported formats:
  macOS:   DMG
  Linux:   pacman (Arch), deb (Debian/Ubuntu), rpm (Fedora/RHEL)
  Windows: NSIS, MSI
HELP
}

MAKEBUNDLE=false
MAKEPACKAGE=false
CHECK=false
CLEAN=false
for arg in "$@"; do
    case "$arg" in
        --makebundle) MAKEBUNDLE=true ;;
        --makepackage) MAKEPACKAGE=true ;;
        --check) CHECK=true ;;
        --clean) CLEAN=true ;;
        --help|-h) show_help; exit 0 ;;
        *) echo "Unknown option: $arg"; echo "Run './build.sh --help' for usage."; exit 1 ;;
    esac
done

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

RELEASES_DIR="$SCRIPT_DIR/releases"

# --- Clean ---

if [ "$CLEAN" = true ]; then
    echo "Cleaning build artifacts..."
    rm -rf src-tauri/target
    rm -rf node_modules
    rm -rf dist
    rm -rf "$RELEASES_DIR"/*
    echo "Clean complete."
    exit 0
fi

# --- Dependency checks ---

MISSING=()
INSTALL_HINT=""
BUNDLE_MISSING=()
BUNDLE_INSTALL_HINT=""

check_command() {
    if ! command -v "$1" &>/dev/null; then
        MISSING+=("$2")
    fi
}

check_bundle_command() {
    if ! command -v "$1" &>/dev/null; then
        BUNDLE_MISSING+=("$2")
    fi
}

check_pkg() {
    if command -v pkg-config &>/dev/null; then
        if ! pkg-config --exists "$1" 2>/dev/null; then
            MISSING+=("$2")
        fi
    fi
}

detect_linux_distro() {
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        echo "$ID"
    elif command -v pacman &>/dev/null; then
        echo "arch"
    elif command -v apt-get &>/dev/null; then
        echo "ubuntu"
    elif command -v dnf &>/dev/null; then
        echo "fedora"
    else
        echo "unknown"
    fi
}

# Common dependencies
check_command "rustc" "Rust (install from https://rustup.rs/)"
check_command "cargo" "Cargo (install from https://rustup.rs/)"
check_command "node" "Node.js (install from https://nodejs.org/)"
check_command "npm" "npm (install from https://nodejs.org/)"

OS="$(uname)"
case "$OS" in
    Darwin)
        if ! xcode-select -p &>/dev/null; then
            MISSING+=("Xcode Command Line Tools")
            INSTALL_HINT="  xcode-select --install"
        fi
        ;;
    Linux)
        DISTRO=$(detect_linux_distro)

        check_command "pkg-config" "pkg-config"
        check_pkg "gtk+-3.0" "GTK3"
        check_pkg "webkit2gtk-4.1" "WebKit2GTK 4.1"
        check_pkg "openssl" "OpenSSL"
        check_pkg "ayatana-appindicator3-0.1" "libayatana-appindicator"
        check_pkg "librsvg-2.0" "librsvg"

        if [ ${#MISSING[@]} -gt 0 ]; then
            case "$DISTRO" in
                arch|manjaro|endeavouros)
                    INSTALL_HINT="  sudo pacman -S webkit2gtk-4.1 openssl libappindicator-gtk3 librsvg"
                    ;;
                ubuntu|debian|pop|linuxmint)
                    INSTALL_HINT="  sudo apt install libwebkit2gtk-4.1-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev"
                    ;;
                fedora)
                    INSTALL_HINT="  sudo dnf install webkit2gtk4.1-devel openssl-devel libappindicator-gtk3-devel librsvg2-devel"
                    ;;
                rhel|centos|rocky|alma)
                    INSTALL_HINT="  sudo yum install webkit2gtk4.1-devel openssl-devel libappindicator-gtk3-devel librsvg2-devel"
                    ;;
                opensuse*|suse*)
                    INSTALL_HINT="  sudo zypper install webkit2gtk3-devel libopenssl-devel libappindicator3-1 librsvg-devel"
                    ;;
                gentoo)
                    INSTALL_HINT="  sudo emerge --ask net-libs/webkit-gtk:4.1 dev-libs/libappindicator"
                    ;;
                alpine)
                    INSTALL_HINT="  sudo apk add webkit2gtk-4.1-dev openssl libayatana-appindicator-dev librsvg"
                    ;;
                *)
                    INSTALL_HINT="  Install the equivalent packages for your distro (see https://v2.tauri.app/start/prerequisites/)"
                    ;;
            esac
        fi

        # Additional dependencies for AppImage bundling
        if [ "$MAKEBUNDLE" = true ]; then
            check_bundle_command "fusermount" "FUSE"
            check_bundle_command "patchelf" "patchelf"
            check_bundle_command "file" "file"
            check_bundle_command "zip" "zip"

            if [ ${#BUNDLE_MISSING[@]} -gt 0 ]; then
                case "$DISTRO" in
                    arch|manjaro|endeavouros)
                        BUNDLE_INSTALL_HINT="  sudo pacman -S fuse2 patchelf file zip"
                        ;;
                    ubuntu|debian|pop|linuxmint)
                        BUNDLE_INSTALL_HINT="  sudo apt install libfuse2 patchelf file zip"
                        ;;
                    fedora)
                        BUNDLE_INSTALL_HINT="  sudo dnf install fuse patchelf file zip"
                        ;;
                    rhel|centos|rocky|alma)
                        BUNDLE_INSTALL_HINT="  sudo yum install fuse patchelf file zip"
                        ;;
                    opensuse*|suse*)
                        BUNDLE_INSTALL_HINT="  sudo zypper install fuse patchelf file zip"
                        ;;
                    alpine)
                        BUNDLE_INSTALL_HINT="  sudo apk add fuse patchelf file zip"
                        ;;
                    *)
                        BUNDLE_INSTALL_HINT="  Install fuse, patchelf, file, and zip for your distro"
                        ;;
                esac
            fi
        fi

        # Additional dependencies for native packaging
        if [ "$MAKEPACKAGE" = true ]; then
            check_bundle_command "zip" "zip"
            case "$DISTRO" in
                arch|manjaro|endeavouros)
                    check_bundle_command "makepkg" "makepkg (base-devel)"
                    ;;
                ubuntu|debian|pop|linuxmint)
                    check_bundle_command "dpkg-deb" "dpkg-dev"
                    ;;
                fedora|rhel|centos|rocky|alma)
                    check_bundle_command "rpmbuild" "rpm-build"
                    ;;
            esac
        fi
        ;;
esac

HAS_ERRORS=false

if [ ${#MISSING[@]} -gt 0 ]; then
    HAS_ERRORS=true
    echo "ERROR: Missing required build dependencies:"
    for dep in "${MISSING[@]}"; do
        echo "  - $dep"
    done
    if [ -n "$INSTALL_HINT" ]; then
        echo ""
        echo "Install with:"
        echo "$INSTALL_HINT"
    fi
    echo ""
fi

if [ ${#BUNDLE_MISSING[@]} -gt 0 ]; then
    HAS_ERRORS=true
    echo "ERROR: Missing dependencies for packaging:"
    for dep in "${BUNDLE_MISSING[@]}"; do
        echo "  - $dep"
    done
    if [ -n "$BUNDLE_INSTALL_HINT" ]; then
        echo ""
        echo "Install with:"
        echo "$BUNDLE_INSTALL_HINT"
    fi
    echo ""
fi

if [ "$HAS_ERRORS" = true ]; then
    exit 1
fi

if [ "$CHECK" = true ]; then
    echo "All dependencies satisfied."
    exit 0
fi

# --- Helper: zip and move to releases ---

zip_to_releases() {
    local src="$1"
    local zipname="$2"
    mkdir -p "$RELEASES_DIR"
    if [ -d "$src" ]; then
        (cd "$(dirname "$src")" && zip -r "$RELEASES_DIR/$zipname" "$(basename "$src")")
        rm -rf "$src"
        echo "  Created releases/$zipname"
    elif [ -f "$src" ]; then
        (cd "$(dirname "$src")" && zip "$RELEASES_DIR/$zipname" "$(basename "$src")")
        rm -f "$src"
        echo "  Created releases/$zipname"
    else
        echo "  Warning: could not find $src"
    fi
}

# --- Native package build ---

if [ "$MAKEPACKAGE" = true ]; then
    mkdir -p "$RELEASES_DIR"

    case "$OS" in
        Linux)
            case "$DISTRO" in
                arch|manjaro|endeavouros)
                    echo "Building Arch package..."
                    cd "$SCRIPT_DIR/pkg/arch"
                    makepkg -sf
                    PKG=$(ls -t "$SCRIPT_DIR/pkg/arch/"*.pkg.tar* 2>/dev/null | head -1)
                    zip_to_releases "$PKG" "Podcasterator-linux-arch.zip"
                    cd "$SCRIPT_DIR"
                    echo ""
                    echo "Build complete!"
                    echo ""
                    echo "Install with:"
                    echo "  sudo pacman -U releases/$(basename "$PKG")"
                    ;;
                ubuntu|debian|pop|linuxmint)
                    echo "Building .deb package..."
                    npm install
                    npm run tauri build -- --bundles deb
                    DEB=$(find src-tauri/target/release/bundle/deb -name "*.deb" | head -1)
                    zip_to_releases "$DEB" "Podcasterator-linux-deb.zip"
                    echo ""
                    echo "Build complete!"
                    echo ""
                    echo "Install with:"
                    echo "  unzip releases/Podcasterator-deb.zip && sudo dpkg -i *.deb"
                    ;;
                fedora|rhel|centos|rocky|alma)
                    echo "Building .rpm package..."
                    npm install
                    npm run tauri build -- --bundles rpm
                    RPM=$(find src-tauri/target/release/bundle/rpm -name "*.rpm" | head -1)
                    zip_to_releases "$RPM" "Podcasterator-linux-rpm.zip"
                    echo ""
                    echo "Build complete!"
                    echo ""
                    echo "Install with:"
                    echo "  unzip releases/Podcasterator-rpm.zip && sudo rpm -i *.rpm"
                    ;;
                *)
                    echo "ERROR: Native packaging not supported for distro '$DISTRO'."
                    echo "Use --makebundle for an AppImage instead."
                    exit 1
                    ;;
            esac
            ;;
        Darwin)
            echo "Building macOS DMG..."
            npm install
            rustup target add aarch64-apple-darwin x86_64-apple-darwin 2>/dev/null || true
            npm run tauri build -- --target universal-apple-darwin --bundles dmg
            DMG=$(find src-tauri/target/universal-apple-darwin/release/bundle/dmg -name "*.dmg" | head -1)
            zip_to_releases "$DMG" "Podcasterator-macos-dmg.zip"
            echo ""
            echo "Build complete!"
            ;;
        MINGW*|MSYS*|CYGWIN*)
            echo "Building Windows installers..."
            npm install
            npm run tauri build -- --bundles nsis,msi
            NSIS_EXE=$(find src-tauri/target/release/bundle/nsis -name "*.exe" 2>/dev/null | head -1)
            zip_to_releases "$NSIS_EXE" "Podcasterator-windows-nsis.zip"
            MSI_FILE=$(find src-tauri/target/release/bundle/msi -name "*.msi" 2>/dev/null | head -1)
            zip_to_releases "$MSI_FILE" "Podcasterator-windows-msi.zip"
            echo ""
            echo "Build complete!"
            ;;
    esac
    exit 0
fi

# --- Build ---

echo "Installing npm dependencies..."
npm install

if [ "$MAKEBUNDLE" = true ]; then
    mkdir -p "$RELEASES_DIR"

    case "$OS" in
        Darwin)
            echo "Building macOS .app bundle..."
            rustup target add aarch64-apple-darwin x86_64-apple-darwin 2>/dev/null || true
            npm run tauri build -- --target universal-apple-darwin --bundles app
            APP=$(find src-tauri/target/universal-apple-darwin/release/bundle/macos -name "*.app" -maxdepth 1 | head -1)
            zip_to_releases "$APP" "Podcasterator-macos.zip"
            echo ""
            echo "Build complete!"
            ;;
        Linux)
            echo "Building Linux AppImage..."
            # Try the standard Tauri AppImage build first
            if npm run tauri build -- --bundles appimage; then
                echo ""
                echo "Build complete!"
            else
                echo ""
                echo "Tauri's AppImage bundler failed (common on Arch/rolling-release distros)."
                echo "Building AppImage manually with NO_STRIP..."
                APPDIR="src-tauri/target/release/bundle/appimage"
                # Find the .AppDir directory
                APPDIR_PATH=$(find "$APPDIR" -maxdepth 1 -name "*.AppDir" -type d | head -1)
                if [ -z "$APPDIR_PATH" ]; then
                    echo "ERROR: No .AppDir found. The Tauri build may not have gotten far enough."
                    exit 1
                fi
                NO_STRIP=true ~/.cache/tauri/linuxdeploy-x86_64.AppImage \
                    --appimage-extract-and-run \
                    --appdir "$APPDIR_PATH" \
                    --output appimage
                # linuxdeploy outputs to current dir, move it to the bundle dir
                mv -f Podcasterator*.AppImage "$APPDIR/" 2>/dev/null || true
                echo ""
                echo "Build complete!"
            fi
            APPIMAGE=$(find src-tauri/target/release/bundle/appimage -name "*.AppImage" | head -1)
            zip_to_releases "$APPIMAGE" "Podcasterator-linux-AppImage.zip"
            echo ""
            ;;
        MINGW*|MSYS*|CYGWIN*)
            echo "Note: On Windows, --makebundle is the same as a plain build (portable exe)."
            echo "Use --makepackage for NSIS/MSI installers."
            npm run tauri build -- --no-bundle
            zip_to_releases "src-tauri/target/release/podcasterator.exe" "Podcasterator-windows.zip"
            echo ""
            echo "Build complete!"
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
