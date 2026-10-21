# Troubleshooting

[Back to README](../README.md) · [Usage reference](usage.md)

## `dtube` or `downtube` is not found

Check the installation directory is on `PATH`: the shell installer defaults to `~/.local/bin`, while the PowerShell installer uses `%LOCALAPPDATA%\Programs\downtube\bin`. Reopen your terminal after changing persistent environment settings. See [Installation](installation.md).

If you prefer a source install, run from the project directory:

```sh
cargo install --locked --path .
```

Ensure Cargo's binary directory is on your shell's `PATH`. With a standard Unix Rust installation, it is usually `~/.cargo/bin`. Open a new terminal after changing shell configuration.

You can also run from the project directory without installing:

```sh
cargo run -- 'https://www.youtube.com/playlist?list=YOUR_PLAYLIST_ID'
```

## A download dependency is missing

Check the tools individually:

```sh
yt-dlp --version
ffmpeg -version
ffprobe -version
deno --version
```

On macOS, install them with `brew install yt-dlp ffmpeg deno`. Other platforms can follow the [yt-dlp installation guide](https://github.com/yt-dlp/yt-dlp#installation) and [FFmpeg downloads](https://ffmpeg.org/download.html).

FFmpeg and ffprobe must be executable programs. Installing a Python package named `ffmpeg` does not provide them. For tools outside `PATH`, use `--yt-dlp /path/to/yt-dlp` and `--ffmpeg-location /directory/containing/ffmpeg-and-ffprobe`.

## YouTube extraction fails

Read the backend error above downtube's final message. Keep yt-dlp current through the package manager or installation method you used. For Homebrew installations:

```sh
brew upgrade yt-dlp
```

Ensure Deno is installed, or that Node is available as a fallback. yt-dlp also needs its EJS component for full YouTube support; standalone releases include it, while Python installations should include the `default` extra. See [yt-dlp's dependency documentation](https://github.com/yt-dlp/yt-dlp#dependencies).

If YouTube asks you to sign in, use a browser session that can access the playlist:

```sh
dtube 'https://www.youtube.com/playlist?list=YOUR_PLAYLIST_ID' --cookies-from-browser firefox
```

Browser sign-in does not make deleted, private, or region-restricted tracks available unless that account can access them. Playlist downloads continue after individual errors and return a nonzero status if the backend reports failure.

## The URL is rejected

Paste the full YouTube playlist URL, including `list=...`, and wrap it in quotes. Bare playlist IDs, channel pages, and standalone video URLs are not accepted. A watch URL with `list=` downloads the entire playlist.

## The destination is unexpected or cannot be created

The default follows your operating system's Music directory, which may differ from `~/Music`. Use `-o '/path/to/music'` to choose explicitly. The playlist folder is always added under that directory.

If directory creation fails, check that the parent is writable, that no path component is an ordinary file, and that the destination volume is available.

## An interrupted track is incomplete

First rerun the original command. If a crash or forced termination left an incomplete MP3 that is now being skipped, remove that specific incomplete MP3 and rerun. Keep any source or `.part` files so the backend can reuse them where possible.

On Unix, downtube signals the downloader's process group, including FFmpeg, and escalates after three seconds if needed. Other platforms terminate the direct downloader process; child-process cleanup may differ.

## There is no animated progress bar

Animations require stderr to be connected to an interactive terminal. Redirected output and `--no-progress` use progress lines. Unknown download sizes show bytes rather than a percentage; speed and ETA may be unavailable early in a download. Conversion uses a spinner rather than an encoding percentage.

## A retry created additional files

Filenames include playlist position, track title, and video ID; the parent folder follows the playlist title. Changes to those fields can create new paths. downtube does not reconcile renamed or reordered playlists against existing folders.
