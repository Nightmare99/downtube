# downtube

**Your playlists. Your music folder. One command.**

Download a YouTube playlist as numbered, tagged MP3s in a folder named after the playlist. Written in Rust. Run it as `downtube` or **`dtube`**.

```sh
dtube 'https://www.youtube.com/playlist?list=YOUR_PLAYLIST_ID'
```

- **Music folder by default** — choose another destination with `-o`.
- **320 kbps MP3s** — select the highest-bitrate available audio source, then convert with FFmpeg.
- **Live progress** — see the track, playlist position, bytes, speed, ETA, and conversion status.
- **Easy retries** — resume partial downloads and preserve completed MP3s.
- **Playlist order** — numbered filenames, Unicode titles, and video IDs to distinguish matching names.
- **YouTube Music support** — accepts playlist links and video links containing `list=`.

## Quick start

### Linux and macOS

Install the latest release, including both command names, without Rust or `sudo`:

```sh
curl -fsSL https://github.com/Nightmare99/downtube/releases/latest/download/install.sh | sh
export PATH="$HOME/.local/bin:$PATH"
dtube --help
```

Add `~/.local/bin` to your shell profile's `PATH` if it is not already there. The installer selects your operating system and CPU, verifies the archive's SHA-256 checksum, and installs into `~/.local/bin`.

### Windows (PowerShell)

```powershell
irm https://github.com/Nightmare99/downtube/releases/latest/download/install.ps1 | iex
dtube --help
```

The Windows installer verifies the checksum, installs into `%LOCALAPPDATA%\Programs\downtube\bin`, and adds that directory to your user and current-session `PATH`. Administrator access is not required.

### Download dependencies

The installers install downtube; downloads additionally need [yt-dlp](https://github.com/yt-dlp/yt-dlp#installation), [FFmpeg](https://ffmpeg.org/download.html) with ffprobe, and [Deno](https://docs.deno.com/runtime/getting_started/installation/) or Node. On macOS:

```sh
brew install yt-dlp ffmpeg deno
```

See [Installation](docs/installation.md) for supported platforms, pinned versions, custom directories, dependency setup, upgrades, removal, and building from source. Full YouTube support also needs yt-dlp's EJS component: official standalone builds include it; Python installations should use `yt-dlp[default]`.

## Use it

```sh
# Save to your Music folder
dtube 'https://www.youtube.com/playlist?list=YOUR_PLAYLIST_ID'

# Choose a parent folder
dtube 'https://www.youtube.com/playlist?list=YOUR_PLAYLIST_ID' -o ~/Downloads

# Use a signed-in browser session
dtube 'https://music.youtube.com/playlist?list=YOUR_PLAYLIST_ID' --cookies-from-browser firefox
```

Replace `YOUR_PLAYLIST_ID` with the real playlist ID, or paste the full playlist URL. Quote URLs so the shell handles `&` correctly. `downtube` accepts the same arguments as `dtube`.

## What you get

```text
~/Music/
└── Late Night Mix/
    ├── 001 - First song [video-id-1].mp3
    ├── 002 - Second song [video-id-2].mp3
    └── 003 - Third song [video-id-3].mp3
```

The default destination follows your system's Music directory, including Linux's XDG configuration, and falls back to `~/Music`. `--output` always adds the playlist folder beneath the chosen destination.

MP3s include available metadata. Names are sanitized for filesystem compatibility, and long titles are shortened. The source may have a lower bitrate than the resulting MP3: encoding at 320 kbps cannot restore missing audio detail.

Press **Ctrl+C** to stop. Repeat the same command to resume. Failed or unavailable tracks are reported while the remaining tracks continue. Renaming or reordering a playlist can create different paths on a later run.

## Documentation

| Guide | What's inside |
| --- | --- |
| [Installation](docs/installation.md) | Binary installers, platforms, dependencies, upgrades, and removal |
| [Usage reference](docs/usage.md) | Every option, destinations, progress, retries, and exit codes |
| [Troubleshooting](docs/troubleshooting.md) | Missing tools, YouTube errors, sign-in, and interrupted files |
| [Contributing](CONTRIBUTING.md) | Local setup, checks, and contribution guidelines |
| [Architecture](docs/architecture.md) | Rust modules, downloader events, and process lifecycle |
| [Releasing](docs/releasing.md) | Versions, Git tags, release artifacts, and workflow checks |
| [Changelog](CHANGELOG.md) | Changes in each version |

## Development

```sh
cargo run -- 'https://www.youtube.com/playlist?list=YOUR_PLAYLIST_ID'
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
```

Automated tests run without downloading from YouTube. See [Contributing](CONTRIBUTING.md) for coverage and platform details.

## License

[MIT](LICENSE).
