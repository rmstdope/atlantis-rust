use atlantis_engine::{
    defs::{
        objects::{O_DUMMY, O_FLEET},
        ItemFlags, ItemId,
    },
    model::{
        Attitude, Faction, Game, GuardMode, Item, ItemList, Market, MarketType, Object, Production,
        Quest, Region, RegionArray, RegionLevelType, Skill, SkillList, TownInfo, Unit, UnitType,
    },
    rng::Rng,
    ruleset::Ruleset,
    save::{load_legacy, save_legacy, LoadedGame, SaveError},
};

const STANDARD_GAME: &str = include_str!("../../../fixtures/standard/turn_0/game.in");
const NEW_ORIGINS_GAME: &str = include_str!("../../../fixtures/neworigins/turn_0/game.in");

macro_rules! game_fixture {
    ($ruleset:literal, $turn:literal) => {
        include_str!(concat!(
            "../../../fixtures/",
            $ruleset,
            "/turn_",
            $turn,
            "/game.in"
        ))
    };
}

const GAME_FIXTURES: [(&str, &str); 28] = [
    ("standard", game_fixture!("standard", "0")),
    ("standard", game_fixture!("standard", "1")),
    ("standard", game_fixture!("standard", "2")),
    ("standard", game_fixture!("standard", "3")),
    ("standard", game_fixture!("standard", "4")),
    ("standard", game_fixture!("standard", "5")),
    ("standard", game_fixture!("standard", "6")),
    ("standard", game_fixture!("standard", "7")),
    ("standard", game_fixture!("standard", "8")),
    ("standard", game_fixture!("standard", "9")),
    ("standard", game_fixture!("standard", "10")),
    ("standard", game_fixture!("standard", "11")),
    ("standard", game_fixture!("standard", "12")),
    ("standard", game_fixture!("standard", "13")),
    ("neworigins", game_fixture!("neworigins", "0")),
    ("neworigins", game_fixture!("neworigins", "1")),
    ("neworigins", game_fixture!("neworigins", "2")),
    ("neworigins", game_fixture!("neworigins", "3")),
    ("neworigins", game_fixture!("neworigins", "4")),
    ("neworigins", game_fixture!("neworigins", "5")),
    ("neworigins", game_fixture!("neworigins", "6")),
    ("neworigins", game_fixture!("neworigins", "7")),
    ("neworigins", game_fixture!("neworigins", "8")),
    ("neworigins", game_fixture!("neworigins", "9")),
    ("neworigins", game_fixture!("neworigins", "10")),
    ("neworigins", game_fixture!("neworigins", "11")),
    ("neworigins", game_fixture!("neworigins", "12")),
    ("neworigins", game_fixture!("neworigins", "13")),
];

fn replace_line(input: &str, line: usize, replacement: &str) -> String {
    let mut lines = input.lines().collect::<Vec<_>>();
    lines[line] = replacement;
    lines.join("\n")
}

fn replace_line_bytes(input: &str, line_index: usize, replacement: &str) -> String {
    let mut output = String::new();
    for (index, line) in input.split_inclusive('\n').enumerate() {
        if index == line_index {
            let ending = if line.ends_with("\r\n") {
                "\r\n"
            } else if line.ends_with('\n') {
                "\n"
            } else {
                ""
            };
            output.push_str(replacement);
            output.push_str(ending);
        } else {
            output.push_str(line);
        }
    }
    output
}

