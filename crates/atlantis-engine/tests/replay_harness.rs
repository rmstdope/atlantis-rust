use std::fs;
use std::io::ErrorKind;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_TEMP_DIR: AtomicUsize = AtomicUsize::new(0);

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        loop {
            let path = std::env::temp_dir().join(format!(
                "atlantis-replay-harness-{}-{}",
                std::process::id(),
                NEXT_TEMP_DIR.fetch_add(1, Ordering::Relaxed)
            ));

            match fs::create_dir(&path) {
                Ok(()) => return Self(path),
                Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("could not create {}: {error}", path.display()),
            }
        }
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn fixture_and_engine() -> (TempDir, PathBuf, PathBuf) {
    let temp_dir = TempDir::new();
    let fixture = temp_dir.path().join("turn_0");
    fs::create_dir(&fixture).expect("create fixture directory");

    for (name, contents) in [
        ("game.in", "game input\n"),
        ("players.in", "players input\n"),
        ("orders.3", "orders input\n"),
        ("game.out", "saved game\n"),
        ("players.out", "saved players\n"),
        (
            "report.1",
            "report start\ncontext before\nexpected line\ncontext after\nreport end\n",
        ),
        ("report.3", "faction report\n"),
        ("template.3", "orders template\n"),
        ("times.42", "times article\n"),
        ("engine-output.txt", "reference engine\n"),
    ] {
        fs::write(fixture.join(name), contents).expect("write fixture file");
    }

    let engine = temp_dir.path().join("fake-engine");
    fs::write(
        &engine,
        r#"#!/bin/sh
set -eu
[ "$1" = run ]
printf 'saved game\n' > game.out
printf 'saved players\n' > players.out
if [ "${REPLAY_TEST_MISMATCH:-0}" = 1 ]; then
    printf 'report start\ncontext before\nactual line\ncontext after\nreport end\n' > report.1
    printf 'changed template\n' > template.3
else
    printf 'report start\ncontext before\nexpected line\ncontext after\nreport end\n' > report.1
    printf 'orders template\n' > template.3
fi
printf 'faction report\n' > report.3
printf 'times article\n' > times.42
printf 'reference engine\n'
"#,
    )
    .expect("write fake engine");

    let mut permissions = fs::metadata(&engine)
        .expect("read fake engine metadata")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&engine, permissions).expect("make fake engine executable");

    (temp_dir, fixture, engine)
}

fn run_harness(fixture: &Path, engine: &Path, mismatch: bool) -> Output {
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/replay-turn.sh");
    Command::new("bash")
        .arg(script)
        .arg(fixture)
        .arg(engine)
        .env("REPLAY_TEST_MISMATCH", if mismatch { "1" } else { "0" })
        .output()
        .expect("run replay harness")
}

#[test]
fn replay_harness_reports_identical_outputs() {
    let (_temp_dir, fixture, engine) = fixture_and_engine();
    let output = run_harness(&fixture, &engine, false);
    let message = String::from_utf8_lossy(&output.stdout);
    let errors = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "harness failed:\n{message}{errors}"
    );
    assert!(
        message.contains("Identical:"),
        "unexpected output: {message}"
    );
}

#[test]
fn replay_harness_reports_first_difference_with_line_context() {
    let (_temp_dir, fixture, engine) = fixture_and_engine();
    let output = run_harness(&fixture, &engine, true);
    let message = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(!output.status.success(), "harness passed: {message}");
    assert!(
        message.contains("First output difference in report.1"),
        "missing first-difference file name: {message}"
    );
    assert!(
        message.contains("@@ -1,5 +1,5 @@"),
        "missing diff line range: {message}"
    );
    assert!(
        message.contains(" context before")
            && message.contains("-expected line")
            && message.contains("+actual line")
            && message.contains(" context after"),
        "missing changed line or surrounding context: {message}"
    );
    assert!(
        !message.contains("template.3"),
        "reported output after the first difference: {message}"
    );
}

#[test]
fn replay_fixtures_contain_all_recorded_turns() {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures");

    for ruleset in ["standard", "neworigins"] {
        let ruleset_dir = fixtures.join(ruleset);
        let mut turn_count = 0;
        for entry in fs::read_dir(&ruleset_dir).expect("read ruleset fixture directory") {
            let entry = entry.expect("read fixture directory entry");
            let file_type = entry.file_type().expect("read fixture entry type");
            if file_type.is_dir() && entry.file_name().to_string_lossy().starts_with("turn_") {
                turn_count += 1;
            }
        }
        assert_eq!(turn_count, 14, "wrong turn count for {ruleset}");

        for turn in 0..14 {
            let turn_dir = ruleset_dir.join(format!("turn_{turn}"));
            for filename in [
                "game.in",
                "players.in",
                "orders.3",
                "game.out",
                "players.out",
                "report.1",
                "report.3",
                "template.3",
                "engine-output.txt",
            ] {
                assert!(
                    turn_dir.join(filename).is_file(),
                    "missing {filename} in {}",
                    turn_dir.display()
                );
            }

            if ruleset == "neworigins" {
                let has_times = fs::read_dir(&turn_dir)
                    .expect("read neworigins fixture directory")
                    .map(|entry| entry.expect("read fixture file").file_name())
                    .any(|name| name.to_string_lossy().starts_with("times."));
                assert!(
                    has_times,
                    "missing times.* output in {}",
                    turn_dir.display()
                );
            }
        }
    }
}
