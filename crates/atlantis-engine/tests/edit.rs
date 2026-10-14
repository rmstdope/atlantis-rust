use atlantis_engine::{
    defs::{objects::O_TOWER, terrains::R_FOREST, ItemFlags, ItemId, ItemTypeFlags},
    edit::{
        add_or_regenerate_town, add_region_gate, add_region_object, create_unit, find_region,
        find_unit, move_unit_to_region, regenerate_markets, regenerate_region,
        regenerate_region_products, remove_market, remove_region_gate, remove_region_object,
        remove_town, rename_region, rename_region_object, rename_town, rename_unit,
        set_market_amount_range, set_market_population_range, set_market_prices, set_region_race,
        set_region_terrain, set_region_wages, set_unit_description, set_unit_faction,
        set_unit_guard, set_unit_item, set_unit_skill, set_unit_type, swap_market_type,
    },
    model::{Game, GuardMode, MarketType, Object, ObjectId, RegionId, Unit, UnitId, UnitType},
    save::{load_legacy, save_legacy, LoadedGame},
};

const STANDARD_GAME: &str = include_str!("../../../fixtures/standard/turn_0/game.in");
const NEWORIGINS_GAME: &str = include_str!("../../../fixtures/neworigins/turn_0/game.in");

fn loaded_game() -> LoadedGame {
    load_legacy(STANDARD_GAME).expect("load standard fixture")
}

fn loaded_neworigins_game() -> LoadedGame {
    load_legacy(NEWORIGINS_GAME).expect("load neworigins fixture")
}

fn first_region_with_unit(game: &Game) -> (RegionId, ObjectId, UnitId) {
    game.region_ids()
        .iter()
        .copied()
        .find_map(|region_id| {
            let region = game.region(region_id)?;
            region.object_ids().iter().copied().find_map(|object_id| {
                let object = game.object(object_id)?;
                object
                    .unit_ids()
                    .first()
                    .copied()
                    .map(|unit_id| (region_id, object_id, unit_id))
            })
        })
        .expect("fixture has a unit in a region object")
}

fn first_region_with_dummy(game: &Game) -> RegionId {
    game.region_ids()
        .iter()
        .copied()
        .find(|region_id| game.get_dummy(*region_id).is_some())
        .expect("fixture has a region dummy")
}

fn first_populated_region(game: &Game) -> RegionId {
    game.region_ids()
        .iter()
        .copied()
        .find(|region_id| {
            game.region(*region_id)
                .is_some_and(|region| region.terrain_type.is_some() && region.population > 0)
        })
        .expect("fixture has a populated region")
}

#[test]
fn find_region_uses_region_array_coordinates() {
    let loaded = loaded_game();
    let (region_id, region) = loaded
        .game
        .region_ids()
        .iter()
        .copied()
        .find_map(|id| {
            let region = loaded.game.region(id)?;
            (loaded
                .game
                .get_region_at(region.xloc, region.yloc, region.zloc)
                == Some(id))
            .then_some((id, region))
        })
        .expect("fixture has a region in an array");

    assert_eq!(
        find_region(&loaded.game, region.xloc, region.yloc, region.zloc),
        Some(region_id)
    );
    assert_eq!(find_region(&loaded.game, 99_999, 99_999, 99_999), None);
}

#[test]
fn find_unit_uses_legacy_number_index() {
    let loaded = loaded_game();
    let (_, _, unit_id) = first_region_with_unit(&loaded.game);
    let number = loaded.game.unit(unit_id).expect("fixture unit").num;

    assert_eq!(find_unit(&loaded.game, number), Some(unit_id));
    assert_eq!(find_unit(&loaded.game, -1), None);
}

#[test]
fn create_unit_uses_active_rules_and_region_dummy() {
    let mut loaded = loaded_game();
    let faction_id = loaded.game.faction_ids()[0];
    let region_id = first_region_with_dummy(&loaded.game);
    let dummy_id = loaded.game.get_dummy(region_id).expect("region dummy");
    let movement_points =
        loaded.ruleset.game_defs().phased_move_offset % loaded.ruleset.game_defs().max_speed;

    let unit_id = create_unit(&mut loaded, faction_id, region_id).expect("create unit");
    let unit = loaded.game.unit(unit_id).expect("created unit");

    assert_eq!(unit.faction, Some(faction_id));
    assert_eq!(unit.object, Some(dummy_id));
    assert_eq!(unit.items.get_num(ItemId::Leaders), 1);
    assert_eq!(unit.reveal, 2);
    assert_eq!(unit.movepoints, movement_points);
    assert!(unit.num > 0);
    assert_eq!(
        loaded.game.object(dummy_id).unwrap().unit_ids().last(),
        Some(&unit_id)
    );
}

