use atlantis_engine::defs::{objects, terrains};
use atlantis_engine::model::{Game, Object, Region, UnitType};
use atlantis_engine::model::{RegionArray, RegionLevelType, TownInfo};
use atlantis_engine::newgame::{add_faction, setup_new_game};
use atlantis_engine::rng::Rng;
use atlantis_engine::ruleset::Ruleset;

#[test]
fn new_game_creates_npc_factions_in_original_order() {
    let ruleset = <dyn Ruleset>::from_name("standard").expect("standard ruleset");
    let mut game = Game::default();

    setup_new_game(&mut game, ruleset, &mut Rng::new(424242)).expect("new game setup");

    let guard_id = game.guard_faction.expect("guard faction id");
    let monster_id = game.monster_faction.expect("monster faction id");
    let guard = game.faction(guard_id).expect("guard faction");
    let monsters = game.faction(monster_id).expect("monster faction");

    assert_eq!(
        (guard.num, guard.name.as_deref()),
        (1, Some("The Guardsmen (1)"))
    );
    assert_eq!(
        (monsters.num, monsters.name.as_deref()),
        (2, Some("Creatures (2)"))
    );
    assert!(guard.faction_types.iter().all(|(_, value)| *value == -1));
    assert!(monsters.faction_types.iter().all(|(_, value)| *value == -1));
    assert_eq!(game.faction_seq, 3);
}

#[test]
fn starting_faction_uses_the_active_ruleset_setup_and_explicit_start_region() {
    let ruleset = <dyn Ruleset>::from_name("neworigins").expect("neworigins ruleset");
    let mut game = Game::default();
    let region = game.add_region(Region::default());
    let dummy = game
        .add_object(region, Object::default())
        .expect("dummy object");
    setup_new_game(&mut game, ruleset, &mut Rng::new(424242)).expect("new game setup");

    let faction_id = add_faction(
        &mut game,
        ruleset,
        false,
        Some(region),
        &mut Rng::new(424242),
    )
    .expect("starting faction");
    let faction = game.faction(faction_id).expect("starting faction");
    let unit_id = game.object(dummy).unwrap().unit_ids()[0];
    let leader = game.unit(unit_id).expect("starting leader");

    assert_eq!(faction.num, 3);
    assert_eq!(
        faction.faction_types,
        [("Martial".to_owned(), 1), ("Magic".to_owned(), 1)]
    );
    assert_eq!(faction.unclaimed, ruleset.game_defs().start_money);
    assert_eq!(faction.start_location, Some(region));
    assert_eq!(leader.faction, Some(faction_id));
    assert_eq!(leader.unit_type as i32, 1);
    assert_eq!(leader.skills.entries().len(), 7);
    assert_ne!(leader.flags & 0x0001, 0);
    assert_eq!(leader.flags & 0x0002, 0);
    assert_eq!(game.faction_seq, 4);
}

#[test]
fn standard_starting_faction_uses_wyreth_skills_and_unclaimed_money() {
    let ruleset = <dyn Ruleset>::from_name("standard").expect("standard ruleset");
    let mut game = Game::default();
    let region = game.add_region(Region::default());
    let dummy = game
        .add_object(region, Object::default())
        .expect("dummy object");
    setup_new_game(&mut game, ruleset, &mut Rng::new(17)).expect("new game setup");

    let faction_id = add_faction(&mut game, ruleset, false, Some(region), &mut Rng::new(17))
        .expect("starting faction");
    let faction = game.faction(faction_id).expect("starting faction");
    let leader = game
        .unit(game.object(dummy).unwrap().unit_ids()[0])
        .expect("starting leader");

    assert_eq!(faction.num, 3);
    assert_eq!(faction.unclaimed, ruleset.game_defs().start_money);
    assert_eq!(leader.unit_type, UnitType::Mage);
    assert_eq!(leader.skills.entries().len(), 3);
    assert_eq!(leader.skills.entries()[0].days, 30);
    assert_eq!(leader.skills.entries()[1].days, 30);
    assert_eq!(leader.skills.entries()[2].days, 30);
    assert_eq!(game.faction_seq, 4);
}

