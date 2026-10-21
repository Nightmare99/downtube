# Usage reference

[Back to README](../README.md)

```text
dtube [OPTIONS] <PLAYLIST_URL>
downtube [OPTIONS] <PLAYLIST_URL>
```

Both names run the same program. One invocation downloads one playlist.

## Accepted URLs

Pass a full HTTP or HTTPS YouTube URL containing a non-empty `list=` parameter:

```sh
dtube 'https://www.youtube.com/playlist?list=YOUR_PLAYLIST_ID'
dtube 'https://music.youtube.com/playlist?list=YOUR_PLAYLIST_ID'
dtube 'https://www.youtube.com/watch?v=VIDEO_ID&list=YOUR_PLAYLIST_ID'
dtube 'https://youtu.be/VIDEO_ID?list=YOUR_PLAYLIST_ID'
```

Video URLs containing `list=` are normalized to the **whole playlist**. Video selection, tracking parameters, and starting indices in the URL are discarded. A video URL without a playlist ID is rejected.

## Options

| Option | Purpose | Default |
| --- | --- | --- |
| `-o, --output <PATH>` | Parent directory for the playlist folder; `--out` is an alias | System Music directory, falling back to `~/Music` |
| `--cookies-from-browser <BROWSER>` | Read browser cookies for a playlist requiring sign-in, e.g. `firefox` or `chrome` | No browser cookies |
| `--yt-dlp <PATH>` | Use a specific yt-dlp executable | `yt-dlp` on `PATH` |
| `--ffmpeg-location <DIR>` | Directory containing **both** ffmpeg and ffprobe | Find both tools on `PATH` |
| `--no-progress` | Use readable progress lines instead of animated bars | Animate when stderr is a terminal |
| `-h, --help` | Show help; `--help` includes more detail | — |
| `-V, --version` | Print the package version | — |

`--no-progress` still reports download progress; it changes the presentation.

## Destinations and filenames

```sh
dtube 'https://www.youtube.com/playlist?list=YOUR_PLAYLIST_ID' --output './My Music'
```

Relative paths are resolved from the current directory. Missing destination directories are created. A leading `~` expands to your home directory.

The output layout is:

```text
<destination>/<playlist title>/<playlist index> - <track title> [<video ID>].mp3
```

Playlist indices have at least three digits. The folder uses the playlist ID when its title is unavailable, and `Playlist` if neither is present. Playlist and track titles are each limited to 150 bytes before filesystem sanitization. Unicode is preserved; incompatible characters are replaced.

Paths follow playlist titles and positions rather than a separate music library database. Playlists with matching folder names share a directory. Renamed or reordered playlists may produce additional files when downloaded again.

## Audio and metadata

downtube selects `bestaudio/best`, sorting by audio bitrate and then sample rate. It prefers an audio-only stream and falls back to a combined audio/video stream if necessary. FFmpeg converts the selected YouTube audio to MP3 at 320 kbps and embeds available metadata.

Source quality limits the result. A larger MP3 bitrate does not improve the original recording. Bitrate, format, naming, and metadata behavior are fixed in this version; there are no flags to change them. Personal yt-dlp configuration files are ignored.

## Progress and logging

Interactive terminals display the current track and playlist position, then a live download bar with bytes, speed, and ETA. When size is unknown, the display shows downloaded bytes without a percentage. Conversion and tagging use a spinner.

All status output and backend diagnostics go to **stderr**. Redirect stderr to capture a readable log:

```sh
dtube 'https://www.youtube.com/playlist?list=YOUR_PLAYLIST_ID' 2>download.log
```

Use `--no-progress` for readable lines even in an interactive terminal.

## Stopping and retrying

Press Ctrl+C, then repeat the same command with the same destination. Download fragments are resumed where supported; conversion may restart. Completed MP3s are preserved. The final “MP3(s) ready” count includes existing files confirmed by yt-dlp.

During normal shutdown, downtube attempts to remove unfinished MP3s it identified as new in this run. Downloaded source files and resume data can remain for a retry. Forced termination or a machine crash can bypass this cleanup; see [Troubleshooting](troubleshooting.md#an-interrupted-track-is-incomplete).

Unavailable tracks are reported while other playlist entries continue. Rerun the command to retry failures.

## Exit codes

| Code | Meaning |
| --- | --- |
| `0` | The downloader completed successfully and confirmed at least one MP3 |
| `1` | Dependency, filesystem, download, conversion, or empty-playlist error; some files may have completed |
| `2` | Invalid command-line arguments or playlist URL |
| `130` | Download interrupted through downtube's Ctrl+C handler |
