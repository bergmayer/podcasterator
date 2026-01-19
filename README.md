# Podcasterator

A cross-platform desktop app that creates a local podcast server from your audio files. Turn any MP3, M4A, MP4, or M4B files into a podcast feed you can subscribe to in your favorite podcast app.

## Building

### Dependencies

- **Rust** (via rustup)
- **Node.js** 18+
- **Platform-specific libraries**:
  - **macOS**: Xcode Command Line Tools
  - **Linux**: `sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libappindicator3-dev librsvg2-dev patchelf`
  - **Windows**: WebView2 (usually pre-installed on Windows 10/11)

Build scripts are provided for all platforms:

- **macOS/Linux**: `./build.sh`
- **Windows**: `build.bat`

## File Locations

### Temporary Files (Audio & Artwork)

Files are copied here when added to the app:

- **macOS**: `~/Library/Caches/podcasterator/`
- **Linux**: `~/.cache/podcasterator/` (follows XDG Base Directory spec)
- **Windows**: `%LOCALAPPDATA%\podcasterator\cache\`

### Configuration (State & Settings)

- **macOS**: `~/Library/Application Support/Podcasterator/state.json`
- **Linux**: `~/.config/Podcasterator/state.json` (follows XDG Base Directory spec)
- **Windows**: `%APPDATA%\Podcasterator\state.json`

## Technical Details

- **Backend**: Rust with Tauri
- **Frontend**: Svelte 5 with TypeScript
- **HTTP Server**: Axum
- **RSS**: rss crate
- **Image Processing**: image crate (Lanczos3 resampling)
- **Port**: 8080
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

Released under the MIT license.
