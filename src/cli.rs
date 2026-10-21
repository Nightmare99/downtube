use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use clap::Parser;
use url::Url;

#[derive(Debug, Parser)]
#[command(
    version,
    about = "YouTube playlists, straight to your music folder.",
    long_about = "Download a YouTube playlist as numbered, 320 kbps MP3s in a folder named after the playlist. Selects the highest-bitrate available audio source. Requires yt-dlp, FFmpeg and ffprobe on PATH.",
    after_help = "EXAMPLES:\n  downtube 'https://www.youtube.com/playlist?list=PL…'\n  dtube 'https://www.youtube.com/playlist?list=PL…' -o ~/Downloads\n  dtube 'https://music.youtube.com/playlist?list=PL…' --cookies-from-browser firefox\n\nInterrupted downloads resume when you run the same command again."
)]
pub struct Cli {
    /// YouTube playlist URL (quote it if it contains &)
    #[arg(value_name = "PLAYLIST_URL", value_parser = playlist_url)]
    pub url: String,

    /// Parent output directory [default: your system Music folder, or ~/Music]
    #[arg(short, long = "output", visible_alias = "out", value_name = "PATH")]
    pub output: Option<PathBuf>,

    /// Browser cookies for playlists that require sign-in (e.g. firefox, chrome)
    #[arg(long, value_name = "BROWSER")]
    pub cookies_from_browser: Option<String>,

    /// Path to a yt-dlp executable
    #[arg(long, default_value = "yt-dlp", value_name = "PATH")]
    pub yt_dlp: PathBuf,

    /// Directory containing ffmpeg and ffprobe
    #[arg(long, value_name = "DIR")]
    pub ffmpeg_location: Option<PathBuf>,

    /// Print readable progress lines instead of animated progress bars
    #[arg(long)]
    pub no_progress: bool,
}

fn playlist_url(raw: &str) -> Result<String, String> {
    let url = Url::parse(raw).map_err(|_| "expected a full YouTube playlist URL".to_owned())?;
    let host = url.host_str().unwrap_or_default();
    let youtube = host == "youtube.com" || host.ends_with(".youtube.com") || host == "youtu.be";
    if !matches!(url.scheme(), "http" | "https") || !youtube {
        return Err("expected an http(s) URL on youtube.com, music.youtube.com or youtu.be".into());
    }
    if !url.username().is_empty() || url.password().is_some() || url.port().is_some() {
        return Err("playlist URLs cannot contain credentials or custom ports".into());
    }
    let list = url
        .query_pairs()
        .find(|(key, _)| key == "list")
        .map(|(_, value)| value.into_owned())
        .filter(|value| {
            !value.is_empty()
                && value
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        })
        .ok_or_else(|| "the URL must contain a valid, non-empty list= playlist ID".to_owned())?;
    // A watch URL with list= should download the whole playlist, not start at a video.
    let mut canonical = Url::parse("https://www.youtube.com/playlist").unwrap();
    canonical.query_pairs_mut().append_pair("list", &list);
    Ok(canonical.into())
}

pub fn output_dir(explicit: Option<PathBuf>) -> Result<PathBuf> {
    let path = match explicit {
        Some(path) if path.as_os_str().is_empty() => bail!("output path cannot be empty"),
        Some(path) => expand_home(path)?,
        None => dirs::audio_dir()
            .or_else(|| dirs::home_dir().map(|home| home.join("Music")))
            .context("could not find your Music folder; specify one with --output PATH")?,
    };
    std::fs::create_dir_all(&path)
        .with_context(|| format!("could not create output directory {}", path.display()))?;
    path.canonicalize()
        .with_context(|| format!("could not resolve output directory {}", path.display()))
}

fn expand_home(path: PathBuf) -> Result<PathBuf> {
    if path == std::path::Path::new("~") {
        return dirs::home_dir().context("could not find your home directory");
    }
    if let Ok(suffix) = path.strip_prefix("~") {
        return Ok(dirs::home_dir()
            .context("could not find your home directory")?
            .join(suffix));
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonicalizes_playlist_and_watch_urls() {
        for input in [
            "https://www.youtube.com/playlist?list=PL_abc-123",
            "https://music.youtube.com/playlist?list=PL_abc-123&si=tracking",
            "https://youtu.be/video?list=PL_abc-123",
            "https://www.youtube.com/watch?v=video&list=PL_abc-123&index=5",
        ] {
            assert_eq!(
                playlist_url(input).unwrap(),
                "https://www.youtube.com/playlist?list=PL_abc-123"
            );
        }
    }

    #[test]
    fn rejects_non_playlists_and_lookalike_hosts() {
        for input in [
            "--exec=bad",
            "https://youtube.com.evil.test/playlist?list=PL123",
            "https://notyoutube.com/playlist?list=PL123",
            "file:///playlist?list=PL123",
            "https://youtube.com/watch?v=abc",
            "https://youtube.com/playlist?list=",
            "https://youtube.com/playlist?list=../escape",
            "https://user@youtube.com/playlist?list=PL123",
        ] {
            assert!(playlist_url(input).is_err(), "accepted {input}");
        }
    }

    #[test]
    fn creates_nested_output_and_rejects_files() {
        let temp = tempfile::tempdir().unwrap();
        let nested = temp.path().join("Music with spaces/Playlists");
        assert_eq!(
            output_dir(Some(nested.clone())).unwrap(),
            nested.canonicalize().unwrap()
        );
        let file = temp.path().join("file");
        std::fs::write(&file, "data").unwrap();
        assert!(output_dir(Some(file)).is_err());
    }
}
