# Installation

[Back to README](../README.md) · [Troubleshooting](troubleshooting.md)

## Supported release binaries

| Operating system | Architectures | Archive target |
| --- | --- | --- |
| Linux | x86-64, ARM64 | `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl` |
| macOS 11 or later | Intel, Apple Silicon | `x86_64-apple-darwin`, `aarch64-apple-darwin` |
| Windows | x64, ARM64 | `x86_64-pc-windows-msvc`, `aarch64-pc-windows-msvc` |

Linux binaries link the musl runtime statically, and Windows binaries link the C runtime statically. Each platform archive includes both `downtube` and `dtube`, plus the license and documentation. Linux and macOS archives use `.tar.gz`; Windows archives use `.zip`.

## Linux and macOS

```sh
curl -fsSL https://github.com/Nightmare99/downtube/releases/latest/download/install.sh | sh
```

The installer requires `curl`, `tar`, and either `sha256sum` or `shasum`, along with standard shell utilities. It installs into `~/.local/bin` without `sudo`. On Apple Silicon it detects a Rosetta shell and selects the native ARM64 build.

Add this to your shell profile if the destination is not already on `PATH`, then open a new terminal:

```sh
export PATH="$HOME/.local/bin:$PATH"
```

For a specific version or a different destination:

```sh
curl -fsSL https://github.com/Nightmare99/downtube/releases/download/v0.2.0/install.sh \
  -o install-downtube.sh
sh install-downtube.sh --version v0.2.0 --install-dir "$HOME/bin"
rm install-downtube.sh
```

`DOWNTUBE_VERSION` and `DOWNTUBE_INSTALL_DIR` are also supported. Explicit command arguments take precedence. A pinned installer still selects the latest binary unless you pass a version or set `DOWNTUBE_VERSION`.

## Windows

Run in Windows PowerShell 5.1 or newer, or PowerShell 7:

```powershell
irm https://github.com/Nightmare99/downtube/releases/latest/download/install.ps1 | iex
```

The default destination is `%LOCALAPPDATA%\Programs\downtube\bin`. The installer updates the current session's `PATH` and your user `PATH`; other open terminals need to be reopened. No administrator privileges are needed.

For a pinned version, a custom directory, or no `PATH` changes:

```powershell
$installer = irm https://github.com/Nightmare99/downtube/releases/download/v0.2.0/install.ps1
& ([scriptblock]::Create($installer)) -Version v0.2.0 -InstallDir "$env:LOCALAPPDATA\downtube\bin" -NoModifyPath
```

`DOWNTUBE_VERSION` and `DOWNTUBE_INSTALL_DIR` environment variables also provide defaults. If your machine's execution policy blocks a saved script, the script-block invocation above avoids changing that policy.

## Runtime dependencies

The installers do not install external download dependencies. Before downloading, make these programs available on `PATH`:

| Dependency | Purpose |
| --- | --- |
| [yt-dlp](https://github.com/yt-dlp/yt-dlp#installation) | Extract playlists and download the selected audio |
| [FFmpeg and ffprobe](https://ffmpeg.org/download.html) | Inspect, convert, and tag MP3s |
| [Deno](https://docs.deno.com/runtime/getting_started/installation/) or Node | Run yt-dlp's YouTube JavaScript support |

On macOS with Homebrew:

```sh
brew install yt-dlp ffmpeg deno
```

On Linux and Windows, use the linked installation guides or your package manager. FFmpeg and ffprobe must be native executable programs. yt-dlp's EJS component is included in official standalone builds; Python installations should use `yt-dlp[default]`. downtube enables Node as a fallback when Deno is absent.

You can point to external tools using `--yt-dlp /path/to/yt-dlp` and `--ffmpeg-location /directory/containing/ffmpeg-and-ffprobe`. Keep these dependencies current as YouTube changes.

## Checksums and manual installation

Installers download an archive and `SHA256SUMS` from the same versioned release, verify the archive before extracting it, then check the binary's version before installation. A checksum mismatch stops installation. Checksums detect damaged or mismatched downloads; they are not independent release signatures.

For manual installation, download your archive and `SHA256SUMS` from [Releases](https://github.com/Nightmare99/downtube/releases), verify the archive's SHA-256 hash against its matching manifest entry, extract it, and place both binaries on `PATH`. You can download and inspect either installer before executing it.

## Upgrading and removal

Run the installer again to upgrade to the latest release, or pass a version to install that release. Close running downtube processes before upgrading on Windows. Downloaded playlists are not changed by installation or removal.

On Linux and macOS, remove both binaries from your chosen install directory. For the default location:

```sh
rm "$HOME/.local/bin/downtube" "$HOME/.local/bin/dtube"
```

On Windows, delete `%LOCALAPPDATA%\Programs\downtube` and remove its `bin` directory from your user `PATH` through Windows Environment Variables settings. Use your custom directory if you overrode the default.

## Build from source

Install a current stable [Rust toolchain](https://rustup.rs/), then:

```sh
git clone https://github.com/Nightmare99/downtube.git
cd downtube
cargo install --locked --path .
```

Both commands are installed into Cargo's binary directory, usually `~/.cargo/bin`, which must be on `PATH`. Source installs still require the runtime dependencies above. If you installed with Cargo, remove the package with `cargo uninstall downtube`.