#[test]
fn faction_without_a_leader_keeps_the_ruleset_starting_balance() {
    let ruleset = <dyn Ruleset>::from_name("neworigins").expect("neworigins ruleset");
    let mut game = Game::default();
    game.year = 2;
    game.month = 0;
    setup_new_game(&mut game, ruleset, &mut Rng::new(3)).expect("new game setup");

    let faction_id =
        add_faction(&mut game, ruleset, true, None, &mut Rng::new(3)).expect("leaderless faction");
    let faction = game.faction(faction_id).expect("leaderless faction");

    assert_eq!(
        faction.unclaimed,
        ruleset.game_defs().start_money + 13 * 300
    );
    assert_eq!(faction.no_start_leader, 1);
    assert_eq!(faction.start_location, None);
    assert_eq!(game.unit_seq, 1);
}

#[test]
fn fixed_map_new_game_matches_cpp_factions_units_and_rng() {
    for (ruleset_name, expected, expected_rng) in [
        (
            "standard",
            include_str!("../../../fixtures/new-game/standard.txt"),
            include_str!("../../../fixtures/new-game/standard.rng"),
        ),
        (
            "neworigins",
            include_str!("../../../fixtures/new-game/neworigins.txt"),
            include_str!("../../../fixtures/new-game/neworigins.rng"),
        ),
    ] {
        let ruleset = <dyn Ruleset>::from_name(ruleset_name).expect("ruleset");
        let (mut game, _) = fixed_map(ruleset);
        let mut rng = Rng::new(424242);

        setup_new_game(&mut game, ruleset, &mut rng).expect("new-game setup");
        add_faction(&mut game, ruleset, false, None, &mut rng).expect("starting faction");

        assert_eq!(
            snapshot(&game),
            expected,
            "{ruleset_name} state differs from the C++ reference"
        );
        #[cfg(feature = "rng-trace")]
        assert_eq!(
            rng.trace(),
            expected_rng,
            "{ruleset_name} random draws differ from the C++ reference"
        );
        #[cfg(not(feature = "rng-trace"))]
        let _ = expected_rng;
    }
}

fn fixed_map(ruleset: &'static dyn Ruleset) -> (Game, atlantis_engine::model::RegionId) {
    let mut game = Game::default();
    let move_points = ruleset.game_defs().phased_move_offset % ruleset.game_defs().max_speed;
    let nexus_array = game.add_region_array(
        RegionArray::new(1, 1, RegionLevelType::Nexus).expect("valid nexus dimensions"),
    );
    let surface_array = game.add_region_array(
        RegionArray::new(8, 16, RegionLevelType::Surface).expect("valid surface dimensions"),
    );
    let nexus = add_fixture_region(
        &mut game,
        nexus_array,
        1,
        0,
        0,
        0,
        terrains::R_NEXUS,
        0,
        move_points,
    );

    let mut region_number = 2;
    for y in 0..16 {
        for x in 0..8 {
            if (x + y) % 2 != 0 {
                continue;
            }
            let terrain = if x == 1 && y == 1 {
                terrains::R_TUNDRA
            } else {
                terrains::R_PLAIN
            };
            let town_population = match (x, y) {
                (0, 0) => 5000,
                (0, 2) => 25000,
                _ => 0,
            };
            let region = add_fixture_region(
                &mut game,
                surface_array,
                region_number,
                x,
                y,
                1,
                terrain,
                town_population,
                move_points,
            );
            region_number += 1;
            if (x, y) == (2, 0) {
                add_fixture_object(&mut game, region, objects::O_MAGETOWER as i32, move_points);
            }
            if (x, y) == (4, 0) {
                add_fixture_object(&mut game, region, objects::O_BKEEP as i32, move_points);
            }
        }
    }

    (game, nexus)
}