#[test]
fn region_object_edits_preserve_list_order() {
    let mut loaded = loaded_game();
    let region_id = first_region_with_dummy(&loaded.game);
    let old_ids = loaded.game.region(region_id).unwrap().object_ids().to_vec();

    let object_id = add_region_object(&mut loaded, region_id, O_TOWER).expect("add tower");
    let object_index = loaded
        .game
        .region(region_id)
        .unwrap()
        .object_ids()
        .iter()
        .position(|id| *id == object_id)
        .expect("new object is in region list");
    let object = loaded.game.object(object_id).unwrap();
    let number = object.num;
    assert_eq!(
        object.name.as_deref(),
        Some(format!("Building [{number}]").as_str())
    );

    rename_region_object(&mut loaded, region_id, object_index, "Great tower")
        .expect("rename tower");
    assert_eq!(
        loaded.game.object(object_id).unwrap().name.as_deref(),
        Some(format!("Great tower [{number}]").as_str())
    );

    assert_eq!(
        remove_region_object(&mut loaded, region_id, object_index).expect("remove tower"),
        object_id
    );
    assert_eq!(loaded.game.region(region_id).unwrap().object_ids(), old_ids);
    assert!(loaded.game.object(object_id).is_some());
}

#[test]
fn region_terrain_race_wages_and_name_edits_validate_values() {
    let mut loaded = loaded_game();
    let region_id = first_region_with_dummy(&loaded.game);
    let race = loaded
        .ruleset
        .tables()
        .item_families
        .items
        .iter()
        .enumerate()
        .find(|(_, item)| {
            item.item_type & ItemTypeFlags::MAN != 0 && item.flags & ItemFlags::DISABLED == 0
        })
        .and_then(|(index, _)| ItemId::from_index(index))
        .expect("ruleset has an enabled race");

    set_region_terrain(&mut loaded, region_id, R_FOREST).expect("set forest terrain");
    set_region_race(&mut loaded, region_id, Some(race)).expect("set local race");
    rename_region(&mut loaded, region_id, "Edited region").expect("rename region");
    set_region_wages(&mut loaded, region_id, 0).expect("leave wages unchanged");

    let region = loaded.game.region(region_id).unwrap();
    assert_eq!(region.terrain_type, Some(R_FOREST));
    assert_eq!(region.race, race as i32);
    assert_eq!(region.name, "Edited region");

    assert!(set_region_terrain(&mut loaded, region_id, usize::MAX).is_err());
    assert!(set_region_race(&mut loaded, region_id, Some(ItemId::Iron)).is_err());
}

#[test]
fn region_product_regeneration_keeps_silver_and_uses_the_loaded_rng() {
    let mut first = loaded_game();
    let mut second = loaded_game();
    let first_region = first_region_with_dummy(&first.game);
    let second_region = first_region_with_dummy(&second.game);
    for loaded in [&mut first, &mut second] {
        let region = loaded
            .game
            .region_mut(first_region_with_dummy(&loaded.game))
            .unwrap();
        region.products.clear();
        region.products.extend([
            atlantis_engine::model::Production {
                item_type: ItemId::Silver as i32,
                skill: -1,
                ..Default::default()
            },
            atlantis_engine::model::Production {
                item_type: ItemId::Wood as i32,
                ..Default::default()
            },
        ]);
    }

    regenerate_region_products(&mut first, first_region).expect("regenerate products");
    regenerate_region_products(&mut second, second_region).expect("repeat seeded regeneration");

    assert_eq!(
        first.game.region(first_region).unwrap().products,
        second.game.region(second_region).unwrap().products
    );
    assert!(first
        .game
        .region(first_region)
        .unwrap()
        .products
        .iter()
        .any(|product| product.item_type == ItemId::Silver as i32));
}

#[test]
fn full_region_regeneration_is_repeatable_for_the_loaded_rng() {
    let mut first = loaded_game();
    let mut second = loaded_game();
    let region_id = first_populated_region(&first.game);

    regenerate_region(&mut first, region_id).expect("regenerate region");
    regenerate_region(&mut second, region_id).expect("repeat seeded regeneration");

    assert_eq!(first.game.region(region_id), second.game.region(region_id));
}

