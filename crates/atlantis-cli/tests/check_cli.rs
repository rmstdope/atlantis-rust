use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

const NEWORIGINS_GAME: &[u8] = include_bytes!("../../../fixtures/neworigins/turn_0/game.in");
const STANDARD_CLEAN_ORDERS: &[u8] =
    include_bytes!("../../../fixtures/check/standard/clean.orders");
const STANDARD_CLEAN_CHECK: &[u8] = include_bytes!("../../../fixtures/check/standard/clean.check");
const STANDARD_ONE_ERROR_ORDERS: &[u8] =
    include_bytes!("../../../fixtures/check/standard/one-error.orders");
const STANDARD_ONE_ERROR_CHECK: &[u8] =
    include_bytes!("../../../fixtures/check/standard/one-error.check");
const NEWORIGINS_ERRORS_ORDERS: &[u8] =
    include_bytes!("../../../fixtures/check/neworigins/errors.orders");
const NEWORIGINS_ERRORS_CHECK: &[u8] =
    include_bytes!("../../../fixtures/check/neworigins/errors.check");
const NEWORIGINS_CLEAN_ORDERS: &[u8] =
    include_bytes!("../../../fixtures/check/neworigins/clean.orders");
const NEWORIGINS_CLEAN_CHECK: &[u8] =
    include_bytes!("../../../fixtures/check/neworigins/clean.check");
const NEWORIGINS_ONE_ERROR_ORDERS: &[u8] =
    include_bytes!("../../../fixtures/check/neworigins/one-error.orders");
const NEWORIGINS_ONE_ERROR_CHECK: &[u8] =
    include_bytes!("../../../fixtures/check/neworigins/one-error.check");
const NEWORIGINS_TWO_ERRORS_ORDERS: &[u8] =
    include_bytes!("../../../fixtures/check/neworigins/two-errors.orders");
const NEWORIGINS_TWO_ERRORS_CHECK: &[u8] =
    include_bytes!("../../../fixtures/check/neworigins/two-errors.check");

static NEXT_DIRECTORY: AtomicUsize = AtomicUsize::new(0);

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new() -> Self {
        let directory = std::env::temp_dir().join(format!(
            "atlantis-check-cli-{}-{}",
            std::process::id(),
            NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).expect("create test directory");
        Self(directory)
    }

    fn with_game(game: &[u8]) -> Self {
        let directory = Self::new();
        fs::write(directory.path().join("game.in"), game).expect("write game fixture");
        directory
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn write(&self, name: &str, contents: &[u8]) {
        fs::write(self.path().join(name), contents).expect("write test input");
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("remove test directory");
    }
}

fn run<I, S>(directory: &TestDirectory, args: I) -> Output
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    Command::new(env!("CARGO_BIN_EXE_atlantis"))
        .args(args)
        .current_dir(directory.path())
        .output()
        .expect("run atlantis")
}

#[test]
fn check_cli_matches_reference_output_and_agreed_file_behavior() {
    let directory = TestDirectory::new();
    directory.write("clean.orders", STANDARD_CLEAN_ORDERS);
    let output = run(
        &directory,
        ["check", "clean.orders", "--ruleset", "standard"],
    );
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, STANDARD_CLEAN_CHECK);
    assert!(output.stderr.is_empty());

    directory.write("one-error.orders", STANDARD_ONE_ERROR_ORDERS);
    let output = run(
        &directory,
        ["check", "one-error.orders", "--ruleset", "standard"],
    );
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, STANDARD_ONE_ERROR_CHECK);

    for (orders_file, orders, expected_check, expected_summary, expected_status) in [
        (
            "clean.3",
            NEWORIGINS_CLEAN_ORDERS,
            NEWORIGINS_CLEAN_CHECK,
            "no errors",
            0,
        ),
        (
            "one.3",
            NEWORIGINS_ONE_ERROR_ORDERS,
            NEWORIGINS_ONE_ERROR_CHECK,
            "1 error",
            1,
        ),
        (
            "orders.3",
            NEWORIGINS_TWO_ERRORS_ORDERS,
            NEWORIGINS_TWO_ERRORS_CHECK,
            "2 errors",
            1,
        ),
    ] {
        directory.write(orders_file, orders);
        directory.write("check.3", b"old result");
        let output = run(
            &directory,
            ["check", orders_file, "check.3", "--ruleset", "neworigins"],
        );
        assert_eq!(output.status.code(), Some(expected_status), "{orders_file}");
        assert_eq!(
            output.stdout,
            format!(
                "Checked {orders_file} against NewOrigins rules: {expected_summary}. Result written to check.3\n"
            )
            .as_bytes(),
            "{orders_file}"
        );
        assert_eq!(
            fs::read(directory.path().join("check.3")).expect("read check result"),
            expected_check,
            "{orders_file}"
        );
    }

    directory.write("errors.orders", NEWORIGINS_ERRORS_ORDERS);
    directory.write("result.check", b"old result");
    let output = run(
        &directory,
        [
            "check",
            "errors.orders",
            "result.check",
            "--ruleset",
            "neworigins",
        ],
    );
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        output.stdout,
        b"Checked errors.orders against NewOrigins rules: 5 errors. Result written to result.check\n"
    );
    assert_eq!(
        fs::read(directory.path().join("result.check")).expect("read check result"),
        NEWORIGINS_ERRORS_CHECK
    );
}

