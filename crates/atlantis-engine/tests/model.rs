use atlantis_engine::defs::objects::{O_DUMMY, O_FLEET, O_TOWER};
use atlantis_engine::defs::ItemId;
use atlantis_engine::defs::OBJECT_DEFS;
use atlantis_engine::model::{
    Faction, Game, ItemList, Object, Region, RegionArray, RegionArrayError, RegionId,
    RegionLevelType, SkillList, Unit, UnitNumberingIssue,
};

#[test]
fn item_list_removing_then_readding_an_item_appends_it() {
    let mut items = ItemList::default();
    items.set_num(ItemId::Silver, 3);
    items.set_num(ItemId::Wood, 4);

    items.set_num(ItemId::Silver, 0);
    items.set_num(ItemId::Silver, 2);

    assert_eq!(
        items
            .entries()
            .iter()
            .map(|item| item.item_type)
            .collect::<Vec<_>>(),
        [ItemId::Wood, ItemId::Silver]
    );
}

#[test]
fn skill_list_removing_then_readding_a_skill_appends_it() {
    let mut skills = SkillList::default();
    skills.set(1, 10, 2);
    skills.set(2, 20, 3);

    skills.remove(1);
    skills.set(1, 5, 1);

    assert_eq!(
        skills
            .entries()
            .iter()
            .map(|skill| skill.skill_type)
            .collect::<Vec<_>>(),
        [2, 1]
    );
}

#[test]
fn turn_number_matches_the_cpp_year_month_formula() {
    let mut game = Game::default();
    game.year = 2;
    game.month = 0;

    assert_eq!(game.turn_number(), 13);
}

#[test]
fn entity_defaults_and_numbered_constructors_match_cpp_initialization() {
    let faction = Faction::default();
    assert_eq!(faction.exists, 1);
    assert_eq!(faction.last_change, -6);
    assert_eq!(faction.default_attitude, 2);
    assert_eq!(faction.name, None);

    let numbered_faction = Faction::new(9, &["war", "trade", "magic"]);
    assert_eq!(numbered_faction.num, 9);
    assert_eq!(
        numbered_faction.faction_types,
        [
            ("war".to_owned(), 1),
            ("trade".to_owned(), 1),
            ("magic".to_owned(), 1),
        ]
    );
    assert_eq!(numbered_faction.name.as_deref(), Some("Faction (9)"));
    assert_eq!(numbered_faction.times, 1);
    assert_eq!(numbered_faction.template_format, 2);

    let region = Region::default();
    assert_eq!(region.name, "Region");
    assert_eq!(region.building_seq, 1);
    assert_eq!(region.gate_open, 1);

    let object = Object::default();
    assert_eq!(object.name.as_deref(), Some("Dummy"));
    assert_eq!(object.inner, -1);
    assert_eq!(object.prevdir, -1);
    assert_eq!(Object::new(4).movepoints, 4);

    let unit = Unit::default();
    assert_eq!(unit.name, None);
    assert_eq!(unit.flags, 0x0002);
    assert_eq!(unit.combat, -1);
    assert_eq!(unit.ready_item, -1);
    assert_eq!(unit.ready_weapon, [-1; 4]);
    assert_eq!(unit.phase, -1);
    assert_eq!(unit.saved_move_dir, -1);
    assert_eq!(Unit::new(17, None, 0, 3).name.as_deref(), Some("Unit (17)"));
}

#[test]
fn memberships_preserve_insertion_order() {
    let mut game = Game::default();
    let first_faction = game.add_faction(Faction::default());
    let second_faction = game.add_faction(Faction::default());
    let region = game.add_region(Region::default());
    let first_object = game.add_object(region, Object::default()).unwrap();
    let second_object = game.add_object(region, Object::default()).unwrap();
    let first_unit = game.add_unit(first_object, Unit::default()).unwrap();
    let second_unit = game.add_unit(first_object, Unit::default()).unwrap();

    assert_eq!(game.faction_ids(), [first_faction, second_faction]);
    assert_eq!(
        game.region(region).unwrap().object_ids(),
        [first_object, second_object]
    );
    assert_eq!(
        game.object(first_object).unwrap().unit_ids(),
        [first_unit, second_unit]
    );
}

#[test]
fn region_lookup_wraps_coordinates_and_preserves_number_lookup() {
    let mut game = Game::default();
    let mut first_state = Region::default();
    first_state.num = 17;
    let first = game.add_region(first_state);
    let mut second_state = Region::default();
    second_state.num = 29;
    let second = game.add_region(second_state);
    let mut surface = RegionArray::new(4, 4, RegionLevelType::Surface).unwrap();
    surface.set_region(0, 0, first).unwrap();
    surface.set_region(3, 1, second).unwrap();
    game.add_region_array(surface);

    assert_eq!(game.get_region(17), Some(first));
    assert_eq!(game.get_region_at(4, 0, 0), Some(first));
    assert_eq!(game.get_region_at(-1, 1, 0), Some(second));
    assert_eq!(game.get_region_at(1, 0, 0), None);
    assert_eq!(game.get_region_at(0, 0, -1), None);
}

#[test]
fn region_lookup_preserves_cpp_single_add_wrapping_for_far_negative_coordinates() {
    let mut game = Game::default();
    let region = game.add_region(Region::default());
    let mut surface = RegionArray::new(4, 4, RegionLevelType::Surface).unwrap();
    surface.set_region(1, 1, region).unwrap();
    game.add_region_array(surface);

    assert_eq!(game.get_region_at(-5, 1, 0), Some(region));
}