#[test]
fn legacy_writer_writes_header_and_factions_in_file_order() {
    let seed = 4242;
    let ruleset = <dyn Ruleset>::from_name("Wyreth").expect("standard ruleset");
    let tables = ruleset.tables();
    let skill_type = tables
        .skills
        .skills
        .iter()
        .position(|skill| skill.abbreviation.is_some())
        .expect("a skill with an abbreviation");
    let skill_abbreviation = tables.skills.skills[skill_type]
        .abbreviation
        .expect("skill abbreviation");

    let mut game = Game::default();
    let mut first = Faction::new(15, ruleset.faction_types());
    first.faction_types[0].1 = -1;
    first.faction_types[1].1 = 2;
    first.faction_types[2].1 = 3;
    first.last_change = -6;
    first.last_orders = 7;
    first.unclaimed = 8;
    first.name = Some("First faction".to_owned());
    first.address = Some("first@example.test".to_owned());
    first.password = Some("secret".to_owned());
    first.times = 1;
    first.show_unit_attitudes = 1;
    first.template_format = 2;
    first.skills.push(Skill {
        skill_type,
        days: 42,
        exp: 5,
    });
    first.items.push(Item {
        item_type: ItemId::Wood,
        num: 9,
        selling: 0,
        checked: false,
    });
    first.default_attitude = 3;
    first.attitudes.push(Attitude {
        faction_num: 3,
        attitude: 2,
    });
    game.add_faction(first);

    let mut second = Faction::new(3, ruleset.faction_types());
    second.name = Some("Second faction".to_owned());
    game.add_faction(second);

    let mut loaded = LoadedGame {
        game,
        ruleset,
        seed,
        rng: Rng::new(seed),
    };
    let mut expected_rng = Rng::new(seed);
    let expected_seed = expected_rng.get_random(10_000);
    let expected_next_seed = expected_rng.get_random(10_000);
    let output = save_legacy(&mut loaded).expect("write legacy save");
    let skill_record = if ruleset.game_defs().required_experience != 0 {
        format!("{skill_abbreviation} 42 5")
    } else {
        format!("{skill_abbreviation} 42")
    };

    assert_eq!(
        output.lines().collect::<Vec<_>>(),
        [
            "atlantis_game".to_owned(),
            "328197".to_owned(),
            "Wyreth".to_owned(),
            "131072".to_owned(),
            "1".to_owned(),
            "-1".to_owned(),
            expected_seed.to_string(),
            "1".to_owned(),
            "1".to_owned(),
            "100".to_owned(),
            "-1".to_owned(),
            "-1".to_owned(),
            "2".to_owned(),
            "15".to_owned(),
            "-1".to_owned(),
            "2".to_owned(),
            "3".to_owned(),
            "-6".to_owned(),
            "7".to_owned(),
            "8".to_owned(),
            "First faction".to_owned(),
            "first@example.test".to_owned(),
            "secret".to_owned(),
            "1".to_owned(),
            "1".to_owned(),
            "2".to_owned(),
            "1".to_owned(),
            skill_record,
            "1".to_owned(),
            "9 WOOD".to_owned(),
            "3".to_owned(),
            "1".to_owned(),
            "3".to_owned(),
            "2".to_owned(),
            "3".to_owned(),
            "1".to_owned(),
            "1".to_owned(),
            "1".to_owned(),
            "-6".to_owned(),
            "0".to_owned(),
            "0".to_owned(),
            "Second faction".to_owned(),
            "NoAddress".to_owned(),
            "none".to_owned(),
            "1".to_owned(),
            "0".to_owned(),
            "2".to_owned(),
            "0".to_owned(),
            "0".to_owned(),
            "2".to_owned(),
            "0".to_owned(),
            "0".to_owned(),
            "0".to_owned(),
            "0".to_owned(),
            "Neighbors".to_owned(),
            "0".to_owned(),
        ]
    );
    assert!(output.ends_with('\n'));
    assert_eq!(loaded.rng.get_random(10_000), expected_next_seed);

    let new_origins = <dyn Ruleset>::from_name("NewOrigins").expect("neworigins ruleset");
    let mut new_game = Game::default();
    let mut new_faction = Faction::new(20, new_origins.faction_types());
    new_faction.faction_types[0].1 = 4;
    new_faction.faction_types[1].1 = 5;
    new_game.add_faction(new_faction);
    let mut new_loaded = LoadedGame {
        game: new_game,
        ruleset: new_origins,
        seed,
        rng: Rng::new(seed),
    };
    let new_output = save_legacy(&mut new_loaded).expect("write neworigins save");
    let new_lines = new_output.lines().collect::<Vec<_>>();
    assert_eq!(new_lines[2], "NewOrigins");
    assert_eq!(&new_lines[12..16], ["1", "20", "4", "5"]);
}

