# Podcasterator

A cross-platform (macOS and Linux) desktop app that creates a local podcast server from your audio files. Turn any MP3, M4A, MP4, or M4B files into a podcast feed you can subscribe to in your favorite podcast app.

## Features

- **Drag & Drop**: Add audio files and folders instantly
- **Podcast Artwork**: Drag images to set artwork (auto-converted to 1400x1400 JPEG)
- **Playlist Management**: Reorder with arrow buttons, alphabetize, or clear all
- **Local Server**: RSS feed on port 8080 with one-click URL copying
- **Safe**: Original files never modified (copies to temp directory)
- **Cross-platform**: macOS and Linux

## Quick Start

```bash
./build.sh
./src-tauri/target/release/podcasterator
```

Or install from a package:
- **Debian/Ubuntu**: `sudo dpkg -i src-tauri/target/release/bundle/deb/Podcasterator_*.deb`
- **Fedora/RHEL**: `sudo rpm -i src-tauri/target/release/bundle/rpm/Podcasterator-*.rpm`

## Usage

1. **Add Files**: Drag audio files/folders onto the app or click the buttons in the drop zone
2. **Set Artwork** (optional): Click the artwork area to select an image
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
- Click the artwork area to select an image file
- **Delete artwork**: Click the delete button to remove the current artwork

## Building

### Requirements

- **Rust** (via rustup)
- **Node.js** 18+
- **System Dependencies**:
  - macOS: Xcode Command Line Tools
  - Linux: `sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libappindicator3-dev librsvg2-dev patchelf`

### Build Command

```bash
./build.sh
```

## Platform Notes

- **Windows**: Run under WSL (Windows Subsystem for Linux)
- **Linux**: Requires WebKit2GTK and GTK3 development libraries

## File Locations

### Temporary Files (Audio & Artwork)

Files are copied here when added to the app:

- **macOS**: `~/Library/Caches/podcasterator/`
- **Linux**: `~/.cache/podcasterator/` (follows XDG Base Directory spec)
  - Or `$XDG_CACHE_HOME/podcasterator/` if set

**Notes:**
- Original files are never modified
- Temp files persist between app launches
- MP4/M4B files are renamed to .m4a for compatibility
- Use "Clear All" to remove all temp files

### Configuration (State & Settings)

- **macOS**: `~/Library/Application Support/Podcasterator/state.json`
- **Linux**: `~/.config/Podcasterator/state.json` (follows XDG Base Directory spec)
  - Or `$XDG_CONFIG_HOME/Podcasterator/state.json` if set

## Technical Details

- **Backend**: Rust with Tauri
- **Frontend**: Svelte 5 with TypeScript
- **HTTP Server**: Axum
- **RSS**: rss crate
- **Image Processing**: image crate (Lanczos3 resampling)
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

Released under the GPL 3.0 license.