#[test]
fn gate_and_town_edits_update_region_state() {
    let mut loaded = loaded_game();
    let region_id = first_region_with_dummy(&loaded.game);
    let original_gate_count = loaded.game.number_of_gates;

    add_region_gate(&mut loaded, region_id).expect("add gate");
    assert!(loaded.game.region(region_id).unwrap().gate > 0);
    assert_eq!(loaded.game.number_of_gates, original_gate_count + 1);
    remove_region_gate(&mut loaded, region_id).expect("remove gate");
    assert_eq!(loaded.game.region(region_id).unwrap().gate, 0);
    assert_eq!(loaded.game.number_of_gates, original_gate_count);

    add_or_regenerate_town(&mut loaded, region_id).expect("add town");
    assert!(loaded.game.region(region_id).unwrap().town.is_some());
    rename_town(&mut loaded, region_id, "New town").expect("rename town");
    assert_eq!(
        loaded
            .game
            .region(region_id)
            .unwrap()
            .town
            .as_ref()
            .unwrap()
            .name
            .as_deref(),
        Some("New town")
    );
    add_or_regenerate_town(&mut loaded, region_id).expect("regenerate town");
    assert_eq!(
        loaded
            .game
            .region(region_id)
            .unwrap()
            .town
            .as_ref()
            .unwrap()
            .name
            .as_deref(),
        Some("New town")
    );
    remove_town(&mut loaded, region_id).expect("remove town");
    assert!(loaded.game.region(region_id).unwrap().town.is_none());
}

#[test]
fn town_creation_removes_lairs_and_empties_their_unit_lists() {
    let mut loaded = loaded_game();
    let region_id = first_region_with_dummy(&loaded.game);
    let lair_type = loaded
        .ruleset
        .tables()
        .objects
        .iter()
        .enumerate()
        .find(|(_, object)| {
            object.monster != -1
                && object.flags & atlantis_engine::defs::objects::object_flags::CANENTER == 0
        })
        .map(|(index, _)| index as i32)
        .expect("ruleset has a non-enterable monster lair");
    let mut lair = Object::default();
    lair.object_type = lair_type;
    let lair_id = loaded
        .game
        .add_object(region_id, lair)
        .expect("add monster lair");
    let unit_id = loaded
        .game
        .add_unit(lair_id, Unit::default())
        .expect("add lair unit");

    add_or_regenerate_town(&mut loaded, region_id).expect("add town");

    assert!(!loaded
        .game
        .region(region_id)
        .unwrap()
        .object_ids()
        .contains(&lair_id));
    assert!(loaded.game.object(lair_id).unwrap().unit_ids().is_empty());
    assert_eq!(loaded.game.unit(unit_id).unwrap().object, Some(lair_id));
}

#[test]
fn market_edits_update_swap_and_remove_matching_items() {
    let mut loaded = loaded_game();
    let region_id = first_region_with_dummy(&loaded.game);

    set_market_population_range(&mut loaded, region_id, ItemId::Wood, 100, 500)
        .expect("create market population bounds");
    set_market_amount_range(&mut loaded, region_id, ItemId::Wood, 3, 30)
        .expect("set market amount bounds");
    set_market_prices(&mut loaded, region_id, ItemId::Wood, 12, 10).expect("set market prices");

    let markets = &loaded.game.region(region_id).unwrap().markets;
    let wood = markets
        .iter()
        .find(|market| market.item == ItemId::Wood as i32)
        .expect("wood market");
    assert_eq!((wood.min_population, wood.max_population), (100, 500));
    assert_eq!((wood.min_amount, wood.max_amount), (3, 30));
    assert_eq!((wood.price, wood.base_price), (12, 10));
    assert_eq!(wood.market_type, MarketType::Sell);

    swap_market_type(&mut loaded, region_id, ItemId::Wood).expect("swap market side");
    assert_eq!(
        loaded
            .game
            .region(region_id)
            .unwrap()
            .markets
            .iter()
            .find(|market| market.item == ItemId::Wood as i32)
            .unwrap()
            .market_type,
        MarketType::Buy
    );
    assert_eq!(
        remove_market(&mut loaded, region_id, ItemId::Wood).unwrap(),
        1
    );
    assert!(!loaded
        .game
        .region(region_id)
        .unwrap()
        .markets
        .iter()
        .any(|market| market.item == ItemId::Wood as i32));
}

#[test]
fn market_regeneration_rebuilds_a_town_market_deterministically() {
    let mut first = loaded_game();
    let mut second = loaded_game();
    let region_id = first_region_with_dummy(&first.game);

    for loaded in [&mut first, &mut second] {
        add_or_regenerate_town(loaded, region_id).expect("create town");
    }
    regenerate_markets(&mut first, region_id).expect("regenerate markets");
    regenerate_markets(&mut second, region_id).expect("repeat seeded regeneration");

    assert_eq!(
        first.game.region(region_id).unwrap().markets,
        second.game.region(region_id).unwrap().markets
    );
}

