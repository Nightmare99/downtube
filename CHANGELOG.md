# Changelog

Versions follow [Semantic Versioning](https://semver.org/). Git tags use a `v` prefix, and the Cargo manifest and lockfile carry the matching version without the prefix.

## [0.2.0] - 2026-10-21

- Publish native release binaries for Linux, macOS, and Windows on x86-64 and ARM64.
- Add checksum-verifying shell and PowerShell installers with version and destination overrides.
- Automate release validation, packaging, publishing, and installation smoke checks in GitHub Actions.
- Document binary installation, upgrades, removal, and the release/versioning process.

## [0.1.0] - 2026-10-21

- Initial source release with `downtube` and `dtube` commands.
- Download YouTube playlists to the system Music folder or a custom destination.
- Select the highest-bitrate available audio and convert it to tagged 320 kbps MP3s.
- Display live progress and support download retries and interruption handling.
