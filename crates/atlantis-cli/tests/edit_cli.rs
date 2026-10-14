use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

use atlantis_engine::{
    defs::{skills::skill_flags, ItemFlags, ItemId, ItemTypeFlags},
    edit::find_region,
    save::{load_save, save_json, LoadedGame},
};

const GAME: &[u8] = include_bytes!("../../../fixtures/standard/turn_0/game.in");
const NEWORIGINS_GAME: &[u8] = include_bytes!("../../../fixtures/neworigins/turn_0/game.in");
static NEXT_DIRECTORY: AtomicUsize = AtomicUsize::new(0);

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new() -> Self {
        Self::with_game(GAME)
    }

    fn with_game(game: &[u8]) -> Self {
        let directory = std::env::temp_dir().join(format!(
            "atlantis-edit-cli-{}-{}",
            std::process::id(),
            NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).expect("create test directory");
        fs::write(directory.join("game.in"), game).expect("write fixture game");
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

fn fixture_loaded() -> LoadedGame {
    fixture_loaded_from(GAME)
}

fn fixture_loaded_from(contents: &[u8]) -> LoadedGame {
    load_save(std::str::from_utf8(contents).expect("fixture is UTF-8")).expect("load fixture")
}

fn region_terrain_example() -> (Vec<String>, String) {
    let loaded = fixture_loaded();
    let region_id = find_region(&loaded.game, 12, 8, 0).expect("fixture region at (12,8,0)");
    let region = loaded.game.region(region_id).expect("fixture region");
    let tables = loaded.ruleset.tables();
    let old_terrain = region
        .terrain_type
        .and_then(|terrain| tables.terrains.get(terrain))
        .map(|terrain| terrain.name)
        .unwrap_or("none");
    let args = vec![
        "edit".to_owned(),
        "region".to_owned(),
        "12".to_owned(),
        "8".to_owned(),
        "0".to_owned(),
        "set-terrain".to_owned(),
        "forest".to_owned(),
    ];
    let expected = format!(
        "Region (12,8) \"{}\": terrain {old_terrain} -> forest",
        region.name
    );
    (args, expected)
}

fn unit_item_example() -> (Vec<String>, String) {
    unit_item_example_from(GAME)
}

fn unit_item_example_from(game: &[u8]) -> (Vec<String>, String) {
    let loaded = fixture_loaded_from(game);
    let (number, name, old_silver) = loaded
        .game
        .region_ids()
        .iter()
        .copied()
        .find_map(|region_id| {
            let region = loaded.game.region(region_id)?;
            region.object_ids().iter().copied().find_map(|object_id| {
                let object = loaded.game.object(object_id)?;
                object.unit_ids().iter().copied().find_map(|unit_id| {
                    let unit = loaded.game.unit(unit_id)?;
                    Some((
                        unit.num,
                        unit.name.as_deref().unwrap_or("Unit").to_owned(),
                        unit.items.get_num(ItemId::Silver),
                    ))
                })
            })
        })
        .expect("fixture has a unit");
    let args = vec![
        "edit".to_owned(),
        "unit".to_owned(),
        number.to_string(),
        "set-item".to_owned(),
        "SILV".to_owned(),
        "500".to_owned(),
    ];
    (
        args,
        format!("Unit {name} ({number}): silver {old_silver} -> 500"),
    )
}

fn first_editable_region_coordinates(loaded: &LoadedGame) -> (i32, i32, i32) {
    loaded
        .game
        .region_ids()
        .iter()
        .copied()
        .find_map(|region_id| {
            let region = loaded.game.region(region_id)?;
            (region.terrain_type.is_some() && loaded.game.get_dummy(region_id).is_some())
                .then_some((region.xloc, region.yloc, region.zloc))
        })
        .expect("fixture has a terrain region with a dummy object")
}

fn first_unit_and_faction(loaded: &LoadedGame) -> (i32, i32, String) {
    let unit_number = loaded
        .game
        .region_ids()
        .iter()
        .copied()
        .find_map(|region_id| {
            let region = loaded.game.region(region_id)?;
            region.object_ids().iter().copied().find_map(|object_id| {
                let object = loaded.game.object(object_id)?;
                object
                    .unit_ids()
                    .first()
                    .and_then(|unit_id| loaded.game.unit(*unit_id))
                    .map(|unit| unit.num)
            })
        })
        .expect("fixture has a unit");
    let faction_number = loaded
        .game
        .faction_ids()
        .first()
        .and_then(|id| loaded.game.faction(*id))
        .map(|faction| faction.num)
        .expect("fixture has a faction");
    let skill = loaded
        .ruleset
        .tables()
        .skills
        .skills
        .iter()
        .find(|skill| skill.flags & skill_flags::DISABLED == 0)
        .and_then(|skill| skill.name)
        .expect("ruleset has a named skill")
        .to_owned();
    (unit_number, faction_number, skill)
}

fn first_enabled_race_name(loaded: &LoadedGame) -> String {
    loaded
        .ruleset
        .tables()
        .item_families
        .items
        .iter()
        .find(|item| {
            item.item_type & ItemTypeFlags::MAN != 0 && item.flags & ItemFlags::DISABLED == 0
        })
        .and_then(|item| item.name.as_deref())
        .expect("ruleset has an enabled race")
        .to_owned()
}

#[test]
fn edit_help_lists_supported_operations() {
    let directory = TestDirectory::new();
    let output = run(&directory, ["edit", "--help"]);

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("help is UTF-8");
    assert!(stdout.contains("Usage: atlantis edit <what> ... [--out FILE] [--dry-run]"));
    assert!(stdout.contains("region X Y Z"));
    assert!(stdout.contains("unit NUM"));
    assert!(stdout.contains("market"));
    assert!(stdout.contains("Names are not case-sensitive"));
}

#[test]
fn region_set_terrain_reports_change_and_keeps_backup() {
    let directory = TestDirectory::new();
    let (args, change) = region_terrain_example();
    let output = run(&directory, &args);

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).expect("output is UTF-8"),
        format!("{change}\nSaved game.in\nPrevious version kept as game.in.bak\n")
    );
    assert_eq!(
        fs::read(directory.path().join("game.in.bak")).unwrap(),
        GAME
    );
    assert_ne!(fs::read(directory.path().join("game.in")).unwrap(), GAME);
}