#[test]
fn legacy_writer_writes_regions_objects_units_and_conditional_fields() {
    let seed = 51;
    let ruleset = <dyn Ruleset>::from_name("Wyreth").expect("standard ruleset");
    let tables = ruleset.tables();
    let skill_type = tables
        .skills
        .skills
        .iter()
        .position(|skill| skill.abbreviation == Some("MINI"))
        .expect("mining skill");
    let sword = tables.item_families.items[ItemId::Sword.index()]
        .abbreviation
        .as_deref()
        .expect("sword abbreviation");
    let terrain = tables.terrains[0].terrain_type;
    let fleet_type = tables.objects[O_FLEET].name;
    let dummy_type = tables.objects[O_DUMMY].name;
    let longboat = tables.item_families.items[ItemId::Longboat.index()]
        .abbreviation
        .as_deref()
        .expect("longboat abbreviation");

    let mut game = Game::default();
    game.number_of_gates = 1;
    let faction_id = game.add_faction(Faction::new(12, ruleset.faction_types()));
    let mut array = RegionArray::new(2, 2, RegionLevelType::Surface).expect("region array");
    array.name = "Test level".to_owned();
    let array_id = game.add_region_array(array);
    let mut first_region = Region::default();
    first_region.name = "First".to_owned();
    first_region.num = 40;
    first_region.terrain_type = Some(0);
    first_region.building_seq = 13;
    first_region.gate = 3;
    first_region.gate_month = 8;
    first_region.race = ItemId::Peasant as i32;
    first_region.population = 101;
    first_region.base_population = 99;
    first_region.wages = 8;
    first_region.max_wages = 9;
    first_region.wealth = 10;
    first_region.elevation = 11;
    first_region.humidity = 12;
    first_region.temperature = 13;
    first_region.vegetation = 14;
    first_region.culture = 15;
    first_region.habitat = 16;
    first_region.development = 17;
    first_region.max_development = 18;
    first_region.town = Some(TownInfo {
        name: Some("Town".to_owned()),
        population: 19,
        habitat: 20,
        ..TownInfo::default()
    });
    first_region.visited = 1;
    first_region.products = vec![
        Production {
            item_type: ItemId::Silver as i32,
            amount: 17,
            base_amount: 12,
            skill: skill_type as i32,
            productivity: 9,
            ..Production::default()
        },
        Production {
            item_type: ItemId::Wood as i32,
            amount: 4,
            base_amount: 1,
            skill: -1,
            productivity: 6,
            ..Production::default()
        },
    ];
    first_region.markets = vec![
        Market {
            market_type: MarketType::Buy,
            item: ItemId::Wood as i32,
            price: 11,
            amount: 12,
            min_population: 13,
            max_population: 14,
            min_amount: 15,
            max_amount: 16,
            base_price: 17,
            ..Market::default()
        },
        Market {
            market_type: MarketType::Sell,
            item: -1,
            ..Market::default()
        },
    ];
    let mut second_region = Region::default();
    second_region.name = "Second".to_owned();
    second_region.num = 5;
    second_region.terrain_type = Some(0);
    second_region.xloc = 1;
    second_region.yloc = 1;
    let first_region_id = game.add_region(first_region.clone());
    let second_region_id = game.add_region(second_region);
    game.region_arrays[array_id]
        .set_region(0, 0, first_region_id)
        .expect("first region cell");
    game.region_arrays[array_id]
        .set_region(1, 1, second_region_id)
        .expect("second region cell");
    game.region_mut(first_region_id).unwrap().neighbors[0] = Some(second_region_id);
    game.region_mut(second_region_id).unwrap().neighbors[0] = Some(first_region_id);

    let mut fleet = Object::default();
    fleet.num = 12;
    fleet.object_type = O_FLEET as i32;
    fleet.incomplete = 1;
    fleet.name = Some("Fleet".to_owned());
    fleet.describe = Some("Fleet description".to_owned());
    fleet.inner = 7;
    fleet.prevdir = 5;
    fleet.runes = 6;
    fleet.ships.push(Item {
        item_type: ItemId::Longboat,
        num: 3,
        selling: 0,
        checked: false,
    });
    let fleet_id = game
        .add_object(first_region_id, fleet)
        .expect("add fleet object");

    let mut unit = Unit::new(44, Some(faction_id), 0, 0);
    unit.name = Some("Unit 44".to_owned());
    unit.describe = Some("Unit description".to_owned());
    unit.unit_type = UnitType::Mage;
    unit.guard = GuardMode::Avoid;
    unit.reveal = 2;
    unit.free = 1;
    unit.ready_item = ItemId::Wood as i32;
    unit.ready_weapon = [ItemId::Sword as i32, -1, -1, -1];
    unit.ready_armor = [ItemId::ChainArmor as i32, -1, -1, -1];
    unit.flags = 0x123;
    unit.items = ItemList::default();
    unit.items.push(Item {
        item_type: ItemId::Wood,
        num: 2,
        selling: 0,
        checked: false,
    });
    unit.items.push(Item {
        item_type: ItemId::Iwolf,
        num: 3,
        selling: 0,
        checked: false,
    });
    unit.skills = SkillList::default();
    unit.skills.push(Skill {
        skill_type,
        days: 10,
        exp: 5,
    });
    unit.combat = skill_type as i32;
    unit.saved_movement = 5;
    unit.saved_move_dir = 2;
    unit.visited.insert("zeta".to_owned());
    unit.visited.insert("alpha".to_owned());
    game.add_unit(fleet_id, unit).expect("add unit to fleet");

    let mut second_object = Object::default();
    second_object.num = 4;
    second_object.object_type = O_DUMMY as i32;
    second_object.name = Some("Warehouse".to_owned());
    second_object.ships.push(Item {
        item_type: ItemId::Longboat,
        num: 1,
        selling: 0,
        checked: false,
    });
    game.add_object(first_region_id, second_object)
        .expect("add second object");

    let mut loaded = LoadedGame {
        game,
        ruleset,
        seed,
        rng: Rng::new(seed),
    };
    let output = save_legacy(&mut loaded).expect("write legacy save");

    assert!(output.contains("2\n1\n2\n2\nTest level\n1\n1\n"));
    assert!(output.contains(&format!("First\n40\n{terrain}\n13\n3\n8\nPEAS\n")));
    assert!(output.contains("Second\n5\n"));
    assert!(output.contains(&format!("2\nSILV\n17\n12\nMINI\n9\nWOOD\n4\n1\n6\n")));
    assert!(output
        .contains("2\n0\nWOOD\n11\n12\n13\n14\n15\n16\n17\n1\nNO_ITEM\n0\n0\n0\n0\n0\n0\n0\n"));
    assert!(output.contains(&format!(
        "12\n{fleet_type}\n1\nFleet\nFleet description\n7\n-1\n6\n1\n"
    )));
    assert!(output.contains(&format!(
        "4\n{dummy_type}\n0\nWarehouse\nnone\n-1\n-1\n0\n0\n"
    )));
    assert!(output.contains(&format!(
        "4\n{dummy_type}\n0\nWarehouse\nnone\n-1\n-1\n0\n0\n1\n1 {longboat}\n"
    )));
    assert!(output.contains(&format!(
        "Unit 44\nUnit description\n44\n1\n12\n2\n2\n1\nWOOD\n{sword}\n"
    )));
    assert!(output.contains("2\n2 WOOD\n3 iWOLF\n1\nMINI 10\nMINI\n5\n2\n2\nalpha\nzeta\n"));
    assert!(output.contains("Neighbors\n5\n-1\n-1\n-1\n-1\n-1\n40\n-1\n-1\n-1\n-1\n-1\n0\n"));
    let first_region_position = output
        .find(&format!("First\n40\n{terrain}\n"))
        .expect("first region serialized");
    let second_region_position = output
        .find("Second\n5\n")
        .expect("second region serialized");
    assert!(first_region_position < second_region_position);
}

