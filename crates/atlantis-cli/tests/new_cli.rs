use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

use atlantis_engine::players::read_players;
use atlantis_engine::save::load_save;

static NEXT_DIRECTORY: AtomicUsize = AtomicUsize::new(0);

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new() -> Self {
        let directory = std::env::temp_dir().join(format!(
            "atlantis-new-cli-{}-{}",
            std::process::id(),
            NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).expect("create test directory");
        Self(directory)
    }

    fn path(&self) -> &Path {
        &self.0
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
fn new_cli_writes_loadable_outputs_for_both_rulesets() {
    for (ruleset_name, expected_name, expected_levels) in
        [("standard", "Wyreth", 2), ("neworigins", "NewOrigins", 3)]
    {
        let directory = TestDirectory::new();
        let output = run(
            &directory,
            [
                "new",
                "--ruleset",
                ruleset_name,
                "--seed",
                "424242",
                "--width",
                "16",
                "--height",
                "16",
            ],
        );
        assert!(
            output.status.success(),
            "stdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stdout.is_empty());
        let game_contents =
            fs::read_to_string(directory.path().join("game.out")).expect("read generated game");
        let players_contents = fs::read_to_string(directory.path().join("players.out"))
            .expect("read generated players");
        let mut loaded = load_save(&game_contents).expect("load generated game");
        assert_eq!(loaded.ruleset.name(), expected_name);
        assert_eq!(loaded.game.region_arrays.len(), expected_levels);

        let ruleset = loaded.ruleset;
        let tables = ruleset.tables();
        read_players(
            &mut loaded.game,
            &players_contents,
            ruleset.game_defs(),
            &tables,
        )
        .expect("load generated players");
        assert_eq!(loaded.game.game_status, 1);
    }
}

#[test]
fn new_cli_uses_the_natural_generator_for_neworigins() {
    let directory = TestDirectory::new();
    let output = run(
        &directory,
        [
            "new",
            "--ruleset",
            "neworigins",
            "--seed",
            "424242",
            "--width",
            "16",
            "--height",
            "16",
            "--generator",
            "2",
        ],
    );

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let game_contents =
        fs::read_to_string(directory.path().join("game.out")).expect("read generated game");
    assert_eq!(
        load_save(&game_contents)
            .expect("load generated game")
            .ruleset
            .name(),
        "NewOrigins"
    );
}

#[test]
fn new_cli_rejects_invalid_generation_options_without_writing_outputs() {
    for args in [
        vec![
            "new",
            "--ruleset",
            "standard",
            "--width",
            "12",
            "--height",
            "16",
        ],
        vec![
            "new",
            "--ruleset",
            "standard",
            "--seed",
            "1",
            "--generator",
            "2",
        ],
        vec!["new", "--ruleset", "standard", "--nexus", "3"],
    ] {
        let directory = TestDirectory::new();
        let output = run(&directory, args);

        assert_eq!(output.status.code(), Some(2));
        assert!(!directory.path().join("game.out").exists());
        assert!(!directory.path().join("players.out").exists());
    }
}

#[test]
fn new_cli_uses_the_default_dimensions_and_requested_nexus_size() {
    let default_directory = TestDirectory::new();
    let output = run(
        &default_directory,
        ["new", "--ruleset", "standard", "--seed", "424242"],
    );
    assert!(output.status.success());
    let default_game =
        fs::read_to_string(default_directory.path().join("game.out")).expect("read default game");
    let default_game = load_save(&default_game).expect("load default game");
    assert_eq!(
        (
            default_game.game.region_arrays[1].x,
            default_game.game.region_arrays[1].y
        ),
        (64, 64)
    );

    let nexus_directory = TestDirectory::new();
    let output = run(
        &nexus_directory,
        [
            "new",
            "--ruleset",
            "neworigins",
            "--seed",
            "424242",
            "--width",
            "16",
            "--height",
            "16",
            "--nexus",
            "2",
        ],
    );
    assert!(output.status.success());
    let nexus_game =
        fs::read_to_string(nexus_directory.path().join("game.out")).expect("read nexus game");
    let nexus_game = load_save(&nexus_game).expect("load nexus game");
    let nexus = &nexus_game.game.region_arrays[0];
    assert_eq!((nexus.x, nexus.y), (2, 2));
    let nexus_regions = (0..nexus.x)
        .flat_map(|x| (0..nexus.y).map(move |y| (x, y)))
        .filter(|(x, y)| nexus.get_region(*x, *y).is_some())
        .count();
    assert_eq!(nexus_regions, 2);
}

#[test]
fn new_cli_help_and_general_help_list_the_command() {
    let directory = TestDirectory::new();
    let command_help = run(&directory, ["new", "--help"]);
    assert!(command_help.status.success());
    assert!(String::from_utf8_lossy(&command_help.stdout).contains("Usage: atlantis new --ruleset"));

    let general_help = run(&directory, ["--help"]);
    assert!(general_help.status.success());
    assert!(String::from_utf8_lossy(&general_help.stdout).contains("new"));

    let unknown_command = run(&directory, ["unknown"]);
    assert_eq!(unknown_command.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&unknown_command.stdout).contains("new` or `edit"));
}