#[test]
fn unit_set_item_reports_change() {
    let directory = TestDirectory::new();
    let (args, change) = unit_item_example();
    let output = run(&directory, &args);

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).expect("output is UTF-8"),
        format!("{change}\nSaved game.in\nPrevious version kept as game.in.bak\n")
    );
}

#[test]
fn dry_run_does_not_write() {
    let directory = TestDirectory::new();
    let (mut args, change) = region_terrain_example();
    args.push("--dry-run".to_owned());
    let output = run(&directory, &args);

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).expect("output is UTF-8"),
        format!("{change}\nDry run: nothing saved.\n")
    );
    assert_eq!(fs::read(directory.path().join("game.in")).unwrap(), GAME);
    assert!(!directory.path().join("game.in.bak").exists());
}

#[test]
fn out_file_keeps_game_in_unchanged() {
    let directory = TestDirectory::new();
    let (mut args, change) = unit_item_example();
    args.extend(["--out".to_owned(), "game.new".to_owned()]);
    let output = run(&directory, &args);

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).expect("output is UTF-8"),
        format!("{change}\nSaved game.new\n")
    );
    assert_eq!(fs::read(directory.path().join("game.in")).unwrap(), GAME);
    assert!(directory.path().join("game.new").exists());
    assert!(!directory.path().join("game.in.bak").exists());
}

