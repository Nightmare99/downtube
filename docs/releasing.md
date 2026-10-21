# Releasing

[Back to README](../README.md) · [Changelog](../CHANGELOG.md)

## Versioning

The package follows stable semantic versions: `MAJOR.MINOR.PATCH`. Use a patch for compatible fixes, a minor for new functionality, and a major for breaking changes once the project reaches 1.0. While on 0.x, document incompatible changes explicitly.

`Cargo.toml` is the version source. `Cargo.lock`, the dated changelog section, and the Git tag must agree. Tags use `v`, e.g. Cargo `0.2.0` corresponds to Git tag `v0.2.0`. Both binary names report the same package version.

The maintenance scripts use Python 3.11 or newer, which is not needed to install or run downtube.

## Cut a release

Start on `main` with a clean working tree and update from the remote. To prepare the next version:

```sh
python3 scripts/version.py --bump 0.3.0
```

The helper updates both Cargo files and rejects an unchanged or lower version. Add a `## [0.3.0] - YYYY-MM-DD` section to `CHANGELOG.md` with the actual date and release notes. Then validate:

```sh
python3 scripts/version.py --check-tag v0.3.0
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
python3 tests/installers.py
shellcheck install.sh
```

Commit the prepared release, push `main`, and push an annotated version tag:

```sh
git add Cargo.toml Cargo.lock CHANGELOG.md
git commit -m "Release v0.3.0"
git push origin main
git tag -a v0.3.0 -m "downtube v0.3.0"
git push origin v0.3.0
```

The examples use `0.3.0`; substitute the version you are actually releasing. Include any other release changes in the commit before tagging. Never move a published version tag to different code; issue a new version for fixes.

## GitHub Actions

[The Release workflow](../.github/workflows/release.yml) runs when a `v*` tag is pushed:

1. Verify the existing Git tag, Cargo manifest, lockfile, and changelog version. Run formatting, Clippy, Rust tests, shell installer tests, and ShellCheck.
2. Test and build native binaries on six runners: Linux, macOS, and Windows, each on x86-64 and ARM64.
3. Package both commands and documentation in `downtube-v<VERSION>-<TARGET>.tar.gz` or `.zip` files. Check the built commands' version and help output. On Windows, also test the PowerShell installer offline with the packaged binaries, including checksum rejection and pinned/latest versions.
4. Collect all six archives, `install.sh`, and `install.ps1`. Generate `SHA256SUMS` for every asset and publish a GitHub Release with that version's changelog notes. Only this job has repository write permission.
5. On all six native runners, download the published installer and install the published archive into a fresh directory containing spaces. Verify both installed commands' version and help output.

The final installation jobs run after publication. A failure there leaves the release published and marks the workflow failed; inspect the logs before announcing it. Fixes to published code or assets should use a new version. A transient download failure can be retried by rerunning failed jobs.

If a tag was pushed before the workflow was available, or validation failed transiently, dispatch the workflow for that existing tag:

```sh
gh workflow run release.yml --ref main -f tag=v0.3.0
gh run list --workflow release.yml
```

The workflow checks out the tagged code. It does not bump versions, create tags, or overwrite an existing release. Manual dispatch cannot add new workflow code to an older tag.

## Verify a published release

Confirm the Release workflow, including all six installation jobs, is green. Inspect its assets:

```sh
gh release view v0.3.0
```

Also run the documented installer on your own machine and check `downtube --version`, `dtube --version`, and `dtube --help`. For downloader changes, check a playlist download or a controlled local media fixture with the actual release binary and verify the resulting MP3 using ffprobe.

The normal [CI workflow](../.github/workflows/ci.yml) runs checks on pushes to `main` and pull requests; it does not publish releases.
