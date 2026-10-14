use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
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
if [ "${REPLAY_TEST_OMIT_OUTPUT_FAMILY:-}" != report ]; then
    if [ "${REPLAY_TEST_MISMATCH:-0}" = 1 ]; then
        printf 'report start\ncontext before\nactual line\ncontext after\nreport end\n' > report.1
    else
        printf 'report start\ncontext before\nexpected line\ncontext after\nreport end\n' > report.1
    fi
    printf 'faction report\n' > report.3
fi
if [ "${REPLAY_TEST_OMIT_OUTPUT_FAMILY:-}" != template ]; then
    if [ "${REPLAY_TEST_MISMATCH:-0}" = 1 ]; then
        printf 'changed template\n' > template.3
    else
        printf 'orders template\n' > template.3
    fi
fi
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

fn run_harness_omitting_family(fixture: &Path, engine: &Path, family: &str) -> Output {
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/replay-turn.sh");
    Command::new("bash")
        .arg(script)
        .arg(fixture)
        .arg(engine)
        .env("REPLAY_TEST_OMIT_OUTPUT_FAMILY", family)
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
fn replay_harness_rejects_missing_expected_output_families() {
    for (filename_prefix, engine_family, expected_error) in [
        (
            "report.",
            "report",
            "missing required expected output family: report.*",
        ),
        (
            "template.",
            "template",
            "missing required expected output family: template.*",
        ),
    ] {
        let (_temp_dir, fixture, engine) = fixture_and_engine();
        for entry in fs::read_dir(&fixture).expect("read fixture directory") {
            let entry = entry.expect("read fixture file");
            if entry
                .file_name()
                .to_string_lossy()
                .starts_with(filename_prefix)
            {
                fs::remove_file(entry.path()).expect("remove expected output file");
            }
        }

        let output = run_harness_omitting_family(&fixture, &engine, engine_family);
        let message = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );

        assert!(!output.status.success(), "harness passed: {message}");
        assert!(
            message.contains(expected_error),
            "missing expected-family error: {message}"
        );
    }
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

#[test]
fn replay_fixtures_match_sha256_manifest() {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures");
    let manifest =
        fs::read_to_string(fixtures.join("SHA256SUMS")).expect("read fixture SHA-256 manifest");
    let mut expected = BTreeMap::new();

    for line in manifest.lines() {
        let (digest, filename) = line
            .split_once("  ")
            .expect("manifest line has checksum and filename");
        let filename = PathBuf::from(filename);
        let filename_text = filename.to_string_lossy();
        assert_eq!(digest.len(), 64, "invalid SHA-256 digest for {filename:?}");
        assert!(
            filename_text.starts_with("fixtures/standard/turn_")
                || filename_text.starts_with("fixtures/neworigins/turn_")
                || filename_text.starts_with("fixtures/new-game/")
                || filename_text.starts_with("fixtures/economy/region-setup/")
                || filename_text == "fixtures/world-generation/classic-terrain-counts.json",
            "unexpected file in fixture manifest: {filename:?}"
        );
        assert!(
            expected
                .insert(filename.clone(), digest.to_owned())
                .is_none(),
            "duplicate fixture manifest entry: {filename:?}"
        );
    }

    let mut actual = BTreeSet::new();
    for ruleset in ["standard", "neworigins"] {
        let ruleset_dir = fixtures.join(ruleset);
        for turn in fs::read_dir(&ruleset_dir).expect("read ruleset fixture directory") {
            let turn = turn.expect("read fixture directory entry");
            if !turn.file_type().expect("read fixture entry type").is_dir()
                || !turn.file_name().to_string_lossy().starts_with("turn_")
            {
                continue;
            }

            for file in fs::read_dir(turn.path()).expect("read turn fixture directory") {
                let file = file.expect("read turn fixture file");
                let relative = PathBuf::from("fixtures")
                    .join(ruleset)
                    .join(turn.file_name())
                    .join(file.file_name());
                actual.insert(relative);
            }
        }
    }
    let new_game_dir = fixtures.join("new-game");
    for file in fs::read_dir(&new_game_dir).expect("read new-game fixture directory") {
        let file = file.expect("read new-game fixture file");
        assert!(
            file.file_type()
                .expect("read new-game fixture type")
                .is_file(),
            "unexpected directory in new-game fixture directory: {:?}",
            file.file_name()
        );
        actual.insert(PathBuf::from("fixtures/new-game").join(file.file_name()));
    }

    for ruleset in ["standard", "neworigins"] {
        let relative_fixture =
            PathBuf::from("fixtures/economy/region-setup").join(format!("{ruleset}.jsonl"));
        let fixture_path = fixtures.join(
            relative_fixture
                .strip_prefix("fixtures")
                .expect("setup fixture path starts with fixtures"),
        );
        assert!(
            fixture_path.is_file(),
            "missing reference setup fixture: {}",
            fixture_path.display()
        );
        actual.insert(relative_fixture);
    }
    actual.insert(PathBuf::from(
        "fixtures/world-generation/classic-terrain-counts.json",
    ));

    assert_eq!(
        actual,
        expected.keys().cloned().collect(),
        "fixture file inventory differs from SHA-256 manifest"
    );

    for (filename, expected_digest) in expected {
        let fixture_path = fixtures.join(
            filename
                .strip_prefix("fixtures")
                .expect("manifest path starts with fixtures"),
        );
        let contents = fs::read(&fixture_path)
            .unwrap_or_else(|error| panic!("read {}: {error}", fixture_path.display()));
        let actual_digest = Sha256::digest(contents)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        assert_eq!(
            actual_digest,
            expected_digest,
            "fixture bytes differ from manifest: {}",
            fixture_path.display()
        );
    }
}