#[test]
fn script_applies_edits_in_order_and_skips_comments() {
    let directory = TestDirectory::new();
    let (region_args, region_change) = region_terrain_example();
    let (unit_args, unit_change) = unit_item_example();
    fs::write(
        directory.path().join("fixes.txt"),
        format!(
            "# apply the terrain edit first\n{} # inline comment\n{}",
            region_args[1..].join(" "),
            unit_args[1..].join(" ")
        ),
    )
    .expect("write edit script");
    let output = run(&directory, ["edit", "--script", "fixes.txt"]);

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).expect("output is UTF-8"),
        format!(
            "{region_change}\n{unit_change}\n\
             2 edits applied. Saved game.in\n\
             Previous version kept as game.in.bak\n"
        )
    );
    assert_eq!(
        fs::read(directory.path().join("game.in.bak")).unwrap(),
        GAME
    );
    assert_ne!(fs::read(directory.path().join("game.in")).unwrap(), GAME);
}

#[test]
fn empty_script_is_rejected() {
    let directory = TestDirectory::new();
    fs::write(directory.path().join("empty.txt"), "# no edits\n\n").expect("write empty script");
    let output = run(&directory, ["edit", "--script", "empty.txt"]);

    assert!(!output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).expect("output is UTF-8"),
        "error: empty.txt has no edits. Nothing was changed.\n"
    );
    assert_eq!(fs::read(directory.path().join("game.in")).unwrap(), GAME);
    assert!(!directory.path().join("game.in.bak").exists());
}

#[test]
fn failed_script_reports_line_and_saves_nothing() {
    let directory = TestDirectory::new();
    let (region_args, region_change) = region_terrain_example();
    let (unit_args, _) = unit_item_example();
    fs::write(
        directory.path().join("broken.txt"),
        format!(
            "{}\n# failure is on physical line 3\nregion 12 8 0 set-terrain swamplandz\n{}",
            region_args[1..].join(" "),
            unit_args[1..].join(" ")
        ),
    )
    .expect("write failing script");
    let output = run(&directory, ["edit", "--script", "broken.txt"]);

    assert!(!output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).expect("output is UTF-8"),
        format!(
            "{region_change}\n\
             error: unknown terrain \"swamplandz\". Valid: plain forest mountain swamp jungle desert tundra ... (line 3)\n\
             Nothing saved: 1 of 3 edits failed.\n"
        )
    );
    assert_eq!(fs::read(directory.path().join("game.in")).unwrap(), GAME);
    assert!(!directory.path().join("game.in.bak").exists());
}

#[test]
fn region_script_reaches_region_town_gate_object_and_market_operations() {
    let directory = TestDirectory::new();
    let loaded = fixture_loaded();
    let (x, y, z) = first_editable_region_coordinates(&loaded);
    let race = first_enabled_race_name(&loaded);
    let commands = [
        format!("region {x} {y} {z} set-terrain forest"),
        format!("region {x} {y} {z} set-race \"{race}\""),
        format!("region {x} {y} {z} set-wages 100"),
        format!("region {x} {y} {z} set-name Edited Region"),
        format!("region {x} {y} {z} regenerate-products"),
        format!("region {x} {y} {z} regenerate-all"),
        format!("region {x} {y} {z} add-gate"),
        format!("region {x} {y} {z} remove-gate"),
        format!("region {x} {y} {z} add-town"),
        format!("region {x} {y} {z} rename-town Edited Town"),
        format!("region {x} {y} {z} regenerate-town"),
        format!("region {x} {y} {z} add-object Tower"),
        format!("region {x} {y} {z} rename-object 1 Great Tower"),
        format!("region {x} {y} {z} remove-object 1"),
        format!("region {x} {y} {z} market population SILV 0 5000"),
        format!("region {x} {y} {z} market amount SILV 1 1000"),
        format!("region {x} {y} {z} market prices SILV 20 20"),
        format!("region {x} {y} {z} market swap SILV"),
        format!("region {x} {y} {z} market remove SILV"),
        format!("region {x} {y} {z} market regenerate"),
        format!("region {x} {y} {z} remove-town"),
    ];
    fs::write(
        directory.path().join("region-edits.txt"),
        commands.join("\n"),
    )
    .expect("write region script");
    let output = run(
        &directory,
        ["edit", "--script", "region-edits.txt", "--dry-run"],
    );

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    let stdout = String::from_utf8(output.stdout).expect("output is UTF-8");
    assert_eq!(stdout.lines().count(), commands.len() + 1);
    assert!(stdout.ends_with("Dry run: nothing saved.\n"));
    assert_eq!(fs::read(directory.path().join("game.in")).unwrap(), GAME);
}

