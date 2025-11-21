# Podcasterator

A cross-platform GUI app (Go + Fyne) that creates a local podcast server from your audio files. Turn any MP3, M4A, MP4, or M4B files into a podcast feed you can subscribe to in your favorite podcast app.

## Features

- **Drag & Drop**: Add audio files and folders instantly
- **Podcast Artwork**: Drag images to set artwork (auto-converted to 1400x1400 JPEG)
- **Playlist Management**: Reorder with arrow buttons, alphabetize, or clear all
- **Local Server**: RSS feed on port 8080 with one-click URL copying
- **Safe**: Original files never modified (copies to temp directory)
- **Cross-platform**: macOS and Linux

## Quick Start

```bash
# Build
go build -o podcasterator

# Run
./podcasterator
```

## Usage

1. **Add Files**: Drag audio files/folders onto the app or click the drop zone
2. **Set Artwork** (optional): Drag an image file onto the app, or click "No artwork set"
3. **Name Your Podcast** (optional): Enter a name in the text field
4. **Launch Server**: Click "Launch Local Podcast Server"
5. **Copy URL**: Click "Copy URL" and paste into your podcast app
6. **Subscribe**: Your podcast app will download the episodes

### Managing Files

- **↑/↓**: Move files up/down in the list
- **✏️**: Rename a file
- **×**: Delete individual files
- **Clear All**: Remove all files from the playlist
- **Alphabetize**: Sort files A-Z by filename
- **Reverse**: Reverse the current file order

**Artwork:**
- **No artwork set**: Click to select an image file
- **Delete artwork**: Click to remove the current artwork

## Building

### Requirements

- Go 1.21+
- Platform-specific dependencies:
  - **macOS**: Xcode Command Line Tools (`xcode-select --install`)
  - **Linux**:
    - **X11 (basic)**: `libgl1-mesa-dev xorg-dev` (Debian/Ubuntu)
    - **Wayland (recommended)**: `libgl1-mesa-dev xorg-dev libwayland-dev libxkbcommon-dev wayland-protocols` (Debian/Ubuntu)
    - **WSL**: Works on Windows via WSL2 with native Linux GUI support

### Build Commands

```bash
# Standard build
go build -o podcasterator

# Optimized build (smaller binary)
go build -ldflags="-s -w" -o podcasterator
```

## Wayland Support (Linux)

Podcasterator now includes automatic Wayland detection and configuration. The application should work on pure Wayland compositors (Niri, Sway, etc.) and Wayland sessions with XWayland support (Cosmic Desktop, GNOME, KDE).

### Automatic Configuration

The application automatically detects Wayland sessions and configures the GLFW backend accordingly. This happens transparently when you run the app.

### Manual Launch (Alternative)

If you experience issues, use the provided launch script:

```bash
# Make the script executable (first time only)
chmod +x launch_wayland.sh

# Launch the app
./launch_wayland.sh
```

### Drag-and-Drop on Wayland

**Known Issue**: Native drag-and-drop may not work reliably on all Wayland compositors due to GLFW/Wayland protocol limitations.

**Workaround**: Use the click-to-select method instead:
1. Click on the drop zone area in the app
2. Use the file picker dialog to select audio files or images
3. The file dialog uses XDG Desktop Portal and works reliably on Wayland

### Build Requirements for Wayland

Ensure you have the necessary Wayland development libraries:

```bash
# Debian/Ubuntu/Mint
sudo apt install libwayland-dev libxkbcommon-dev wayland-protocols

# Fedora/RHEL
sudo dnf install wayland-devel libxkbcommon-devel wayland-protocols-devel

# Arch Linux
sudo pacman -S wayland libxkbcommon wayland-protocols
```

### Troubleshooting Wayland

**App doesn't launch:**
- Ensure GLFW is built with Wayland support (dependencies should handle this automatically)
- Try the launch script: `./launch_wayland.sh`
- Check environment variables: `echo $WAYLAND_DISPLAY $XDG_SESSION_TYPE`

**Drag-and-drop doesn't work:**
- This is a known limitation on some Wayland compositors
- Use the click-to-select method instead (click the drop zone)
- The file picker dialog uses XDG Desktop Portal and should work

**App window issues:**
- Some Wayland compositors handle window decorations differently
- If the window appears incorrectly, try running with X11 fallback: `GDK_BACKEND=x11 ./podcasterator`

## File Locations

### Temporary Files (Audio & Artwork)

Files are copied here when added to the app:

- **macOS**: `~/Library/Caches/podcasterator/`
- **Linux**: `~/.cache/podcasterator/` (follows XDG Base Directory spec)
  - Or `$XDG_CACHE_HOME/podcasterator/` if set
  - **WSL**: Same as Linux (`~/.cache/podcasterator/` in your WSL home)

**Structure:**
```
podcasterator/
├── artwork.jpg                    (podcast artwork)
├── [uuid-1]/audiofile1.m4a       (copied audio files)
├── [uuid-2]/audiofile2.mp3
└── [uuid-3]/audiofile3.m4a
```

**Notes:**
- Original files are never modified
- Temp files persist between app launches
- MP4/M4B files are renamed to .m4a for compatibility
- Use "Clear All" to remove all temp files

### Configuration (State & Settings)

- **macOS**: `~/Library/Application Support/Podcasterator/state.json`
- **Linux**: `~/.config/Podcasterator/state.json` (follows XDG Base Directory spec)
  - Or `$XDG_CONFIG_HOME/Podcasterator/state.json` if set
  - **WSL**: Same as Linux (`~/.config/Podcasterator/state.json` in your WSL home)

## Technical Details

- **Language**: Go 1.21+
- **GUI**: Fyne v2
- **RSS**: gorilla/feeds
- **Image Processing**: nfnt/resize
- **Port**: 8080 (no admin required)
- **Feed Format**: RSS 2.0 with iTunes extensions

## Supported Formats

**Audio**: MP3, M4A, MP4, M4B (MP4/M4B auto-renamed to M4A)
**Images**: PNG, JPG, JPEG, GIF, BMP, TIFF

## How It Works

1. Audio files are copied to a temp directory with unique IDs
2. File modification times are adjusted to control episode order
3. RSS feed is generated with enclosures pointing to local files
4. HTTP server serves the feed and audio files on port 8080
5. Your podcast app downloads episodes like any other podcast
6. Once podcast episodes are downloaded by your app, you can stop the server

## License

Released into the public domain under the Unlicense.

## Credits

Created with [Claude Code](https://claude.com/claude-code).
