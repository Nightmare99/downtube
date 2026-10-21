#!/bin/sh
# Install a published downtube release without Rust or administrator privileges.
set -eu

fail() { printf 'downtube: %s\n' "$*" >&2; exit 1; }

main() {
    version=${DOWNTUBE_VERSION:-latest}
    install_dir=${DOWNTUBE_INSTALL_DIR:-"$HOME/.local/bin"}
    while [ "$#" -gt 0 ]; do
        case "$1" in
            --version) [ "$#" -ge 2 ] || fail '--version needs a value'; version=$2; shift 2 ;;
            --install-dir) [ "$#" -ge 2 ] || fail '--install-dir needs a path'; install_dir=$2; shift 2 ;;
            -h|--help)
                printf '%s\n' 'Usage: install.sh [--version vX.Y.Z] [--install-dir PATH]' \
                    'Defaults: latest release, ~/.local/bin. No sudo or Rust required.'
                return ;;
            *) fail "unknown option: $1" ;;
        esac
    done
    [ -n "$install_dir" ] || fail 'installation directory cannot be empty'
    case "$install_dir" in /*) ;; *) install_dir="$PWD/$install_dir" ;; esac
    for tool in curl tar mktemp awk; do
        command -v "$tool" >/dev/null 2>&1 || fail "required tool not found: $tool"
    done
    if command -v sha256sum >/dev/null 2>&1; then
        hash_tool=sha256sum
    elif command -v shasum >/dev/null 2>&1; then
        hash_tool=shasum
    else
        fail 'SHA-256 verification requires sha256sum or shasum'
    fi
    case "$(uname -s)" in
        Linux) platform=unknown-linux-musl ;;
        Darwin) platform=apple-darwin ;;
        *) fail 'supported operating systems are Linux and macOS; use install.ps1 on Windows' ;;
    esac
    architecture=$(uname -m)
    # Prefer the native Apple Silicon binary when called from a Rosetta shell.
    if [ "$platform" = apple-darwin ] && [ "$architecture" = x86_64 ] &&
        [ "$(sysctl -in sysctl.proc_translated 2>/dev/null || true)" = 1 ]; then
        architecture=arm64
    fi
    case "$architecture" in
        x86_64|amd64) architecture=x86_64 ;;
        aarch64|arm64) architecture=aarch64 ;;
        *) fail "unsupported CPU architecture: $architecture (need x86_64 or ARM64)" ;;
    esac

    repo=https://github.com/Nightmare99/downtube
    if [ "$version" = latest ]; then
        release_url=$(curl --proto '=https' --tlsv1.2 -fsSL --retry 3 \
            -o /dev/null -w '%{url_effective}' "$repo/releases/latest") || fail 'could not find the latest release'
        version=${release_url##*/}
    fi
    case "$version" in v*) ;; *) version="v$version" ;; esac
    printf '%s\n' "$version" | LC_ALL=C grep -Eq '^v[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$' ||
        fail 'invalid version; expected vX.Y.Z or vX.Y.Z-prerelease'

    asset="downtube-$version-$architecture-$platform.tar.gz"
    download="$repo/releases/download/$version"
    scratch=$(mktemp -d "${TMPDIR:-/tmp}/downtube-install.XXXXXXXX") || fail 'could not create temporary directory'
    trap 'rm -rf "$scratch"' EXIT
    trap 'exit 130' INT
    trap 'exit 143' TERM HUP
    printf 'Downloading downtube %s (%s-%s)…\n' "$version" "$architecture" "$platform"
    curl --proto '=https' --tlsv1.2 -fsSL --retry 3 "$download/$asset" -o "$scratch/$asset" || fail 'release download failed'
    curl --proto '=https' --tlsv1.2 -fsSL --retry 3 "$download/SHA256SUMS" -o "$scratch/SHA256SUMS" || fail 'checksum download failed'
    expected=$(awk -v file="$asset" '$2 == file { print $1 }' "$scratch/SHA256SUMS")
    printf '%s\n' "$expected" | LC_ALL=C grep -Eq '^[0-9a-f]{64}$' || fail 'release checksum missing or malformed'
    if [ "$hash_tool" = sha256sum ]; then
        actual=$(sha256sum "$scratch/$asset" | awk '{ print $1 }')
    else
        actual=$(shasum -a 256 "$scratch/$asset" | awk '{ print $1 }')
    fi
    [ "$actual" = "$expected" ] || fail 'checksum mismatch; nothing was installed'
    tar -xzf "$scratch/$asset" -C "$scratch" downtube dtube
    if [ ! -f "$scratch/downtube" ] || [ ! -f "$scratch/dtube" ]; then
        fail 'release is missing binaries'
    fi
    chmod 755 "$scratch/downtube" "$scratch/dtube"
    reported=$("$scratch/downtube" --version) || fail 'downloaded binary cannot run on this system'
    [ "$reported" = "downtube ${version#v}" ] || fail 'binary version does not match the release'
    mkdir -p "$install_dir"
    # Rename within the destination so upgrades can replace a running executable.
    for binary in downtube dtube; do
        staged=$(mktemp "$install_dir/.$binary.XXXXXXXX") || fail 'destination is not writable'
        if ! cp "$scratch/$binary" "$staged" || ! chmod 755 "$staged" || ! mv -f "$staged" "$install_dir/$binary"; then
            rm -f "$staged"
            fail "could not install $binary"
        fi
    done
    printf 'Installed %s and dtube to %s\n' "$reported" "$install_dir"
    case ":$PATH:" in
        *":$install_dir:"*) ;;
        *) printf '\nAdd this directory to PATH in your shell profile:\n  %s\n' "$install_dir" ;;
    esac
    if ! command -v yt-dlp >/dev/null 2>&1 || ! command -v ffmpeg >/dev/null 2>&1 || ! command -v ffprobe >/dev/null 2>&1; then
        printf '\nDownloads also need yt-dlp, FFmpeg (including ffprobe), and Deno or Node.\n'
        if [ "$platform" = apple-darwin ]; then
            printf 'Install dependencies with: brew install yt-dlp ffmpeg deno\n'
        else
            printf 'Dependency setup: https://github.com/Nightmare99/downtube/blob/main/docs/installation.md\n'
        fi
    fi
    printf 'Try: dtube --help\n'
}

# Download the entire script before executing when used through curl | sh.
main "$@"
