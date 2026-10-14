use atlantis_engine::defs::{self, ItemId, ItemTypeFlags};
use atlantis_engine::model::{
    Faction, FactionItemShow, FactionSkillShow, Game, Object, Region, RegionArray, RegionLevelType,
    Unit,
};
use atlantis_engine::players::{read_players, write_players, PlayersError};
use atlantis_engine::ruleset::Ruleset;

macro_rules! player_fixture {
    ($ruleset:ident, $turn:literal) => {
        (
            stringify!($ruleset),
            $turn,
            include_str!(concat!(
                "../../../fixtures/",
                stringify!($ruleset),
                "/turn_",
                stringify!($turn),
                "/players.in"
            )),
            include_str!(concat!(
                "../../../fixtures/",
                stringify!($ruleset),
                "/turn_",
                stringify!($turn),
                "/players.out"
            )),
        )
    };
}

const PLAYER_FIXTURES: &[(&str, i32, &str, &str)] = &[
    player_fixture!(standard, 1),
    player_fixture!(standard, 2),
    player_fixture!(standard, 3),
    player_fixture!(standard, 4),
    player_fixture!(standard, 5),
    player_fixture!(standard, 6),
    player_fixture!(standard, 7),
    player_fixture!(standard, 8),
    player_fixture!(standard, 9),
    player_fixture!(standard, 10),
    player_fixture!(standard, 11),
    player_fixture!(standard, 12),
    player_fixture!(standard, 13),
    player_fixture!(neworigins, 1),
    player_fixture!(neworigins, 2),
    player_fixture!(neworigins, 3),
    player_fixture!(neworigins, 4),
    player_fixture!(neworigins, 5),
    player_fixture!(neworigins, 6),
    player_fixture!(neworigins, 7),
    player_fixture!(neworigins, 8),
    player_fixture!(neworigins, 9),
    player_fixture!(neworigins, 10),
    player_fixture!(neworigins, 11),
    player_fixture!(neworigins, 12),
    player_fixture!(neworigins, 13),
];

#[test]
fn item_id_index_conversion_is_checked() {
    assert_eq!(
        ItemId::from_index(ItemId::Silver.index()),
        Some(ItemId::Silver)
    );
    assert_eq!(ItemId::from_index(ItemId::COUNT), None);
}

#[test]
fn faction_discovery_lists_preserve_item_and_skill_values() {
    let mut faction = Faction::default();
    faction.items.set_num(ItemId::Wood, 2);
    faction.items.set_num(ItemId::Silver, 1);
    faction.skills.set(3, 30, 4);

    assert_eq!(faction.items.get_num(ItemId::Wood), 2);
    assert_eq!(faction.items.get_num(ItemId::Silver), 1);
    assert_eq!(
        faction
            .skills
            .entries()
            .iter()
            .map(|skill| (skill.skill_type, skill.days, skill.exp))
            .collect::<Vec<_>>(),
        [(3, 30, 4)]
    );
}

#[test]
fn players_files_write_the_original_header_and_faction_field_order() {
    let ruleset = <dyn Ruleset>::from_name("standard").expect("standard ruleset");
    let tables = ruleset.tables();
    let mut game = Game::default();
    game.year = 2;
    game.month = 0;
    let faction = game.add_faction(Faction::new(1, ruleset.faction_types()));

    read_players(
        &mut game,
        "AtlantisPlayerStatus\n\
         Version: 328196\n\
         TurnNumber: 999\n\
         GameStatus: Running\n\
         \n\
         Faction: 1\n\
         Name: Alpha Faction\n\
         Email: alpha@example.test\n\
         Password: secret\n\
         LastOrders: 9\n\
         FirstTurn: 2\n\
         SendTimes: 0\n\
         Template: short\n\
         Battle: anything\n\
         Custom: preserved\n",
        ruleset.game_defs(),
        &tables,
    )
    .expect("players.in parses");

    assert_eq!(game.faction(faction).unwrap().last_orders, 0);
    assert_eq!(
        write_players(&game).expect("players.out writes"),
        "AtlantisPlayerStatus\n\
         Version: 328197\n\
         TurnNumber: 13\n\
         GameStatus: Running\n\
         \n\
         Faction: 1\n\
         Name: Alpha Faction\n\
         Email: alpha@example.test\n\
         Password: secret\n\
         LastOrders: 0\n\
         FirstTurn: 2\n\
         SendTimes: 0\n\
         Template: short\n\
         Battle: na\n\
         Custom: preserved\n"
    );
}

