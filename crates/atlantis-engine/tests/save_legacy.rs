use atlantis_engine::{
    defs::{ItemFlags, ItemId},
    model::{GuardMode, MarketType},
    rng::Rng,
    save::{load_legacy, SaveError},
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
