#!/bin/bash
# Podcasterator Wayland Launch Script
#
# This script ensures proper Wayland support for Podcasterator on Linux.
# It sets necessary environment variables and launches the application.

# Detect if running on Wayland
if [ -n "$WAYLAND_DISPLAY" ] || [ "$XDG_SESSION_TYPE" = "wayland" ]; then
    echo "Detected Wayland session, configuring environment..."

    # Force GLFW to use Wayland backend
    export GLFW_PLATFORM=wayland

    # Set GDK backend to prefer Wayland
    export GDK_BACKEND=wayland,x11

    # Qt applications (if any) should use Wayland
    export QT_QPA_PLATFORM=wayland;xcb

    # Enable Wayland for SDL2 (if used by dependencies)
    export SDL_VIDEODRIVER=wayland

    # XDG Desktop Portal for file dialogs
    export GTK_USE_PORTAL=1

    echo "Environment configured for Wayland"
else
    echo "Running on X11 session"
fi

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Launch the Podcasterator binary
# Adjust the binary path as needed (e.g., ./podcasterator, ./bin/podcasterator, etc.)
BINARY="$SCRIPT_DIR/podcasterator"

if [ ! -f "$BINARY" ]; then
    echo "Error: Podcasterator binary not found at $BINARY"
    echo "Please build the application first with: go build"
    exit 1
fi

echo "Launching Podcasterator..."
exec "$BINARY" "$@"
