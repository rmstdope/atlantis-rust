use atlantis_engine::defs::{
    objects::{object_flags, O_BOG, O_MFORTRESS, O_ROADN},
    skills::skill_flags,
    terrains::R_PLAIN,
    ItemFlags, ItemId,
};
use atlantis_engine::ruleset::Ruleset;

#[test]
fn ruleset_table_snapshots_match_cpp_references() {
    for (name, expected) in [
        (
            "standard",
            include_str!("../../../fixtures/tables/standard.jsonl"),
        ),
        (
            "neworigins",
            include_str!("../../../fixtures/tables/neworigins.jsonl"),
        ),
    ] {
        let ruleset = <dyn Ruleset>::from_name(name).expect("known ruleset");
        let mut actual = String::from("{\"table\":\"metadata\",\"format_version\":1}\n");
        for global in ruleset.game_defs().globals_jsonl() {
            actual.push_str(&global);
            actual.push('\n');
        }
        actual.push_str(
            &ruleset
                .tables()
                .tables_jsonl()
                .expect("all table references resolve"),
        );
        let actual_lines = actual.lines().collect::<Vec<_>>();
        let expected_lines = expected.lines().collect::<Vec<_>>();
        assert_eq!(
            actual_lines.len(),
            expected_lines.len(),
            "{name} table row count"
        );
        for (line_number, (actual, expected)) in actual_lines.iter().zip(expected_lines).enumerate()
        {
            assert_eq!(
                actual,
                &expected,
                "{name} table differs from C++ reference at line {}",
                line_number + 1
            );
        }
    }
}

#[test]
fn standard_tables_apply_startup_mutations() {
    let standard = <dyn Ruleset>::from_name("standard").expect("standard ruleset");
    let tables = standard.tables();

    let pick = tables
        .item_families
        .find_item_index("PICK")
        .expect("pick item");
    assert_eq!(
        tables.item_families.items[pick].flags & ItemFlags::DISABLED,
        0
    );

    let axe = &tables.item_families.items[ItemId::Axe.index()];
    assert_eq!(axe.mult_item, ItemId::Hammer.index() as i32);
    assert_eq!(axe.mult_value, 1);

    assert_eq!(tables.objects[O_ROADN].flags & object_flags::DISABLED, 0);
    let armorcraft = tables
        .skills
        .skills
        .iter()
        .find(|skill| skill.name == Some("armorcraft"))
        .expect("armorcraft skill");
    assert_eq!(armorcraft.flags & skill_flags::DISABLED, 0);
}

#[test]
fn neworigins_tables_apply_startup_mutations_and_preserve_enable_item_bug() {
    let neworigins = <dyn Ruleset>::from_name("neworigins").expect("neworigins ruleset");
    let tables = neworigins.tables();

    let adamantium = &tables.item_families.items[ItemId::Admantium.index()];
    assert_eq!(adamantium.production_level, 5);
    assert_eq!(adamantium.production_skill.as_deref(), Some("MINI"));

    assert_eq!(tables.objects[O_ROADN].item, ItemId::Stone.index() as i32);
    assert_eq!(tables.objects[O_ROADN].cost, 30);
    assert_eq!(tables.objects[O_ROADN].skill, Some("BUIL"));
    assert_eq!(tables.objects[O_MFORTRESS].name, "Magical Fortress");

    assert_eq!(
        tables.terrains[R_PLAIN].races,
        [
            ItemId::HighElf.index() as i32,
            ItemId::CentaurMan.index() as i32,
            ItemId::Gnoll.index() as i32,
            ItemId::Man.index() as i32,
        ]
    );

    let object_id_as_item = &tables.item_families.items[O_BOG];
    assert_eq!(object_id_as_item.flags & ItemFlags::DISABLED, 0);
}

#[test]
fn ruleset_snapshots_are_independent() {
    let standard = <dyn Ruleset>::from_name("standard").expect("standard ruleset");
    let neworigins = <dyn Ruleset>::from_name("neworigins").expect("neworigins ruleset");

    let neworigins_first = neworigins.tables();
    let standard_after = standard.tables();
    let neworigins_again = neworigins.tables();

    assert_eq!(neworigins_first, neworigins_again);
    assert_ne!(
        standard_after.objects[O_MFORTRESS].name,
        neworigins_first.objects[O_MFORTRESS].name
    );
}
