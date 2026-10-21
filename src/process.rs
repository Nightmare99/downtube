use std::{
    io::{BufRead, BufReader, Read},
    process::{Child, Command, ExitStatus, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::{Duration, Instant},
};

use anyhow::{Context, Result};

struct ManagedChild {
    child: Child,
    reaped: bool,
}

impl ManagedChild {
    fn stop(&mut self, force: bool) {
        #[cfg(unix)]
        {
            // The downloader has its own process group, including FFmpeg children.
            let signal = if force { libc::SIGKILL } else { libc::SIGINT };
            // SAFETY: a negative child PID targets only the group created below.
            unsafe {
                libc::kill(-(self.child.id() as i32), signal);
            }
        }
        #[cfg(not(unix))]
        {
            let _ = force;
            let _ = self.child.kill();
        }
    }
}

impl Drop for ManagedChild {
    fn drop(&mut self) {
        if !self.reaped {
            self.stop(true);
            let _ = self.child.wait();
        }
    }
}

fn read_lines(
    reader: impl Read + Send + 'static,
    sender: mpsc::SyncSender<std::io::Result<String>>,
) {
    thread::spawn(move || {
        for line in BufReader::new(reader).lines() {
            let failed = line.is_err();
            if sender.send(line).is_err() || failed {
                break;
            }
        }
    });
}

/// Drain both pipes concurrently so a warning-heavy backend cannot deadlock.
/// None means the user interrupted the download.
pub fn run(
    mut command: Command,
    cancelled: Arc<AtomicBool>,
    mut on_line: impl FnMut(&str),
) -> Result<Option<ExitStatus>> {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let mut managed = ManagedChild {
        child: command.spawn().context("could not start yt-dlp")?,
        reaped: false,
    };
    let (sender, receiver) = mpsc::sync_channel(256);
    read_lines(managed.child.stdout.take().unwrap(), sender.clone());
    read_lines(managed.child.stderr.take().unwrap(), sender);
    let mut stopping = None;
    let mut forced = false;
    let mut disconnected = false;
    let mut status = None;
    loop {
        if cancelled.load(Ordering::SeqCst) && stopping.is_none() {
            on_line("Stopping download…");
            stopping = Some(Instant::now());
            managed.stop(false);
        }
        if stopping.is_some_and(|at| at.elapsed() >= Duration::from_secs(3)) && !forced {
            managed.stop(true);
            forced = true;
        }
        if !disconnected {
            match receiver.recv_timeout(Duration::from_millis(100)) {
                Ok(line) => on_line(&line.context("could not read yt-dlp output")?),
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => disconnected = true,
            }
        } else if status.is_none() {
            thread::sleep(Duration::from_millis(50));
        }
        if status.is_none() {
            status = managed
                .child
                .try_wait()
                .context("could not wait for yt-dlp")?;
        }
        if disconnected && status.is_some() {
            managed.reaped = true;
            return Ok(if stopping.is_some() { None } else { status });
        }
    }
}
