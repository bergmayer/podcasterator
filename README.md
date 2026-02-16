# Podcasterator

A cross-platform desktop app that creates a local podcast server from your audio files. Turn any MP3, M4A, MP4, or M4B files into a podcast feed you can subscribe to in your favorite podcast app.

## Building

### Dependencies

- [Rust](https://rustup.rs/)
- [Node.js](https://nodejs.org/)
- **macOS**: Xcode Command Line Tools
- **Linux**: WebKit2GTK 4.1, GTK3, OpenSSL, libayatana-appindicator, librsvg

The build scripts check for missing dependencies before building and provide distro-specific install commands. To install Linux dependencies manually:

- **Arch**: `sudo pacman -S webkit2gtk-4.1 openssl libappindicator-gtk3 librsvg`
- **Debian/Ubuntu**: `sudo apt install libwebkit2gtk-4.1-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev`
- **Fedora**: `sudo dnf install webkit2gtk4.1-devel openssl-devel libappindicator-gtk3-devel librsvg2-devel`
- **RHEL/CentOS**: `sudo yum install webkit2gtk4.1-devel openssl-devel libappindicator-gtk3-devel librsvg2-devel`
- **openSUSE**: `sudo zypper install webkit2gtk3-devel libopenssl-devel libappindicator3-1 librsvg-devel`

See the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) page for other distros.

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

### Configuration (State & Settings)

- **macOS**: `~/Library/Application Support/Podcasterator/state.json`
- **Linux**: `~/.config/Podcasterator/state.json` (follows XDG Base Directory spec)
- **Windows**: `%APPDATA%\Podcasterator\state.json`

## Technical Details

- **Desktop App**: [Tauri](https://crates.io/crates/tauri) (Rust) with [Svelte](https://svelte.dev/) 5 (TypeScript)
- **Podcast Server**: [Axum](https://crates.io/crates/axum)
- **RSS Generation**: [rss](https://crates.io/crates/rss) crate
- **Image Processing**: [image](https://crates.io/crates/image) crate (Lanczos3 resampling)
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