#[test]
fn legacy_writer_writes_all_quest_variants_and_sorted_destinations() {
    let ruleset = <dyn Ruleset>::from_name("Wyreth").expect("standard ruleset");
    let dummy_type = ruleset.tables().objects[O_DUMMY].name;
    let mut game = Game::default();

    let mut slay = Quest {
        quest_type: 0,
        target: 10,
        ..Quest::default()
    };
    slay.rewards.push(Item {
        item_type: ItemId::Wood,
        num: 2,
        selling: 0,
        checked: false,
    });

    let harvest = Quest {
        quest_type: 1,
        objective_type: ItemId::Wood as i32,
        objective_num: 3,
        region_num: 20,
        ..Quest::default()
    };
    let build = Quest {
        quest_type: 2,
        building: O_DUMMY as i32,
        region_name: "The East".to_owned(),
        ..Quest::default()
    };
    let mut visit = Quest {
        quest_type: 3,
        building: O_DUMMY as i32,
        ..Quest::default()
    };
    visit.destinations.insert("zeta".to_owned());
    visit.destinations.insert("alpha".to_owned());
    let demolish = Quest {
        quest_type: 5,
        target: 11,
        region_num: 22,
        ..Quest::default()
    };
    let mut default = Quest {
        quest_type: 4,
        target: 12,
        objective_type: ItemId::Wood as i32,
        objective_num: 4,
        building: O_DUMMY as i32,
        region_num: 23,
        region_name: "Last City".to_owned(),
        ..Quest::default()
    };
    default.destinations.insert("zeta".to_owned());
    default.destinations.insert("alpha".to_owned());
    game.quests = vec![slay, harvest, build, visit, demolish, default];

    let mut loaded = LoadedGame {
        game,
        ruleset,
        seed: 7,
        rng: Rng::new(7),
    };
    let output = save_legacy(&mut loaded).expect("write quest save");
    let quest_records = output
        .split_once("Neighbors\n")
        .expect("regions end with neighbors")
        .1;
    assert_eq!(
        quest_records,
        format!(
            "6\n\
             0\n10\n1\n2 WOOD\n\
             1\n3 WOOD\n20\n0\n\
             2\n{dummy_type}\nThe East\n0\n\
             3\n{dummy_type}\n2\nalpha\nzeta\n0\n\
             5\n11\n22\n0\n\
             4\n12\n4 WOOD\n{dummy_type}\n23\nLast City\n2\nalpha\nzeta\n0\n"
        )
    );
}