#[test]
fn read_players_applies_reward_times_send_times_and_first_turn() {
    let ruleset = <dyn Ruleset>::from_name("standard").expect("standard ruleset");
    let tables = ruleset.tables();
    let mut globals = *ruleset.game_defs();
    globals.times_reward = 5000;
    let mut game = Game::default();
    let faction_id = game.add_faction(Faction::new(1, ruleset.faction_types()));

    read_players(
        &mut game,
        "AtlantisPlayerStatus\n\
         Version: 328197\n\
         TurnNumber: 1\n\
         GameStatus: Running\n\
         Faction: 1\n\
         RewardTimes\n\
         Reward: 250\n\
         SendTimes: 0\n\
         FirstTurn: 7\n",
        &globals,
        &tables,
    )
    .expect("players.in parses");

    let faction = game.faction(faction_id).unwrap();
    assert_eq!(faction.unclaimed, 5250);
    assert_eq!(
        faction.events,
        ["Times reward of 5000 silver.", "Reward of 250 silver."]
    );
    assert_eq!(faction.times, 0);
    assert_eq!(faction.start_turn, 7);
}

#[test]
fn read_players_applies_last_orders_only_when_scripts_maintain_them() {
    let ruleset = <dyn Ruleset>::from_name("standard").expect("standard ruleset");
    let tables = ruleset.tables();
    let mut globals = *ruleset.game_defs();
    globals.lastorders_maintained_by_scripts = 1;
    let mut game = Game::default();
    let faction_id = game.add_faction(Faction::new(1, ruleset.faction_types()));

    read_players(
        &mut game,
        "AtlantisPlayerStatus\n\
         Version: 328197\n\
         TurnNumber: 1\n\
         GameStatus: Running\n\
         Faction: 1\n\
         LastOrders: 19\n",
        &globals,
        &tables,
    )
    .expect("players.in parses");

    assert_eq!(game.faction(faction_id).unwrap().last_orders, 19);
}

#[test]
fn read_players_updates_location_and_new_unit() {
    let ruleset = <dyn Ruleset>::from_name("standard").expect("standard ruleset");
    let tables = ruleset.tables();
    let mut game = Game::default();
    let faction_id = game.add_faction(Faction::new(1, ruleset.faction_types()));
    let region_id = game.add_region(Region::default());
    let mut regions = RegionArray::new(4, 4, RegionLevelType::Surface).unwrap();
    regions.set_region(0, 0, region_id).unwrap();
    game.add_region_array(regions);
    let dummy = game.add_object(region_id, Object::default()).unwrap();

    read_players(
        &mut game,
        "AtlantisPlayerStatus\n\
         Version: 328197\n\
         TurnNumber: 1\n\
         GameStatus: Running\n\
         Faction: 1\n\
         Loc: 0 0 0\n\
         NewUnit: 42\n",
        ruleset.game_defs(),
        &tables,
    )
    .expect("players.in parses");

    assert_eq!(game.faction(faction_id).unwrap().region, Some(region_id));
    let unit_id = game.object(dummy).unwrap().unit_ids()[0];
    let unit = game.unit(unit_id).unwrap();
    assert_eq!(unit.num, 1);
    assert_eq!(unit.gm_alias, 42);
    assert_eq!(unit.faction, Some(faction_id));
    assert_eq!(
        game.faction(faction_id).unwrap().events,
        ["Unit (1): Is given to your faction."]
    );
}

