# Podcasterator

A cross-platform desktop app that creates a local podcast server from your audio files. Turn any MP3, M4A, MP4, or M4B files into a podcast feed you can subscribe to in a podcast app.

## Building

### Dependencies

- [Rust](https://rustup.rs/)
- [Node.js](https://nodejs.org/)
- **macOS**: Xcode Command Line Tools
- **Linux**: WebKit2GTK 4.1, GTK3, OpenSSL, libayatana-appindicator, librsvg

The build scripts check for missing dependencies before building and provide distro-specific install commands. To install Linux dependencies manually:

### Build

- **macOS/Linux**: `./build.sh`
- **Windows**: `build.bat`

Options:
- `--makebundle` — Build a distributable bundle (AppImage, DMG, or NSIS/MSI)
- `--makepackage` — Build and install a native package for your distro (pacman, deb, rpm)
- `--clean` — Remove all build artifacts

## File Locations

### Temporary Files (Audio & Artwork)

Files are copied here when added to the app:

- **macOS**: `~/Library/Caches/podcasterator/`
- **Linux**: `~/.cache/podcasterator/` (follows XDG Base Directory spec)
- **Windows**: `%LOCALAPPDATA%\podcasterator\`

They are deleted when cleared from the interface or when the app quits. The artwork and podcast name persist between launches unless manually cleared. 

### Configuration (State & Settings)

- **macOS**: `~/Library/Application Support/Podcasterator/state.json`
- **Linux**: `~/.config/Podcasterator/state.json` (follows XDG Base Directory spec)
- **Windows**: `%APPDATA%\Podcasterator\state.json`

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
