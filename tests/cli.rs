#![cfg(unix)]

use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::Path,
    process::{Command, Output},
    thread,
    time::{Duration, Instant},
};

const URL: &str = "https://www.youtube.com/watch?v=abc&list=PL_example&index=2";

fn executable(path: &Path, script: &str) {
    fs::write(path, script).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

struct Fixture {
    temp: tempfile::TempDir,
}

impl Fixture {
    fn new(body: &str) -> Self {
        let temp = tempfile::tempdir().unwrap();
        executable(
            &temp.path().join("yt-dlp"),
            &format!(
                "#!/bin/sh\nif [ \"$1\" = --version ]; then echo 2026.01.01; exit 0; fi\nprintf '%s\\n' \"$@\" > \"$DTUBE_TEST_ARGS\"\n{body}\n"
            ),
        );
        for name in ["ffmpeg", "ffprobe"] {
            executable(&temp.path().join(name), "#!/bin/sh\nexit 0\n");
        }
        Self { temp }
    }

    fn command(&self, alias: bool) -> Command {
        let mut command = Command::new(if alias {
            env!("CARGO_BIN_EXE_dtube")
        } else {
            env!("CARGO_BIN_EXE_downtube")
        });
        command
            .arg(URL)
            .arg("--yt-dlp")
            .arg(self.temp.path().join("yt-dlp"))
            .arg("--ffmpeg-location")
            .arg(self.temp.path())
            .arg("--output")
            .arg(self.temp.path().join("Music 100%/nested"))
            .env("DTUBE_TEST_ARGS", self.temp.path().join("args"))
            .env(
                "DTUBE_TEST_OUTPUT",
                self.temp
                    .path()
                    .canonicalize()
                    .unwrap()
                    .join("Music 100%/nested"),
            )
            .env("DTUBE_TEST_READY", self.temp.path().join("ready"));
        command
    }
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

const EVENTS: &str = r#"
echo '__DTUBE_TRACK__{"title":"Night song","playlist_title":"Evening Mix","playlist_index":1,"n_entries":2}'
echo '__DTUBE_PROGRESS__{"downloaded_bytes":1024,"total_bytes":2048,"speed":512,"eta":2,"status":"downloading"}' >&2
echo '__DTUBE_PROGRESS__{"downloaded_bytes":2048,"total_bytes":2048,"status":"finished"}' >&2
echo '__DTUBE_POSTPROCESS__{"status":"started"}'
echo '__DTUBE_DONE__"/Music/Evening Mix/001 - Night song [abc].mp3"'
"#;

#[test]
fn both_commands_show_help_without_dependencies() {
    for binary in [env!("CARGO_BIN_EXE_dtube"), env!("CARGO_BIN_EXE_downtube")] {
        let output = Command::new(binary).arg("--help").output().unwrap();
        assert!(output.status.success());
        let help = String::from_utf8(output.stdout).unwrap();
        assert!(help.contains("PLAYLIST_URL"));
        assert!(help.contains("--output"));
    }
}

#[test]
fn streams_progress_and_uses_literal_output_paths_for_both_names() {
    for alias in [false, true] {
        let fixture = Fixture::new(EVENTS);
        let output = fixture.command(alias).output().unwrap();
        assert!(output.status.success(), "{}", stderr(&output));
        let log = stderr(&output);
        assert!(log.contains("[1/2]"));
        assert!(log.contains("Downloading"));
        assert!(log.contains("converting to MP3"));
        assert!(log.contains("1 MP3(s) ready"));
        let args = fs::read_to_string(fixture.temp.path().join("args")).unwrap();
        assert!(args.contains("--audio-quality\n320K\n"));
        assert!(args.contains("--ignore-config\n"));
        assert!(args.contains("Music 100%/nested"));
        assert!(args.ends_with("--\nhttps://www.youtube.com/playlist?list=PL_example\n"));
        assert!(output.stdout.is_empty());
    }
}

#[test]
fn reports_partial_failure_instead_of_claiming_success() {
    let fixture = Fixture::new(&format!(
        "{EVENTS}\necho 'ERROR: track 2 is private' >&2\nexit 1"
    ));
    let output = fixture.command(false).output().unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("track 2 is private"));
    assert!(stderr(&output).contains("playlist incomplete (1 MP3(s) ready)"));
}

#[test]
fn rejects_empty_playlists_and_missing_dependencies() {
    let fixture = Fixture::new("exit 0");
    let empty = fixture.command(false).output().unwrap();
    assert_eq!(empty.status.code(), Some(1));
    assert!(stderr(&empty).contains("no MP3s were produced"));
    fs::remove_file(fixture.temp.path().join("ffprobe")).unwrap();
    let missing = fixture.command(false).output().unwrap();
    assert_eq!(missing.status.code(), Some(1));
    assert!(stderr(&missing).contains("ffprobe was not found"));
}

#[test]
fn rejects_non_playlist_input_before_starting_backend() {
    let output = Command::new(env!("CARGO_BIN_EXE_dtube"))
        .arg("https://youtube.com/watch?v=abc")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("list="));
}

#[test]
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn defaults_to_the_system_music_directory() {
    let fixture = Fixture::new(EVENTS);
    let home = fixture.temp.path().join("home");
    let config = fixture.temp.path().join("config");
    fs::create_dir_all(&home).unwrap();
    fs::create_dir_all(&config).unwrap();
    fs::write(
        config.join("user-dirs.dirs"),
        "XDG_MUSIC_DIR=\"$HOME/My Tunes\"\n",
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_dtube"))
        .arg(URL)
        .arg("--yt-dlp")
        .arg(fixture.temp.path().join("yt-dlp"))
        .arg("--ffmpeg-location")
        .arg(fixture.temp.path())
        .env("HOME", &home)
        .env("XDG_CONFIG_HOME", config)
        .env("DTUBE_TEST_ARGS", fixture.temp.path().join("args"))
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", stderr(&output));
    let expected = home.join(if cfg!(target_os = "linux") {
        "My Tunes"
    } else {
        "Music"
    });
    assert!(expected.is_dir(), "Music directory was not created");
    assert!(stderr(&output).contains(expected.to_str().unwrap()));
}

#[test]
fn ctrl_c_stops_the_downloader_and_exits_130() {
    let fixture = Fixture::new(
        r#"
trap 'exit 130' INT
printf '__DTUBE_TRACK__{"_filename":"%s/interrupted.webm"}\n' "$DTUBE_TEST_OUTPUT"
echo 'incomplete audio' > "$DTUBE_TEST_OUTPUT/interrupted.mp3"
: > "$DTUBE_TEST_READY"
while :; do sleep 1; done
"#,
    );
    let mut child = fixture
        .command(false)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let start = Instant::now();
    while !fixture.temp.path().join("ready").exists() {
        if start.elapsed() > Duration::from_secs(10) {
            let _ = child.kill();
            let _ = child.wait();
            panic!("downloader never started");
        }
        thread::sleep(Duration::from_millis(20));
    }
    // SAFETY: this is the test's own live child process.
    assert_eq!(unsafe { libc::kill(child.id() as i32, libc::SIGINT) }, 0);
    while child.try_wait().unwrap().is_none() {
        if start.elapsed() > Duration::from_secs(15) {
            let _ = child.kill();
            let _ = child.wait();
            panic!("cancellation did not finish");
        }
        thread::sleep(Duration::from_millis(20));
    }
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(130), "{}", stderr(&output));
    assert!(stderr(&output).contains("Run the same command to resume"));
    assert!(
        !fixture
            .temp
            .path()
            .join("Music 100%/nested/interrupted.mp3")
            .exists()
    );
}