fn add_fixture_region(
    game: &mut Game,
    array_index: usize,
    number: i32,
    x: i32,
    y: i32,
    z: i32,
    terrain: usize,
    town_population: i32,
    move_points: i32,
) -> atlantis_engine::model::RegionId {
    let town = (town_population != 0).then(|| TownInfo {
        name: Some("Fixture Town".to_owned()),
        population: town_population,
        development: 50,
        ..TownInfo::default()
    });
    let mut region = Region::default();
    region.name = format!("Region {number}");
    region.num = number;
    region.terrain_type = Some(terrain);
    region.xloc = x;
    region.yloc = y;
    region.zloc = z;
    region.town = town;
    let region_id = game.add_region(region);
    game.region_arrays[array_index]
        .set_region(x, y, region_id)
        .expect("fixture coordinate is a valid hex");
    add_fixture_object(game, region_id, objects::O_DUMMY as i32, move_points);
    region_id
}

fn add_fixture_object(
    game: &mut Game,
    region_id: atlantis_engine::model::RegionId,
    object_type: i32,
    move_points: i32,
) {
    let mut object = Object::new(move_points);
    object.object_type = object_type;
    game.add_object(region_id, object)
        .expect("fixture object region exists");
}

fn snapshot(game: &Game) -> String {
    use std::fmt::Write as _;

    let mut output = String::new();
    for faction_id in game.faction_ids() {
        let faction = game.faction(*faction_id).expect("ordered faction exists");
        let types = faction
            .faction_types
            .iter()
            .map(|(name, value)| format!("{name}:{value}"))
            .collect::<Vec<_>>()
            .join(",");
        let attitudes = faction
            .attitudes
            .iter()
            .map(|attitude| format!("{}:{}", attitude.faction_num, attitude.attitude))
            .collect::<Vec<_>>()
            .join(",");
        let items = faction
            .items
            .entries()
            .iter()
            .map(|item| format!("{}:{}", item.item_type.index(), item.num))
            .collect::<Vec<_>>()
            .join(",");
        let skills = snapshot_skills(&faction.skills.entries());
        let shows = faction
            .shows
            .iter()
            .map(|show| format!("{}:{}", show.skill_type, show.level))
            .collect::<Vec<_>>()
            .join(",");
        let start = faction
            .start_location
            .and_then(|region_id| game.region(region_id))
            .map_or_else(
                || "-".to_owned(),
                |region| format!("{},{},{}", region.xloc, region.yloc, region.zloc),
            );
        writeln!(
            output,
            "FACTION|{}|{}|{}|{}|{}|T{}|A{}|I{}|S{}|H{}|R{}|{}",
            faction.num,
            faction.name.as_deref().unwrap_or(""),
            faction.last_orders,
            faction.start_turn,
            faction.unclaimed,
            types,
            attitudes,
            items,
            skills,
            shows,
            start,
            faction.no_start_leader
        )
        .expect("writing to a string succeeds");
    }

    for region_id in game.region_ids() {
        let region = game.region(*region_id).expect("ordered region exists");
        for object_id in region.object_ids() {
            let object = game.object(*object_id).expect("ordered object exists");
            for unit_id in object.unit_ids() {
                let unit = game.unit(*unit_id).expect("ordered unit exists");
                let faction = game
                    .faction(unit.faction.expect("fixture unit has a faction"))
                    .expect("unit faction exists");
                let items = unit
                    .items
                    .entries()
                    .iter()
                    .map(|item| format!("{}:{}", item.item_type.index(), item.num))
                    .collect::<Vec<_>>()
                    .join(",");
                let skills = snapshot_skills(&unit.skills.entries());
                writeln!(
                    output,
                    "UNIT|{}|{}|{}|{}|{}|{}|{}|{}|{},{},{}|{}|I{}|S{}",
                    unit.num,
                    faction.num,
                    unit.unit_type as i32,
                    unit.name.as_deref().unwrap_or(""),
                    unit.guard as i32,
                    unit.reveal,
                    unit.flags,
                    unit.combat,
                    region.xloc,
                    region.yloc,
                    region.zloc,
                    object.object_type,
                    items,
                    skills
                )
                .expect("writing to a string succeeds");
            }
        }
    }
    output
}

fn snapshot_skills(skills: &[atlantis_engine::model::Skill]) -> String {
    skills
        .iter()
        .map(|skill| format!("{}:{}:{}", skill.skill_type, skill.days, skill.exp))
        .collect::<Vec<_>>()
        .join(",")
}