#[test]
fn check_cli_selects_ruleset_from_game_or_override() {
    let detected = TestDirectory::with_game(NEWORIGINS_GAME);
    detected.write("clean.orders", STANDARD_CLEAN_ORDERS);
    let output = run(&detected, ["check", "clean.orders"]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, STANDARD_CLEAN_CHECK);

    let overridden = TestDirectory::with_game(b"not a save");
    overridden.write("one-error.orders", STANDARD_ONE_ERROR_ORDERS);
    let output = run(
        &overridden,
        ["check", "one-error.orders", "--ruleset", "standard"],
    );
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, STANDARD_ONE_ERROR_CHECK);
}

#[test]
fn check_cli_reports_agreed_errors_and_help_without_a_banner() {
    let directory = TestDirectory::new();
    let output = run(&directory, ["check", "missing.txt"]);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(
        output.stdout,
        b"error: there is no orders file missing.txt\n"
    );

    directory.write("orders", STANDARD_CLEAN_ORDERS);
    let output = run(&directory, ["check", "orders"]);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(
        output.stdout,
        b"error: no game.in here to tell which rules to check against. Add --ruleset standard or --ruleset neworigins\n"
    );

    let output = run(&directory, ["check", "orders", "--ruleset", "elves"]);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(
        output.stdout,
        b"error: unknown ruleset elves. Use --ruleset standard or --ruleset neworigins\n"
    );

    directory.write("blocked", b"not a directory");
    let output = run(
        &directory,
        [
            "check",
            "orders",
            "blocked/check.3",
            "--ruleset",
            "standard",
        ],
    );
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(output.stdout, b"error: could not write blocked/check.3\n");

    let output = run(&directory, ["check"]);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(
        output.stdout,
        b"error: an orders file is required. Try: atlantis check --help\n"
    );

    let output = run(&directory, ["check", "--help"]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        output.stdout,
        b"Usage: atlantis check ORDERFILE [CHECKFILE] [--ruleset standard|neworigins]\n  Checks an orders file without running a turn. The result (each order echoed back,\n  errors marked *** Error: ... ***) goes to CHECKFILE, or to the screen if it is left out.\n  The rules come from game.in in this folder unless --ruleset is given.\n"
    );

    let output = run(&directory, ["--help"]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        output.stdout,
        b"Usage: atlantis <command> ...\nCommands: new, edit, check\n"
    );
}

#[test]
fn check_cli_reports_corrupt_game_save() {
    let directory = TestDirectory::with_game(b"not a save");
    directory.write("orders", STANDARD_CLEAN_ORDERS);

    let output = run(&directory, ["check", "orders"]);

    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&output.stdout).starts_with("error: could not load game.in: "),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn check_cli_does_not_wrap_long_completion_lines() {
    let directory = TestDirectory::new();
    let orders_file = format!("{}.orders", "orders".repeat(20));
    let check_file = format!("{}.check", "result".repeat(15));
    directory.write(&orders_file, STANDARD_CLEAN_ORDERS);

    let output = run(
        &directory,
        ["check", &orders_file, &check_file, "--ruleset", "standard"],
    );

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        output.stdout,
        format!(
            "Checked {orders_file} against Wyreth rules: no errors. Result written to {check_file}\n"
        )
        .as_bytes()
    );
    assert_eq!(
        fs::read(directory.path().join(check_file)).expect("read long-path result"),
        STANDARD_CLEAN_CHECK
    );
}
