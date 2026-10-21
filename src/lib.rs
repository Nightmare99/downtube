mod cli;
mod downloader;
mod process;
mod progress;

use std::{
    process::ExitCode,
    sync::{Arc, atomic::AtomicBool},
};

use anyhow::Result;
use clap::Parser;
use console::style;

pub fn run() -> ExitCode {
    let cli = cli::Cli::parse();
    match execute(cli) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::from(130),
        Err(error) => {
            eprintln!("{} {error:#}", style("error:").red().bold());
            ExitCode::FAILURE
        }
    }
}

fn execute(cli: cli::Cli) -> Result<bool> {
    downloader::check_dependencies(&cli)?;
    let output = cli::output_dir(cli.output.clone())?;
    let cancelled = Arc::new(AtomicBool::new(false));
    let signal = cancelled.clone();
    ctrlc::set_handler(move || signal.store(true, std::sync::atomic::Ordering::SeqCst))?;
    eprintln!(
        "\n  {}  {}",
        style("downtube").cyan().bold(),
        style("playlists → MP3 · 320 kbps").dim()
    );
    eprintln!("  {} {}\n", style("Save to").dim(), output.display());
    let mut ui = progress::Progress::new(!cli.no_progress);
    let mut pending = downloader::PendingFiles::new(&output);
    let status = process::run(downloader::command(&cli, &output), cancelled, |line| {
        pending.observe(line);
        ui.line(line)
    })?;
    ui.finish();
    drop(pending);
    if status.is_none() {
        eprintln!("\n  Interrupted. Run the same command to resume.");
        return Ok(false);
    }
    if !status.unwrap().success() {
        anyhow::bail!(
            "playlist incomplete ({} MP3(s) ready). See the messages above; run the same command to retry. If YouTube extraction fails, update yt-dlp and ensure Deno or Node is installed.",
            ui.completed()
        );
    }
    if ui.completed() == 0 {
        anyhow::bail!("no MP3s were produced; the playlist may be empty or unavailable");
    }
    eprintln!(
        "\n  {} {} MP3(s) ready in {}\n",
        style("✓").green().bold(),
        ui.completed(),
        output.display()
    );
    Ok(true)
}