#[test]
fn legacy_writer_roundtrips_every_fixture_after_cpp_load_normalization() {
    for (index, (ruleset, input)) in GAME_FIXTURES.iter().enumerate() {
        let mut loaded = load_legacy(input)
            .unwrap_or_else(|error| panic!("fixture {index} ({ruleset}) failed to load: {error}"));
        if index == 0 {
            let normalized_region = loaded
                .game
                .region_ids()
                .iter()
                .filter_map(|region_id| loaded.game.region(*region_id))
                .find(|region| region.num == 451)
                .expect("standard turn-0 region 451");
            assert_eq!(normalized_region.building_seq, 1);
        }
        let first_output = save_legacy(&mut loaded)
            .unwrap_or_else(|error| panic!("fixture {index} ({ruleset}) failed to save: {error}"));
        let first_seed = first_output
            .lines()
            .nth(6)
            .expect("saved game has a seed header");
        let mut normalized = load_legacy(&first_output).unwrap_or_else(|error| {
            panic!("fixture {index} ({ruleset}) normalized save failed to load: {error}")
        });
        let second_output = save_legacy(&mut normalized).unwrap_or_else(|error| {
            panic!("fixture {index} ({ruleset}) normalized save failed to write: {error}")
        });
        let expected = replace_line_bytes(&second_output, 6, first_seed);
        if first_output != expected {
            let offset = first_output
                .bytes()
                .zip(expected.bytes())
                .position(|(actual, expected)| actual != expected)
                .unwrap_or(first_output.len().min(expected.len()));
            let start = offset.saturating_sub(40);
            let output_end = (offset + 40).min(first_output.len());
            let expected_end = (offset + 40).min(expected.len());
            panic!(
                "fixture {index} ({ruleset}) normalized save changes at byte {offset}; first {:?}, second {:?}",
                String::from_utf8_lossy(&first_output.as_bytes()[start..output_end]),
                String::from_utf8_lossy(&expected.as_bytes()[start..expected_end])
            );
        }
    }
}