#[test]
fn unit_script_reaches_item_skill_faction_type_location_and_detail_operations() {
    let directory = TestDirectory::new();
    let loaded = fixture_loaded();
    let (unit, faction, skill) = first_unit_and_faction(&loaded);
    let (x, y, z) = first_editable_region_coordinates(&loaded);
    let commands = [
        format!("unit {unit} set-item SILV 500"),
        format!("unit {unit} set-skill \"{skill}\" 30"),
        format!("unit {unit} set-faction {faction}"),
        format!("unit {unit} set-type mage"),
        format!("unit {unit} set-location {x} {y} {z}"),
        format!("unit {unit} set-name Edited Unit"),
        format!("unit {unit} set-description Edited description"),
        format!("unit {unit} set-guard guard"),
        format!("unit {unit} clear-description"),
        format!("unit create {faction} {x} {y} {z}"),
    ];
    fs::write(directory.path().join("unit-edits.txt"), commands.join("\n"))
        .expect("write unit script");
    let output = run(
        &directory,
        ["edit", "--script", "unit-edits.txt", "--dry-run"],
    );

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    let stdout = String::from_utf8(output.stdout).expect("output is UTF-8");
    assert_eq!(stdout.lines().count(), commands.len() + 1);
    assert!(stdout.ends_with("Dry run: nothing saved.\n"));
    assert_eq!(fs::read(directory.path().join("game.in")).unwrap(), GAME);
}

#[test]
fn lookups_are_case_insensitive_and_failed_skill_script_is_atomic() {
    let directory = TestDirectory::new();
    let (region_args, change) = region_terrain_example();
    let mut command = region_args;
    command[1] = "ReGiOn".to_owned();
    command[5] = "SeT-TeRrAiN".to_owned();
    command[6] = "FoReSt".to_owned();
    command.push("--dry-run".to_owned());
    let output = run(&directory, &command);

    assert!(output.status.success());
    assert!(String::from_utf8(output.stdout)
        .expect("output is UTF-8")
        .starts_with(&format!("{change}\n")));
    assert_eq!(fs::read(directory.path().join("game.in")).unwrap(), GAME);

    let (unit_args, _) = unit_item_example();
    let combat_skill = run(
        &directory,
        [
            "edit".to_owned(),
            "unit".to_owned(),
            unit_args[2].clone(),
            "set-skill".to_owned(),
            "COMB".to_owned(),
            "10".to_owned(),
            "--dry-run".to_owned(),
        ],
    );
    assert!(
        combat_skill.status.success(),
        "{}",
        String::from_utf8_lossy(&combat_skill.stdout)
    );
    assert!(String::from_utf8(combat_skill.stdout)
        .expect("output is UTF-8")
        .contains("skill combat days per man"));

    fs::write(
        directory.path().join("bad-skill.txt"),
        format!("unit {} set-skill ZZZZ 10\n", unit_args[2]),
    )
    .expect("write bad skill script");
    let failed = run(&directory, ["edit", "--script", "bad-skill.txt"]);

    assert!(!failed.status.success());
    assert_eq!(
        String::from_utf8(failed.stdout).expect("output is UTF-8"),
        "error: unknown skill \"ZZZZ\" (line 1)\n\
         Nothing saved: 1 of 1 edits failed.\n"
    );
    assert_eq!(fs::read(directory.path().join("game.in")).unwrap(), GAME);
}