#[test]
fn unit_edits_move_membership_and_round_trip_through_legacy_save() {
    let mut loaded = loaded_game();
    let (source_region, source_object, unit_id) = first_region_with_unit(&loaded.game);
    let target_region = loaded
        .game
        .region_ids()
        .iter()
        .copied()
        .find(|region_id| {
            *region_id != source_region && loaded.game.get_dummy(*region_id).is_some()
        })
        .expect("fixture has another region dummy");
    let target_object = loaded.game.get_dummy(target_region).unwrap();
    let unit_number = loaded.game.unit(unit_id).unwrap().num;
    let faction_ids = loaded.game.faction_ids().to_vec();
    let men = loaded
        .game
        .unit(unit_id)
        .unwrap()
        .items
        .entries()
        .iter()
        .filter(|entry| {
            loaded.ruleset.tables().item_families.items[entry.item_type.index()].item_type
                & ItemTypeFlags::MAN
                != 0
        })
        .map(|entry| entry.num)
        .sum::<i32>();
    let expected_skill_days = 30 * men as u32;
    let skill = loaded
        .ruleset
        .tables()
        .skills
        .skills
        .iter()
        .position(|skill| skill.flags & atlantis_engine::defs::skills::skill_flags::DISABLED == 0)
        .expect("ruleset has an enabled skill");

    set_unit_item(&mut loaded, unit_id, ItemId::Wood, 7).expect("set unit item");
    set_unit_faction(&mut loaded, unit_id, faction_ids[1]).expect("change unit faction");
    set_unit_skill(&mut loaded, unit_id, skill, 30).expect("set unit skill");
    set_unit_type(&mut loaded, unit_id, UnitType::Guard).expect("change unit type");
    set_unit_guard(&mut loaded, unit_id, GuardMode::Guard).expect("set guard mode");
    rename_unit(&mut loaded, unit_id, "Edited unit").expect("rename unit");
    set_unit_description(&mut loaded, unit_id, Some("Edited description"))
        .expect("set description");
    move_unit_to_region(&mut loaded, unit_id, target_region).expect("move unit");

    let unit = loaded.game.unit(unit_id).unwrap();
    assert_eq!(unit.num, unit_number);
    assert_eq!(unit.items.get_num(ItemId::Wood), 7);
    assert_eq!(unit.skills.get_days(skill), expected_skill_days);
    assert_eq!(unit.faction, Some(faction_ids[1]));
    assert_eq!(unit.unit_type, UnitType::Guard);
    assert_eq!(unit.guard, GuardMode::Guard);
    assert_eq!(unit.name.as_deref(), Some("Edited unit"));
    assert_eq!(unit.describe.as_deref(), Some("Edited description"));
    assert_eq!(unit.object, Some(target_object));
    assert!(!loaded
        .game
        .object(source_object)
        .unwrap()
        .unit_ids()
        .contains(&unit_id));

    let output = save_legacy(&mut loaded).expect("write edited legacy save");
    let reloaded = load_legacy(&output).expect("load edited legacy save");
    let saved_unit_id = find_unit(&reloaded.game, unit_number).expect("legacy unit number");
    let saved_unit = reloaded.game.unit(saved_unit_id).unwrap();
    assert_eq!(saved_unit.name.as_deref(), Some("Edited unit"));
    assert_eq!(saved_unit.items.get_num(ItemId::Wood), 7);
    assert_eq!(saved_unit.skills.get_days(skill), expected_skill_days);
    assert_eq!(saved_unit.faction, Some(faction_ids[1]));
    assert_eq!(saved_unit.unit_type, UnitType::Guard);
    assert_eq!(saved_unit.guard, GuardMode::Guard);
    assert_eq!(saved_unit.describe.as_deref(), Some("Edited description"));

    set_unit_item(&mut loaded, unit_id, ItemId::Wood, 0).expect("remove unit item");
    assert_eq!(
        loaded
            .game
            .unit(unit_id)
            .unwrap()
            .items
            .get_num(ItemId::Wood),
        0
    );
}

#[test]
fn edits_round_trip_with_neworigins_tables_and_legacy_save() {
    let mut loaded = loaded_neworigins_game();
    let (_, _, unit_id) = first_region_with_unit(&loaded.game);
    let unit_number = loaded.game.unit(unit_id).unwrap().num;
    let region_id = first_populated_region(&loaded.game);
    let item = loaded
        .ruleset
        .tables()
        .item_families
        .items
        .iter()
        .enumerate()
        .find(|(_, item)| item.flags & ItemFlags::DISABLED == 0)
        .and_then(|(index, _)| ItemId::from_index(index))
        .expect("neworigins has an enabled item");

    set_unit_item(&mut loaded, unit_id, item, 4).expect("set neworigins unit item");
    regenerate_region_products(&mut loaded, region_id).expect("regenerate neworigins products");

    let output = save_legacy(&mut loaded).expect("write neworigins legacy save");
    let reloaded = load_legacy(&output).expect("reload neworigins legacy save");
    let saved_unit_id = find_unit(&reloaded.game, unit_number).expect("neworigins unit number");

    assert_eq!(reloaded.ruleset.game_defs().ruleset_name, "NewOrigins");
    assert_eq!(
        reloaded
            .game
            .unit(saved_unit_id)
            .unwrap()
            .items
            .get_num(item),
        4
    );
}