#[test]
fn legacy_header_selects_ruleset_and_rejects_incompatible_versions() {
    let mut standard = load_legacy(STANDARD_GAME).expect("load standard fixture");
    assert_eq!(standard.ruleset.name(), "Wyreth");
    assert_eq!(standard.seed, 6456);
    assert_eq!(
        standard.rng.get_random(10_000),
        Rng::new(6456).get_random(10_000)
    );

    let new_origins = load_legacy(NEW_ORIGINS_GAME).expect("load neworigins fixture");
    assert_eq!(new_origins.ruleset.name(), "NewOrigins");
    assert_eq!(new_origins.seed, 3070);

    let old_engine = replace_line(STANDARD_GAME, 1, "262661");
    assert!(matches!(
        load_legacy(&old_engine),
        Err(SaveError::IncompatibleEngineVersion { .. })
    ));

    let newer_engine_patch = replace_line(STANDARD_GAME, 1, "328198");
    assert!(matches!(
        load_legacy(&newer_engine_patch),
        Err(SaveError::NewerEnginePatch { .. })
    ));

    let unknown_ruleset = replace_line(STANDARD_GAME, 2, "Unknown");
    assert!(matches!(
        load_legacy(&unknown_ruleset),
        Err(SaveError::UnsupportedRuleset { .. })
    ));

    let old_ruleset_major = replace_line(STANDARD_GAME, 3, "65536");
    assert!(matches!(
        load_legacy(&old_ruleset_major),
        Err(SaveError::RulesetUpgradeRejected { .. })
    ));
}

#[test]
fn legacy_faction_types_follow_the_selected_ruleset() {
    for (input, expected_count, expected_first_type) in
        [(STANDARD_GAME, 3, "War"), (NEW_ORIGINS_GAME, 2, "Martial")]
    {
        let loaded = load_legacy(input).expect("load fixture");
        for faction_id in loaded.game.faction_ids() {
            let faction = loaded.game.faction(*faction_id).expect("faction exists");
            assert_eq!(faction.faction_types.len(), expected_count);
            assert_eq!(faction.faction_types[0].0, expected_first_type);
            assert!(faction
                .attitudes
                .iter()
                .all(|attitude| attitude.faction_num != faction.num));
        }
    }
}

#[test]
fn legacy_regions_read_conditional_fields_and_preserve_order() {
    let standard = load_legacy(STANDARD_GAME).expect("load standard fixture");
    assert_eq!(standard.game.region_ids().len(), 649);
    assert_eq!(standard.game.region_arrays.len(), 4);
    assert_eq!(standard.game.number_of_gates, 14);
    assert_eq!((standard.game.year, standard.game.month), (1, -1));
    let first_region_id = standard.game.region_ids()[0];
    let first_region = standard.game.region(first_region_id).expect("first region");
    assert_eq!(first_region.name, "Unnamed");
    assert_eq!(first_region.num, 0);

    let mut saw_town = false;
    let mut saw_buy_market = false;
    let mut saw_sell_market = false;
    let mut saw_product = false;
    let mut saw_neighbor = false;
    for region_id in standard.game.region_ids() {
        let region = standard.game.region(*region_id).expect("region exists");
        if let Some(town) = &region.town {
            saw_town = true;
            assert!(town.name.is_some());
            assert!((0..=100).contains(&town.development));
        }
        for market in &region.markets {
            match market.market_type {
                MarketType::Buy => saw_buy_market = true,
                MarketType::Sell => saw_sell_market = true,
            }
        }
        saw_product |= !region.products.is_empty();
        saw_neighbor |= region.neighbors.iter().any(Option::is_some);
        for neighbor in region.neighbors.iter().flatten() {
            assert!(standard.game.region(*neighbor).is_some());
        }
        for object_id in region.object_ids() {
            let object = standard.game.object(*object_id).expect("object exists");
            assert_eq!(object.region, Some(*region_id));
        }
    }
    assert!(saw_town);
    assert!(saw_buy_market);
    assert!(saw_sell_market);
    assert!(saw_product);
    assert!(saw_neighbor);

    let new_origins = load_legacy(NEW_ORIGINS_GAME).expect("load neworigins fixture");
    assert_eq!(new_origins.game.region_ids().len(), 161);
    assert_eq!(new_origins.game.region_arrays.len(), 3);
    assert_eq!(new_origins.game.number_of_gates, 2);
}