#[test]
fn json_input_is_written_as_json() {
    let directory = TestDirectory::new();
    let json = save_json(&fixture_loaded()).expect("serialize JSON fixture");
    fs::write(directory.path().join("game.in"), json.as_bytes()).expect("write JSON fixture");
    let loaded = fixture_loaded();
    let (x, y, z) = first_editable_region_coordinates(&loaded);
    let output = run(
        &directory,
        [
            "edit".to_owned(),
            "region".to_owned(),
            x.to_string(),
            y.to_string(),
            z.to_string(),
            "set-name".to_owned(),
            "JSON Region".to_owned(),
        ],
    );

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    let written = fs::read_to_string(directory.path().join("game.in")).unwrap();
    assert!(written.starts_with('{'));
    assert!(load_save(&written).is_ok());
}

#[test]
fn neworigins_accepts_case_insensitive_item_abbreviations() {
    let directory = TestDirectory::with_game(NEWORIGINS_GAME);
    let (mut args, change) = unit_item_example_from(NEWORIGINS_GAME);
    args[1] = "UnIt".to_owned();
    args[3] = "SET-ITEM".to_owned();
    args[4] = "sIlV".to_owned();
    let output = run(&directory, &args);

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).expect("output is UTF-8"),
        format!("{change}\nSaved game.in\nPrevious version kept as game.in.bak\n")
    );
    assert_eq!(
        fs::read(directory.path().join("game.in.bak")).unwrap(),
        NEWORIGINS_GAME
    );
}

#[test]
fn missing_region_and_argument_errors_do_not_write() {
    let directory = TestDirectory::new();
    let missing_region = run(
        &directory,
        ["edit", "region", "99", "8", "99", "set-terrain", "forest"],
    );

    assert!(!missing_region.status.success());
    assert_eq!(
        String::from_utf8(missing_region.stdout).expect("output is UTF-8"),
        "error: there is no region at (99,8,99). Nothing was changed.\n"
    );
    let missing_argument = run(
        &directory,
        ["edit", "region", "12", "8", "0", "set-terrain"],
    );
    assert!(!missing_argument.status.success());
    assert_eq!(
        String::from_utf8(missing_argument.stdout).expect("output is UTF-8"),
        "error: set-terrain needs a terrain name. Try: atlantis edit region --help. Nothing was changed.\n"
    );
    assert_eq!(fs::read(directory.path().join("game.in")).unwrap(), GAME);
    assert!(!directory.path().join("game.in.bak").exists());
}

#[test]
fn script_with_one_edit_reports_count_and_out_cannot_overwrite_input() {
    let directory = TestDirectory::new();
    let (region_args, change) = region_terrain_example();
    fs::write(directory.path().join("one.txt"), region_args[1..].join(" "))
        .expect("write one-line script");
    let script = run(&directory, ["edit", "--script", "one.txt"]);
    assert!(script.status.success());
    assert_eq!(
        String::from_utf8(script.stdout).expect("output is UTF-8"),
        format!(
            "{change}\n\
             1 edit applied. Saved game.in\n\
             Previous version kept as game.in.bak\n"
        )
    );

    let original = fs::read(directory.path().join("game.in")).unwrap();
    let out_same = run(
        &directory,
        [
            "edit",
            "region",
            "12",
            "8",
            "0",
            "set-terrain",
            "forest",
            "--out",
            "game.in",
        ],
    );
    assert!(!out_same.status.success());
    assert!(String::from_utf8(out_same.stdout)
        .expect("output is UTF-8")
        .contains("--out must be different from the input file"));
    assert_eq!(
        fs::read(directory.path().join("game.in")).unwrap(),
        original
    );
}

#[test]
fn out_hard_link_cannot_overwrite_input_without_backup() {
    let directory = TestDirectory::new();
    let hard_link = directory.path().join("game.link");
    fs::hard_link(directory.path().join("game.in"), &hard_link).expect("create hard link");
    let (mut args, _) = region_terrain_example();
    args.extend(["--out".to_owned(), "game.link".to_owned()]);

    let output = run(&directory, &args);

    assert!(!output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).expect("output is UTF-8"),
        "error: --out must be different from the input file. Nothing was changed.\n"
    );
    assert_eq!(fs::read(directory.path().join("game.in")).unwrap(), GAME);
    assert_eq!(fs::read(hard_link).unwrap(), GAME);
    assert!(!directory.path().join("game.in.bak").exists());
}
