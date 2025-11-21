# Podcasterator

A cross-platform GUI app (Go + Fyne) that creates a local podcast server from your audio files. Turn any MP3, M4A, MP4, or M4B files into a podcast feed you can subscribe to in your favorite podcast app.

## Features

- **Drag & Drop**: Add audio files and folders instantly
- **Podcast Artwork**: Drag images to set artwork (auto-converted to 1400x1400 JPEG)
- **Playlist Management**: Reorder with arrow buttons, alphabetize, or clear all
- **Local Server**: RSS feed on port 8080 with one-click URL copying
- **Safe**: Original files never modified (copies to temp directory)
- **Cross-platform**: macOS, Linux, and Windows

## Quick Start

**macOS/Linux:**
```bash
go build -o podcasterator
./podcasterator
```

**Windows:**
```powershell
go build -ldflags "-H windowsgui" -o podcasterator.exe
.\podcasterator.exe
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

**Tested and built on:** Linux, macOS, and Windows

### Requirements

- **Go** 1.21+
- **C Compiler**:
  - macOS: Xcode Command Line Tools
  - Linux: GCC (via build-essential or equivalent)
  - Windows: MinGW-w64 (via Chocolatey: `choco install mingw`)

### Build Commands

**macOS/Linux:**
```bash
go build -o podcasterator
```

**Windows:**
```powershell
$env:CC = "gcc"
go build -ldflags "-H windowsgui" -o podcasterator.exe
```

## Platform Notes

### Linux

**Tested and works on:** KDE Plasma (Wayland)

**Known Issues:**
- **Cosmic Desktop**: No drag-and-drop support (use click-to-select file picker)
- **Niri**: Does not launch due to Wayland compatibility issues
- **WSL**: Does not work due to networking issues
- **YMMV** for other Linux environments

**Workaround:** Click the drop zone to open the file picker instead of drag-and-drop.

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
