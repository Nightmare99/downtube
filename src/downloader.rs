use std::{
    path::Path,
    process::{Command, Stdio},
};

use anyhow::{Context, Result, bail};

use crate::cli::Cli;

pub const TRACK: &str = "__DTUBE_TRACK__";
pub const PROGRESS: &str = "__DTUBE_PROGRESS__";
pub const POSTPROCESS: &str = "__DTUBE_POSTPROCESS__";
pub const DONE: &str = "__DTUBE_DONE__";

pub fn check_dependencies(cli: &Cli) -> Result<()> {
    check(&cli.yt_dlp, "--version", "yt-dlp")?;
    for tool in ["ffmpeg", "ffprobe"] {
        let executable = cli.ffmpeg_location.as_ref().map_or_else(
            || tool.into(),
            |dir| dir.join(format!("{tool}{}", std::env::consts::EXE_SUFFIX)),
        );
        check(&executable, "-version", tool)?;
    }
    Ok(())
}

fn check(executable: &Path, flag: &str, name: &str) -> Result<()> {
    let status = Command::new(executable)
        .arg(flag)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .with_context(|| format!("{name} was not found or could not be started ({}).\nInstall dependencies: macOS: brew install yt-dlp ffmpeg deno\nOther platforms: https://github.com/yt-dlp/yt-dlp#installation", executable.display()))?;
    if !status.success() {
        bail!("{name} failed its version check; check your installation");
    }
    Ok(())
}

pub fn command(cli: &Cli, output: &Path) -> Command {
    let mut command = Command::new(&cli.yt_dlp);
    command.args([
        "--ignore-config", "--yes-playlist", "--no-abort-on-error",
        "--format", "bestaudio/best", "--format-sort", "abr,asr", "--format-sort-force",
        "--extract-audio", "--audio-format", "mp3", "--audio-quality", "320K",
        "--embed-metadata", "--continue", "--no-overwrites", "--windows-filenames",
        "--no-simulate", "--quiet", "--progress", "--newline", "--no-colors",
        "--progress-delta", "0.2", "--socket-timeout", "30",
        // Enable Node as a fallback when the recommended Deno runtime is absent.
        "--js-runtimes", "node",
        "--output", "%(playlist_title,playlist_id|Playlist).150B/%(playlist_index)03d - %(title).150B [%(id)s].%(ext)s",
        "--print", "before_dl:__DTUBE_TRACK__%(.{title,playlist_index,n_entries,playlist_title,_filename})j",
        "--print", "after_move:__DTUBE_DONE__%(filepath)j",
        "--progress-template", "download:__DTUBE_PROGRESS__%(progress)j",
        "--progress-template", "postprocess:__DTUBE_POSTPROCESS__%(progress)j",
    ]);
    // Keep the user-supplied root out of the output template: '%' is literal here.
    let mut home = std::ffi::OsString::from("home:");
    home.push(output);
    command.arg("--paths").arg(home);
    if let Some(browser) = &cli.cookies_from_browser {
        command.arg("--cookies-from-browser").arg(browser);
    }
    if let Some(location) = &cli.ffmpeg_location {
        command.arg("--ffmpeg-location").arg(location);
    }
    command.arg("--").arg(&cli.url);
    command
}

/// Remove unfinished MP3s on interruption or conversion failure. Otherwise a
/// subsequent run could mistake FFmpeg's partially written output for a song.
pub struct PendingFiles {
    root: std::path::PathBuf,
    files: std::collections::HashSet<std::path::PathBuf>,
    existing: std::collections::HashSet<String>,
    protected: Vec<String>,
}