#[test]
fn legacy_lists_drop_disabled_items_and_empty_skills() {
    for (_, input) in GAME_FIXTURES {
        let loaded = load_legacy(input).expect("load fixture");
        let tables = loaded.ruleset.tables();
        for faction_id in loaded.game.faction_ids() {
            let faction = loaded.game.faction(*faction_id).expect("faction exists");
            for item in faction.items.entries() {
                assert_eq!(
                    tables.item_families.items[item.item_type.index()].flags & ItemFlags::DISABLED,
                    0
                );
                assert!(item.num >= 1);
            }
            assert!(faction
                .skills
                .entries()
                .iter()
                .all(|skill| skill.days != 0 || skill.exp != 0));
        }
        for region_id in loaded.game.region_ids() {
            let region = loaded.game.region(*region_id).expect("region exists");
            for object_id in region.object_ids() {
                let object = loaded.game.object(*object_id).expect("object exists");
                for unit_id in object.unit_ids() {
                    let unit = loaded.game.unit(*unit_id).expect("unit exists");
                    assert!(unit.items.entries().iter().all(|item| {
                        item.num >= 1
                            && tables.item_families.items[item.item_type.index()].flags
                                & ItemFlags::DISABLED
                                == 0
                    }));
                    assert!(unit
                        .skills
                        .entries()
                        .iter()
                        .all(|skill| skill.days != 0 || skill.exp != 0));
                    assert!(!matches!(unit.guard, GuardMode::Set | GuardMode::Advance));
                }
            }
        }
    }
}

#[test]
fn legacy_units_normalize_guard_modes_and_restore_memberships() {
    for (_, input) in GAME_FIXTURES {
        let loaded = load_legacy(input).expect("load fixture");
        for region_id in loaded.game.region_ids() {
            let region = loaded.game.region(*region_id).expect("region exists");
            for object_id in region.object_ids() {
                let object = loaded.game.object(*object_id).expect("object exists");
                for unit_id in object.unit_ids() {
                    let unit = loaded.game.unit(*unit_id).expect("unit exists");
                    assert_eq!(unit.object, Some(*object_id));
                    assert!(unit
                        .faction
                        .is_some_and(|faction| loaded.game.faction(faction).is_some()));
                    assert_eq!(loaded.game.get_unit_id(unit.num), Some(*unit_id));
                    assert!(!matches!(unit.guard, GuardMode::Set | GuardMode::Advance));
                }
            }
        }
    }
}

#[test]
fn all_28_recorded_game_inputs_load_with_spot_checked_state() {
    for (index, (ruleset, input)) in GAME_FIXTURES.iter().enumerate() {
        let loaded = load_legacy(input)
            .unwrap_or_else(|error| panic!("fixture {index} ({ruleset}) failed: {error}"));
        let expected_ruleset = match *ruleset {
            "standard" => "Wyreth",
            "neworigins" => "NewOrigins",
            _ => unreachable!("fixture ruleset is declared above"),
        };
        assert_eq!(loaded.ruleset.name(), expected_ruleset, "fixture {index}");
        assert!(
            !loaded.game.region_ids().is_empty(),
            "fixture {index} has no regions"
        );
        let faction_type_count = if *ruleset == "standard" { 3 } else { 2 };
        for faction_id in loaded.game.faction_ids() {
            let faction = loaded.game.faction(*faction_id).expect("faction exists");
            assert_eq!(faction.faction_types.len(), faction_type_count);
            assert!(faction
                .attitudes
                .iter()
                .all(|attitude| attitude.faction_num != faction.num));
        }
    }

    let standard = load_legacy(STANDARD_GAME).expect("load standard turn 0");
    assert_eq!(standard.game.faction_ids().len(), 2);
    assert_eq!(standard.game.region_ids().len(), 649);
    assert_eq!(standard.game.region_arrays.len(), 4);

    let new_origins = load_legacy(NEW_ORIGINS_GAME).expect("load neworigins turn 0");
    assert_eq!(new_origins.game.faction_ids().len(), 2);
    assert_eq!(new_origins.game.region_ids().len(), 161);
    assert_eq!(new_origins.game.region_arrays.len(), 3);

    for index in 0..ItemId::COUNT {
        assert_eq!(
            ItemId::from_index(index).expect("valid item index").index(),
            index
        );
    }
    assert_eq!(ItemId::from_index(ItemId::COUNT), None);
}