#[test]
fn read_players_applies_item_and_skill_grants() {
    let ruleset = <dyn Ruleset>::from_name("standard").expect("standard ruleset");
    let tables = ruleset.tables();
    let mut game = Game::default();
    let faction_id = game.add_faction(Faction::new(1, ruleset.faction_types()));
    let region = game.add_region(Region::default());
    let object = game.add_object(region, Object::default()).unwrap();
    let mut unit = Unit::new(7, Some(faction_id), 0, 0);
    unit.items.set_num(ItemId::Leaders, 1);
    let unit_id = game.add_unit(object, unit).unwrap();
    let men_flag =
        tables.item_families.items[ItemId::Leaders.index()].item_type & ItemTypeFlags::MAN;
    assert_ne!(men_flag, 0);
    let mining = defs::skills::parse_skill(&tables.skills.skills, "mining").unwrap();

    read_players(
        &mut game,
        "AtlantisPlayerStatus\n\
         Version: 328197\n\
         TurnNumber: 1\n\
         GameStatus: Running\n\
         Faction: 1\n\
         Item: 7 2 iron\n\
         Skill: 7 mining 30\n",
        ruleset.game_defs(),
        &tables,
    )
    .expect("players.in parses");

    let unit = game.unit(unit_id).unwrap();
    assert_eq!(unit.items.get_num(ItemId::Iron), 2);
    assert_eq!(
        unit.skills
            .entries()
            .iter()
            .find(|skill| skill.skill_type == mining)
            .map(|skill| skill.days),
        Some(30)
    );
    let faction = game.faction(faction_id).unwrap();
    assert_eq!(faction.items.get_num(ItemId::Iron), 2);
    assert_eq!(
        faction.item_shows,
        [FactionItemShow {
            item_type: ItemId::Iron,
            full: true
        }]
    );
    assert_eq!(faction.skills.get_days(mining), 1);
    assert_eq!(
        faction.shows,
        [FactionSkillShow {
            skill_type: mining,
            level: 1
        }]
    );
    assert_eq!(faction.events.len(), 2);
    assert!(faction.events[0].starts_with("Unit (7): Is given "));
    assert!(faction.events[1].starts_with("Unit (7): Is taught 30 days of "));
}

#[test]
fn read_players_item_discovery_queues_granted_skill_shows() {
    let ruleset = <dyn Ruleset>::from_name("standard").expect("standard ruleset");
    let mut tables = ruleset.tables();
    let mining = defs::skills::parse_skill(&tables.skills.skills, "mining").unwrap();
    let iron = &mut tables.item_families.items[ItemId::Iron.index()];
    iron.grant_skill = Some(
        tables.skills.skills[mining]
            .abbreviation
            .unwrap()
            .to_owned(),
    );
    iron.max_grant = 2;

    let mut game = Game::default();
    let faction_id = game.add_faction(Faction::new(1, ruleset.faction_types()));
    let region = game.add_region(Region::default());
    let object = game.add_object(region, Object::default()).unwrap();
    let mut unit = Unit::new(7, Some(faction_id), 0, 0);
    unit.items.set_num(ItemId::Leaders, 1);
    game.add_unit(object, unit).unwrap();

    read_players(
        &mut game,
        "AtlantisPlayerStatus\n\
         Version: 328197\n\
         TurnNumber: 1\n\
         GameStatus: Running\n\
         Faction: 1\n\
         Item: 7 1 iron\n",
        ruleset.game_defs(),
        &tables,
    )
    .expect("players.in parses");

    let faction = game.faction(faction_id).unwrap();
    assert_eq!(faction.skills.get_days(mining), 2);
    assert_eq!(
        faction.shows,
        [
            FactionSkillShow {
                skill_type: mining,
                level: 1
            },
            FactionSkillShow {
                skill_type: mining,
                level: 2
            }
        ]
    );
    assert_eq!(
        faction.item_shows,
        [FactionItemShow {
            item_type: ItemId::Iron,
            full: true
        }]
    );
}

