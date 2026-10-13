use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn assert_dump_matches_reference(ruleset: &str) {
    let output = Command::new(env!("CARGO_BIN_EXE_atlantis"))
        .args(["dump-tables", "--ruleset", ruleset])
        .output()
        .expect("run atlantis dump-tables");

    assert!(
        output.status.success(),
        "dump-tables failed for {ruleset}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !output.stdout.is_empty(),
        "dump-tables returned no output for {ruleset}"
    );

    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/tables")
        .join(format!("{ruleset}.jsonl"));
    let expected =
        fs::read(&fixture).unwrap_or_else(|error| panic!("read {}: {error}", fixture.display()));

    assert_eq!(
        output.stdout,
        expected,
        "{ruleset} dump differs from the C++ reference at {}",
        fixture.display()
    );
}

#[test]
#[ignore = "Enable after at-pko.2 through at-pko.5 implement the Rust dump command and tables."]
fn dump_tables_standard_matches_cpp_reference() {
    assert_dump_matches_reference("standard");
}

#[test]
#[ignore = "Enable after at-pko.2 through at-pko.5 implement the Rust dump command and tables."]
fn dump_tables_neworigins_matches_cpp_reference() {
    assert_dump_matches_reference("neworigins");
}
