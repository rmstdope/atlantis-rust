use atlantis_engine::defs::{ItemFlags, ItemId, ItemTypeFlags, Tables, ITEM_COUNT};

#[test]
fn item_table_has_source_enum_count_and_indices() {
    let tables = Tables::default();

    assert_eq!(ITEM_COUNT, 229);
    assert_eq!(ItemId::Devil.index() + 1, ITEM_COUNT);
    assert_eq!(tables.items.len(), ITEM_COUNT);
    let silver = &tables.items[ItemId::Silver as usize];
    assert_eq!(silver.name.as_deref(), Some("silver"));
    assert_eq!(silver.abbreviation.as_deref(), Some("SILV"));
}

#[test]
fn default_item_families_match_cpp_pre_modify_reference() {
    let tables = Tables::default();
    let expected = include_bytes!("../../../fixtures/tables/item-families-default.jsonl");

    assert_eq!(tables.item_families_jsonl().as_bytes(), expected);
}

#[test]
fn item_lookup_preserves_cpp_alias_rules() {
    let tables = Tables::default();
    let silver = ItemId::Silver.index();

    assert_eq!(tables.lookup_item("SILV"), Some(silver));
    assert_eq!(tables.find_item_index("sIlV"), Some(silver));
    assert_eq!(tables.parse_item("silver"), Some(silver));
    assert_eq!(tables.parse_item("wood_elf"), Some(ItemId::WoodElf.index()));

    let (illusion_index, illusion) = tables
        .items
        .iter()
        .enumerate()
        .find(|(_, item)| {
            item.item_type & ItemTypeFlags::ILLUSION != 0 && item.abbreviation.is_some()
        })
        .expect("reference data has an illusion item");
    let illusion_abbreviation = illusion.abbreviation.as_deref().unwrap();
    assert_eq!(
        tables.lookup_item(&format!("i{illusion_abbreviation}")),
        Some(illusion_index)
    );
    assert_eq!(
        tables.parse_all_items(&format!(
            "i{}",
            illusion.name.as_deref().expect("illusion has a name")
        )),
        Some(illusion_index)
    );
}

#[test]
fn enabled_item_parsing_rejects_disabled_items() {
    let tables = Tables::default();
    let roses = ItemId::Roses.index();

    assert_ne!(tables.items[roses].flags & ItemFlags::DISABLED, 0);
    assert_eq!(tables.lookup_item("ROSE"), Some(roses));
    assert_eq!(tables.find_item_index("rose"), Some(roses));
    assert_eq!(tables.parse_all_items("ROSE"), Some(roses));
    assert_eq!(tables.parse_item("ROSE"), None);
}

#[test]
fn giveable_and_transportable_parsing_enforce_flags() {
    let tables = Tables::default();
    let cannot_give = tables
        .items
        .iter()
        .find(|item| {
            item.flags & (ItemFlags::CANTGIVE | ItemFlags::DISABLED) == ItemFlags::CANTGIVE
        })
        .expect("reference data has an enabled non-giveable item");
    assert_eq!(
        tables.parse_giveable_item(cannot_give.abbreviation.as_deref().unwrap()),
        None
    );

    let cannot_transport = tables
        .items
        .iter()
        .find(|item| {
            item.flags & (ItemFlags::NOTRANSPORT | ItemFlags::DISABLED | ItemFlags::CANTGIVE)
                == ItemFlags::NOTRANSPORT
        })
        .expect("reference data has an enabled non-transportable item");
    assert_eq!(
        tables.parse_transportable_item(cannot_transport.abbreviation.as_deref().unwrap()),
        None
    );
    assert_eq!(
        tables.parse_transportable_item("SILV"),
        Some(ItemId::Silver.index())
    );
}

#[test]
fn item_family_finders_match_cpp_string_equality() {
    let tables = Tables::default();

    let item = tables
        .items
        .iter()
        .find_map(|item| item.abbreviation.as_deref())
        .unwrap();
    assert!(tables
        .find_item(item.to_ascii_lowercase().as_str())
        .is_some());

    let race = tables
        .races
        .iter()
        .find_map(|race| race.abbreviation.as_deref())
        .unwrap();
    assert!(tables.find_race(&race.to_ascii_lowercase()).is_some());

    let monster = tables
        .monsters
        .iter()
        .find_map(|monster| monster.abbreviation.as_deref())
        .unwrap();
    assert!(tables
        .find_monster(&monster.to_ascii_lowercase(), false)
        .is_some());

    let weapon = tables
        .weapons
        .iter()
        .find_map(|weapon| weapon.abbreviation.as_deref())
        .unwrap();
    assert!(tables.find_weapon(&weapon.to_ascii_lowercase()).is_some());

    let armor = tables
        .armor
        .iter()
        .find_map(|armor| armor.abbreviation.as_deref())
        .unwrap();
    assert!(tables.find_armor(&armor.to_ascii_lowercase()).is_some());

    let mount = tables
        .mounts
        .iter()
        .find_map(|mount| mount.abbreviation.as_deref())
        .unwrap();
    assert!(tables.find_mount(&mount.to_ascii_lowercase()).is_some());

    let battle_item = tables
        .battle_items
        .iter()
        .find_map(|item| item.abbreviation.as_deref())
        .unwrap();
    assert!(tables
        .find_battle_item(&battle_item.to_ascii_lowercase())
        .is_some());
}

#[test]
fn race_item_finder_accepts_combined_man_flags() {
    let tables = Tables::default();
    let race_index = tables
        .races
        .iter()
        .position(|race| race.abbreviation.as_deref() == Some("LEAD"))
        .expect("reference data has the LEAD race");

    assert_ne!(
        tables.items[0].item_type,
        ItemTypeFlags::MAN,
        "LEAD item combines MAN with another type flag"
    );
    assert_ne!(tables.items[0].item_type & ItemTypeFlags::MAN, 0);
    assert_eq!(tables.find_race_item_index(race_index), Some(0));
}
