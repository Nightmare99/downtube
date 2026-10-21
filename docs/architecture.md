# Architecture

[Back to README](../README.md) · [Contributing](../CONTRIBUTING.md)

downtube owns the command interface, destination selection, progress display, and process lifecycle. It delegates YouTube extraction and transfer to yt-dlp, which invokes FFmpeg for audio conversion and metadata.

```text
Playlist URL
    │
    ▼
Rust CLI ── validate URL, check dependencies, resolve destination
    │
    ▼
yt-dlp ─── extract playlist and select/download audio
    │
    ▼
FFmpeg ─── convert to MP3 and write tags
    │
    ▼
<Music directory>/<playlist>/<number> - <title> [<id>].mp3
```

## Modules

| File | Responsibility |
| --- | --- |
| `src/main.rs`, `src/bin/dtube.rs` | Equivalent entry points calling `downtube::run()` |
| `src/lib.rs` | Orchestration, signal handler, summary, and exit status |
| `src/cli.rs` | Clap arguments, YouTube URL normalization, home expansion, and Music directory resolution |
| `src/downloader.rs` | Dependency checks, yt-dlp command construction, and unfinished-MP3 tracking |
| `src/process.rs` | Subprocess ownership, pipe readers, cancellation, and reaping |
| `src/progress.rs` | Structured-event parsing and terminal or plain-text presentation |
| `tests/cli.rs` | CLI integration fixtures and checks |

## Backend contract

The command disables personal yt-dlp configuration and explicitly sets playlist handling, format selection, MP3 bitrate, metadata, resume behavior, filenames, and progress templates. The destination root is passed through `--paths`; only the relative folder and filename layout is a template. No shell is used to execute the downloader.

yt-dlp emits newline-delimited events using `--print` and `--progress-template`:

| Marker | Payload | Purpose |
| --- | --- | --- |
| `__DTUBE_TRACK__` | JSON object with title, playlist position/count, playlist title, and source filename | Start track display and identify prospective MP3 output |
| `__DTUBE_PROGRESS__` | JSON download progress object | Update bytes, total or estimated size, speed, and ETA |
| `__DTUBE_POSTPROCESS__` | JSON postprocessing progress object | Display conversion/tagging activity |
| `__DTUBE_DONE__` | JSON string containing the final path | Confirm a ready MP3 and release it from unfinished-file tracking |

Completion is counted at `after_move`, after conversion and postprocessing. Unrecognized lines, including backend diagnostics, are shown to the user. Malformed events do not count as completed files. Terminal control characters are stripped from rendered event text.

## Process lifecycle

Separate reader threads drain stdout and stderr into a bounded channel. The main loop consumes events, checks process status, and observes a cancellation flag set by the Ctrl+C handler. Reader or process errors propagate through a guard that stops and reaps the downloader.

On Unix, the downloader gets its own process group. Cancellation sends SIGINT to that group and escalates to SIGKILL after three seconds. Other platforms use direct child termination. The loop drains remaining output before returning.

`PendingFiles` tracks prospective MP3 paths inside the destination that appear to be new in this run, using existence and creation-time checks. Completed paths are removed from the pending set. On drop, remaining pending MP3s are removed where possible so they are not mistaken for completed downloads on retry. Cleanup is best effort and does not survive forced process termination or a machine crash.

## Persistence and limits

The tool has no library database or download archive. yt-dlp's existing-file and resume behavior determines what can be reused. Playlist titles and indices form part of each path, so renamed or reordered playlists can create additional files. Available source formats and access depend on YouTube and yt-dlp; conversion bitrate cannot increase source fidelity.
