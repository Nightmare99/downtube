use std::{
    io::IsTerminal,
    time::{Duration, Instant},
};

use console::style;
use indicatif::{HumanBytes, HumanDuration, ProgressBar, ProgressDrawTarget, ProgressStyle};
use serde_json::Value;

use crate::downloader::{DONE, POSTPROCESS, PROGRESS, TRACK};

pub struct Progress {
    bar: ProgressBar,
    animated: bool,
    completed: usize,
    last_plain: Option<Instant>,
    playlist: Option<String>,
}

impl Progress {
    pub fn new(animate: bool) -> Self {
        let animated = animate && std::io::stderr().is_terminal();
        let bar = ProgressBar::new_spinner();
        if !animated {
            bar.set_draw_target(ProgressDrawTarget::hidden());
        }
        let mut this = Self {
            bar,
            animated,
            completed: 0,
            last_plain: None,
            playlist: None,
        };
        this.spinner("Reading playlist and finding audio…");
        this
    }

    fn spinner(&mut self, message: &str) {
        self.bar
            .set_style(ProgressStyle::with_template("  {spinner:.cyan} {msg}").unwrap());
        self.bar.set_message(clean(message));
        self.bar.enable_steady_tick(Duration::from_millis(90));
        if !self.animated {
            eprintln!("  {}", clean(message));
        }
    }

    fn print(&self, message: &str) {
        if self.animated {
            self.bar.println(message);
        } else {
            eprintln!("{message}");
        }
    }

    pub fn line(&mut self, line: &str) {
        if let Some(data) = event(line, TRACK) {
            let playlist = text(&data, "playlist_title");
            if self.playlist.as_deref() != Some(&playlist) {
                self.print(&format!(
                    "  {} {}",
                    style("Playlist").dim(),
                    style(&playlist).bold()
                ));
                self.playlist = Some(playlist);
            }
            let title = text(&data, "title");
            let index = data["playlist_index"]
                .as_u64()
                .map_or_else(|| "?".into(), |v| v.to_string());
            let total = data["n_entries"]
                .as_u64()
                .map_or_else(|| "?".into(), |v| v.to_string());
            self.print(&format!(
                "\n  {} {}",
                style(format!("[{index}/{total}]")).cyan(),
                style(title).bold()
            ));
            self.bar.set_prefix(format!("[{index}/{total}]"));
            self.bar.set_position(0);
            self.last_plain = None;
            self.spinner("Starting audio download…");
        } else if let Some(data) = event(line, PROGRESS) {
            let downloaded = number(&data, "downloaded_bytes").unwrap_or(0.0) as u64;
            let total =
                number(&data, "total_bytes").or_else(|| number(&data, "total_bytes_estimate"));
            let speed = number(&data, "speed")
                .map_or_else(|| "—".into(), |v| format!("{}/s", HumanBytes(v as u64)));
            let eta = number(&data, "eta").map_or_else(
                || "—".into(),
                |v| HumanDuration(Duration::from_secs_f64(v)).to_string(),
            );
            if let Some(total) = total.filter(|total| *total > 0.0) {
                self.bar.set_style(ProgressStyle::with_template("  {spinner:.cyan} {wide_bar:.cyan/blue} {percent:>3}%  {bytes}/{total_bytes}  {msg}").unwrap().progress_chars("━━─"));
                self.bar.set_length(total as u64);
                self.bar.set_position(downloaded);
            } else {
                self.bar.set_style(
                    ProgressStyle::with_template("  {spinner:.cyan} {bytes}  {msg}").unwrap(),
                );
                self.bar.set_position(downloaded);
            }
            self.bar.set_message(format!("{speed} · ETA {eta}"));
            if !self.animated
                && self
                    .last_plain
                    .is_none_or(|at| at.elapsed() >= Duration::from_secs(1))
            {
                self.print(&format!(
                    "  Downloading {} · {speed} · ETA {eta}",
                    HumanBytes(downloaded)
                ));
                self.last_plain = Some(Instant::now());
            }
            if data["status"] == "finished" {
                self.spinner("Download complete · converting to MP3…");
            }
        } else if let Some(data) = event(line, POSTPROCESS) {
            if data["status"] == "started" {
                self.spinner("Converting to MP3 / writing tags…");
            }
        } else if let Some(data) = event(line, DONE) {
            if let Some(path) = data.as_str() {
                self.completed += 1;
                self.print(&format!("  {} {}", style("✓").green(), clean(path)));
                self.spinner("Finding next track…");
            }
        } else if !line.trim().is_empty() {
            // Preserve backend diagnostics, including failed/private playlist entries.
            self.print(&format!("  {}", clean(line)));
        }
    }

    pub fn finish(&self) {
        self.bar.finish_and_clear();
    }
    pub fn completed(&self) -> usize {
        self.completed
    }
}

impl Drop for Progress {
    fn drop(&mut self) {
        self.bar.finish_and_clear();
    }
}

fn event(line: &str, marker: &str) -> Option<Value> {
    serde_json::from_str(line.strip_prefix(marker)?).ok()
}

fn number(data: &Value, key: &str) -> Option<f64> {
    data[key]
        .as_f64()
        .filter(|v| v.is_finite() && *v >= 0.0 && *v <= u64::MAX as f64 / 2.0)
}

fn text(data: &Value, key: &str) -> String {
    clean(data[key].as_str().unwrap_or("Unknown"))
}

fn clean(text: &str) -> String {
    text.chars().filter(|c| !c.is_control()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handles_unknown_sizes_missing_fields_and_malformed_events() {
        let mut ui = Progress::new(false);
        for line in [
            "__DTUBE_TRACK__{\"title\":\"A \\\"song\\\" 🎵\",\"playlist_index\":2,\"n_entries\":3,\"playlist_title\":\"Mix\"}",
            "__DTUBE_PROGRESS__{\"downloaded_bytes\":1024,\"total_bytes\":null,\"speed\":null,\"eta\":null}",
            "__DTUBE_PROGRESS__{\"downloaded_bytes\":2048,\"total_bytes_estimate\":4096,\"speed\":1024,\"eta\":2}",
            "__DTUBE_PROGRESS__{\"status\":\"finished\"}",
            "__DTUBE_POSTPROCESS__{\"status\":\"started\"}",
            "__DTUBE_DONE__not JSON",
            "WARNING: unavailable track",
            "__DTUBE_DONE__null",
        ] {
            ui.line(line);
        }
        assert_eq!(ui.completed(), 0);
        ui.line("__DTUBE_DONE__\"/Music/Mix/002 - A song.mp3\"");
        assert_eq!(ui.completed(), 1);
    }

    #[test]
    fn removes_terminal_control_characters_from_titles() {
        assert_eq!(clean("hello\n\r\t\x1b[2Jworld"), "hello[2Jworld");
        assert!(number(&serde_json::json!({"eta": -1}), "eta").is_none());
    }
}
