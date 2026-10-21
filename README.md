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

Install [Rust](https://rustup.rs/) and the download dependencies. On macOS with Homebrew:

```sh
brew install yt-dlp ffmpeg deno
```

Clone the repository and install both commands:

```sh
git clone https://github.com/Nightmare99/downtube.git
cd downtube
cargo install --locked --path .
dtube --help
```

Ensure Cargo's binary directory, usually `~/.cargo/bin`, is on your `PATH`.

On Linux or Windows, install [yt-dlp](https://github.com/yt-dlp/yt-dlp#installation), [FFmpeg](https://ffmpeg.org/download.html) with ffprobe, and [Deno](https://docs.deno.com/runtime/getting_started/installation/), then run the same Cargo command. Full YouTube support also needs yt-dlp's EJS component: official standalone builds include it; Python installations should use `yt-dlp[default]`. Node is enabled as a fallback runtime.

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
| [Usage reference](docs/usage.md) | Every option, destinations, progress, retries, and exit codes |
| [Troubleshooting](docs/troubleshooting.md) | Missing tools, YouTube errors, sign-in, and interrupted files |
| [Contributing](CONTRIBUTING.md) | Local setup, checks, and contribution guidelines |
| [Architecture](docs/architecture.md) | Rust modules, downloader events, and process lifecycle |

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
