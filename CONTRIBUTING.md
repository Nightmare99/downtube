# Contributing

[Back to README](README.md) · [Architecture](docs/architecture.md)

## Local setup

Use a current stable Rust toolchain. The project uses Rust edition 2024. Work from the repository root:

```sh
cargo build --locked --bins
cargo run -- --help
cargo run --bin dtube -- --help
```

Actual downloads additionally need yt-dlp, FFmpeg, ffprobe, and a supported JavaScript runtime. See the [README](README.md#quick-start). The automated test suite uses local fixtures and fake executables, so those download dependencies are not needed to run it.

## Checks

Before submitting a change, run:

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
python3 tests/installers.py
shellcheck install.sh
cargo build --locked --release --bins
```

Unit tests cover URL handling, paths, progress events, and unfinished-file cleanup. CLI integration tests cover both executable names, backend failures, output handling, dependency errors, and Ctrl+C. Integration tests in `tests/cli.rs` are currently Unix-only; the default Music directory integration check runs on macOS and Linux.

The automated suite does not exercise live YouTube extraction or FFmpeg conversion. For changes to download arguments or event templates, also check a real download with content you can access. Confirm the folder name, track order, MP3 encoding and tags, live progress, and behavior on a second run. Include those results when describing the change.

The shell installer tests use Python 3.11+ and mocked HTTPS downloads with real local archives. They cover latest/pinned installs, upgrades, paths with spaces, architecture selection, checksum rejection, and version mismatches. GitHub's Release workflow additionally tests the published installers on all supported operating systems and architectures. See [Releasing](docs/releasing.md) for version bumps and release steps.

## Making changes

Keep both command names equivalent. Preserve the distinction between download completion and an MP3 that has finished conversion and tagging. Send diagnostics and progress to stderr, and preserve useful backend errors.

Use argument arrays when spawning tools. Keep the output root separate from yt-dlp's filename template so characters such as `%` in paths remain literal. Avoid parsing yt-dlp's human-readable log output; use the structured event templates.

Add tests for new behavior or regressions where they provide useful coverage. Changes to CLI options or output behavior should also update [Usage](docs/usage.md), the README examples, and help text as appropriate.

Keep `Cargo.lock` under version control: this project distributes executable applications. Commit dependency lockfile changes alongside the dependency update. Generated builds, local tool environments, and downloaded media are ignored by Git.

## Reporting problems

Include the command shape, operating system, downtube version, dependency versions, and relevant stderr output. Mention whether the playlist is public, whether browser cookies were needed, and whether a retry changes the result. Redact private playlist links, account details, and local paths as needed. Do not include browser cookie data.

## Pull requests

Describe the problem, the resulting behavior, and the checks you ran. Call out platform-specific behavior or remaining limits so reviewers can assess the change.