#[test]
fn read_players_rejects_deferred_faction_and_order_directives() {
    let ruleset = <dyn Ruleset>::from_name("standard").expect("standard ruleset");
    let tables = ruleset.tables();
    let mut game = Game::default();
    game.add_faction(Faction::new(1, ruleset.faction_types()));

    assert_eq!(
        read_players(
            &mut game,
            "AtlantisPlayerStatus\n\
             Version: 328197\n\
             TurnNumber: 1\n\
             GameStatus: Running\n\
             Faction: new\n",
            ruleset.game_defs(),
            &tables
        ),
        Err(PlayersError::UnsupportedDirective(
            "Faction: new".to_owned()
        ))
    );
    assert_eq!(
        read_players(
            &mut game,
            "AtlantisPlayerStatus\n\
             Version: 328197\n\
             TurnNumber: 1\n\
             GameStatus: Running\n\
             Faction: 1\n\
             Order: quit\n",
            ruleset.game_defs(),
            &tables
        ),
        Err(PlayersError::UnsupportedDirective("Order:".to_owned()))
    );
}

#[test]
fn rewards_credit_npc_factions_without_recording_events() {
    let ruleset = <dyn Ruleset>::from_name("standard").expect("standard ruleset");
    let tables = ruleset.tables();
    let mut globals = *ruleset.game_defs();
    globals.times_reward = 5000;
    let mut faction = Faction::new(1, ruleset.faction_types());
    faction
        .faction_types
        .iter_mut()
        .find(|(faction_type, _)| faction_type.eq_ignore_ascii_case("war"))
        .expect("war faction type")
        .1 = -1;
    let mut game = Game::default();
    let faction_id = game.add_faction(faction);

    read_players(
        &mut game,
        "AtlantisPlayerStatus\n\
         Version: 328197\n\
         TurnNumber: 1\n\
         GameStatus: Running\n\
         Faction: 1\n\
         RewardTimes\n\
         Reward: 250\n",
        &globals,
        &tables,
    )
    .expect("players.in parses");

    let faction = game.faction(faction_id).unwrap();
    assert_eq!(faction.unclaimed, 5250);
    assert!(faction.events.is_empty());
}

#[test]
fn supported_player_fixtures_match_players_out() {
    for (ruleset_name, turn, input, expected) in PLAYER_FIXTURES {
        let ruleset = <dyn Ruleset>::from_name(ruleset_name).expect("fixture ruleset");
        let tables = ruleset.tables();
        let mut game = Game::default();
        let output_turn = turn + 1;
        game.year = (output_turn - 1) / 12 + 1;
        game.month = (output_turn - 1) % 12;
        for number in 1..=3 {
            game.add_faction(Faction::new(number, ruleset.faction_types()));
        }

        read_players(&mut game, input, ruleset.game_defs(), &tables)
            .unwrap_or_else(|error| panic!("{ruleset_name} turn {turn}: {error}"));
        let actual = write_players(&game).expect("players.out writes");
        let actual = without_last_orders_if_turn_processing_owns_them(ruleset_name, &actual);
        let expected = without_last_orders_if_turn_processing_owns_them(ruleset_name, expected);
        assert_eq!(actual, expected, "{ruleset_name} turn {turn}");
    }
}

fn without_last_orders_if_turn_processing_owns_them(ruleset: &str, text: &str) -> String {
    if ruleset != "standard" {
        return text.to_owned();
    }
    text.split_inclusive('\n')
        .filter(|line| !line.starts_with("LastOrders:"))
        .fold(String::with_capacity(text.len()), |mut output, line| {
            output.push_str(line);
            output
        })
}

#[test]
fn fixture_last_orders_filter_preserves_other_bytes() {
    assert_eq!(
        without_last_orders_if_turn_processing_owns_them(
            "standard",
            "Name: Alpha\r\nLastOrders: 2\r\nEmail: user"
        ),
        "Name: Alpha\r\nEmail: user"
    );
}