impl PendingFiles {
    pub fn new(root: &Path) -> Self {
        let mut state = Self {
            root: root.to_owned(),
            files: Default::default(),
            existing: Default::default(),
            protected: Vec::new(),
        };
        // Snapshot before starting the downloader: filesystem timestamps can be
        // coarser than the interval between launching it and receiving an event.
        let mut directories = vec![root.to_owned()];
        while let Some(directory) = directories.pop() {
            let Ok(entries) = std::fs::read_dir(&directory) else {
                state.protected.push(path_key(&directory));
                continue;
            };
            for entry in entries {
                let Ok(entry) = entry else {
                    state.protected.push(path_key(&directory));
                    continue;
                };
                let path = entry.path();
                match entry.file_type() {
                    Ok(kind) if kind.is_symlink() => state.protected.push(path_key(&path)),
                    Ok(kind) if kind.is_dir() => directories.push(path),
                    Ok(_) => {
                        state.existing.insert(path_key(&path));
                    }
                    Err(_) => state.protected.push(path_key(&path)),
                }
            }
        }
        state
    }

    pub fn observe(&mut self, line: &str) {
        if let Some(json) = line.strip_prefix(TRACK)
            && let Ok(data) = serde_json::from_str::<serde_json::Value>(json)
            && let Some(filename) = data["_filename"].as_str()
        {
            let mp3 = Path::new(filename).with_extension("mp3");
            let key = path_key(&mp3);
            let protected = self
                .protected
                .iter()
                .any(|prefix| key == *prefix || key.starts_with(&format!("{prefix}/")));
            if mp3.starts_with(&self.root) && !self.existing.contains(&key) && !protected {
                self.files.insert(mp3);
            }
        } else if let Some(json) = line.strip_prefix(DONE)
            && let Ok(filename) = serde_json::from_str::<String>(json)
        {
            self.files.remove(Path::new(&filename));
        }
    }
}

// Case-folding conservatively protects existing files on case-insensitive
// filesystems too. Collisions only skip cleanup; they never delete extra files.
fn path_key(path: &Path) -> String {
    path.to_string_lossy()
        .replace('\\', "/")
        .trim_end_matches('/')
        .to_lowercase()
}

impl Drop for PendingFiles {
    fn drop(&mut self) {
        for file in &self.files {
            if let Err(error) = std::fs::remove_file(file)
                && error.kind() != std::io::ErrorKind::NotFound
            {
                eprintln!(
                    "Could not remove unfinished MP3 {}: {error}",
                    file.display()
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_partial_mp3_but_preserves_finished_and_existing_files() {
        let temp = tempfile::tempdir().unwrap();
        let partial = temp.path().join("partial.mp3");
        let finished = temp.path().join("finished.mp3");
        let existing = temp.path().join("existing.mp3");
        std::fs::write(&existing, "existing song").unwrap();
        {
            let mut pending = PendingFiles::new(temp.path());
            for path in [&partial, &finished, &existing] {
                let info = serde_json::json!({"_filename": path.with_extension("webm")});
                pending.observe(&format!("{TRACK}{info}"));
            }
            std::fs::write(&partial, "partial audio").unwrap();
            std::fs::write(&finished, "finished audio").unwrap();
            pending.observe(&format!(
                "{DONE}{}",
                serde_json::to_string(&finished).unwrap()
            ));
        }
        assert!(!partial.exists());
        assert_eq!(std::fs::read_to_string(finished).unwrap(), "finished audio");
        assert_eq!(std::fs::read_to_string(existing).unwrap(), "existing song");
    }

    #[test]
    fn cleans_new_mp3_even_when_output_arrives_before_the_track_event() {
        let temp = tempfile::tempdir().unwrap();
        let mp3 = temp.path().join("fast.mp3");
        {
            let mut pending = PendingFiles::new(temp.path());
            std::fs::write(&mp3, "partial audio").unwrap();
            let info = serde_json::json!({"_filename": mp3.with_extension("webm")});
            pending.observe(&format!("{TRACK}{info}"));
        }
        assert!(!mp3.exists());
    }

    #[test]
    #[cfg(unix)]
    fn preserves_files_behind_symlinked_directories() {
        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let mp3 = outside.path().join("existing.mp3");
        std::fs::write(&mp3, "existing audio").unwrap();
        let link = root.path().join("linked");
        std::os::unix::fs::symlink(outside.path(), &link).unwrap();
        {
            let mut pending = PendingFiles::new(root.path());
            let info = serde_json::json!({"_filename": link.join("existing.webm")});
            pending.observe(&format!("{TRACK}{info}"));
        }
        assert!(mp3.exists());
    }
}