#[test]
fn region_keeps_dummy_object_and_hell_units() {
    let mut game = Game::default();
    let region = game.add_region(Region::default());
    let first_dummy = game.add_object(region, Object::default()).unwrap();
    let mut tower = Object::default();
    tower.object_type = O_TOWER as i32;
    game.add_object(region, tower).unwrap();
    let dead_unit = game
        .add_hell_unit(
            region,
            Unit {
                num: 12,
                ..Unit::default()
            },
        )
        .unwrap();

    assert_eq!(game.get_dummy(region), Some(first_dummy));
    assert_eq!(game.region(region).unwrap().hell_unit_ids(), [dead_unit]);
}

#[test]
fn attaching_a_unit_moves_it_to_the_tail_of_the_new_object_list() {
    let mut game = Game::default();
    let region = game.add_region(Region::default());
    let first_object = game.add_object(region, Object::default()).unwrap();
    let second_object = game.add_object(region, Object::default()).unwrap();
    let first_unit = game.add_unit(first_object, Unit::default()).unwrap();
    let second_unit = game.add_unit(first_object, Unit::default()).unwrap();

    game.attach_unit(second_object, first_unit).unwrap();
    assert_eq!(game.object(first_object).unwrap().unit_ids(), [second_unit]);
    assert_eq!(game.object(second_object).unwrap().unit_ids(), [first_unit]);

    game.attach_unit(first_object, first_unit).unwrap();
    assert_eq!(
        game.object(first_object).unwrap().unit_ids(),
        [second_unit, first_unit]
    );
}

#[test]
fn object_is_fleet_for_fleet_type_sailors_or_ships() {
    let mut object = Object::default();
    assert_eq!(object.is_fleet(&OBJECT_DEFS), Some(false));

    object.object_type = O_FLEET as i32;
    assert_eq!(object.is_fleet(&OBJECT_DEFS), Some(true));

    object.object_type = O_DUMMY as i32;
    object.ships.set_num(ItemId::Longboat, 1);
    assert_eq!(object.is_fleet(&OBJECT_DEFS), Some(true));

    object.ships.set_num(ItemId::Longboat, 0);
    let sailors_object = OBJECT_DEFS
        .iter()
        .position(|definition| definition.sailors > 0)
        .expect("reference table contains a sailors-required object");
    object.object_type = sailors_object as i32;
    assert_eq!(object.is_fleet(&OBJECT_DEFS), Some(true));
}

#[test]
fn unit_number_setup_reassigns_duplicates_and_get_new_unit_reuses_the_lowest_free_number() {
    let mut game = Game::default();
    let region = game.add_region(Region::default());
    let object = game.add_object(region, Object::default()).unwrap();
    let first = game
        .add_unit(
            object,
            Unit {
                num: 5,
                ..Unit::default()
            },
        )
        .unwrap();
    let second = game
        .add_unit(
            object,
            Unit {
                num: 2,
                ..Unit::default()
            },
        )
        .unwrap();
    let duplicate = game
        .add_unit(
            object,
            Unit {
                num: 2,
                ..Unit::default()
            },
        )
        .unwrap();
    let invalid = game.add_unit(object, Unit::default()).unwrap();

    assert_eq!(game.setup_unit_seq().unwrap(), 6);
    assert_eq!(
        game.setup_unit_nums().unwrap(),
        [
            UnitNumberingIssue::Duplicate {
                unit: duplicate,
                duplicate_number: 2,
                reassigned_number: 6,
            },
            UnitNumberingIssue::Invalid {
                unit: invalid,
                invalid_number: 0,
                reassigned_number: 7,
            },
        ]
    );
    assert_eq!(game.unit_seq, 8);
    assert_eq!(game.get_unit_id(2), Some(second));
    assert_eq!(game.get_unit_id(5), Some(first));
    assert_eq!(game.get_unit_id(6), Some(duplicate));
    assert_eq!(game.get_unit_id(7), Some(invalid));

    let new_unit = game.get_new_unit(None, 0, 3).unwrap();
    let next_unit = game.get_new_unit(None, 0, 3).unwrap();
    let last_gap = game.get_new_unit(None, 0, 3).unwrap();
    let appended_unit = game.get_new_unit(None, 0, 3).unwrap();
    assert_eq!(game.unit(new_unit).unwrap().num, 1);
    assert_eq!(
        game.unit(new_unit).unwrap().name.as_deref(),
        Some("Unit (1)")
    );
    assert_eq!(game.unit(new_unit).unwrap().movepoints, 3);
    assert_eq!(game.unit(next_unit).unwrap().num, 3);
    assert_eq!(game.unit(last_gap).unwrap().num, 4);
    assert_eq!(game.unit(appended_unit).unwrap().num, 8);
    game.attach_unit(object, new_unit).unwrap();
    assert_eq!(
        game.object(object).unwrap().unit_ids().last(),
        Some(&new_unit)
    );
}

#[test]
fn region_array_rejects_non_hex_cells() {
    let mut array = RegionArray::new(4, 2, RegionLevelType::Surface).unwrap();
    assert_eq!(
        array.set_region(1, 0, RegionId::new(0)),
        Err(RegionArrayError::CoordinateHasNoRegion)
    );
}
