use atlantis_engine::defs::{
    enums::{flags, BattleLogLevel, DestroyBehavior, FactionActivityRules},
    strings::{DIRECTION_ABBREVIATIONS, DIRECTION_NAMES, MONTH_NAMES, SEASON_NAMES},
    GameDefs,
};
use atlantis_engine::ruleset::Ruleset;

const STANDARD_REFERENCE: &str = include_str!("../../../fixtures/tables/standard.jsonl");
const NEWORIGINS_REFERENCE: &str = include_str!("../../../fixtures/tables/neworigins.jsonl");

fn expected_globals(reference: &str) -> Vec<String> {
    reference
        .lines()
        .filter(|line| line.starts_with("{\"table\":\"globals\","))
        .map(str::to_owned)
        .collect()
}

fn assert_globals_match_reference(defs: &GameDefs, reference: &str) {
    assert_eq!(
        defs.globals_jsonl(),
        expected_globals(reference),
        "GameDefs globals differ from the committed C++ reference"
    );
}

#[test]
fn standard_globals_match_cpp_reference() {
    let standard = <dyn Ruleset>::from_name("Wyreth").expect("standard ruleset");
    assert_globals_match_reference(standard.game_defs(), STANDARD_REFERENCE);
}

#[test]
fn neworigins_globals_match_cpp_reference() {
    let neworigins = <dyn Ruleset>::from_name("NewOrigins").expect("neworigins ruleset");
    assert_globals_match_reference(neworigins.game_defs(), NEWORIGINS_REFERENCE);
}

#[test]
fn ruleset_name_and_static_data_match_cpp() {
    let standard = <dyn Ruleset>::from_name("standard").expect("standard ruleset id");
    let neworigins = <dyn Ruleset>::from_name("neworigins").expect("neworigins ruleset id");

    assert_eq!(standard.name(), "Wyreth");
    assert_eq!(neworigins.name(), "NewOrigins");
    assert!(<dyn Ruleset>::from_name("unknown").is_none());

    let standard_limits = standard.faction_limits();
    assert_eq!(standard_limits.mages, &[0, 1, 2, 3, 5, 7]);
    assert_eq!(standard_limits.apprentices, &[0, 2, 4, 6, 10, 14]);
    assert_eq!(standard_limits.taxes, &[0, 10, 24, 40, 60, 100]);
    assert_eq!(standard_limits.trades, &[0, 10, 24, 40, 60, 100]);
    assert_eq!(standard_limits.quartermasters, &[0, 2, 4, 8, 12, 20]);
    assert_eq!(standard_limits.tacticians, &[0, 1, 2, 4, 6, 10]);
    assert_eq!(standard_limits.martial, &[0, 10, 25, 40, 60, 90]);

    let neworigins_limits = neworigins.faction_limits();
    assert_eq!(neworigins_limits.mages, &[1, 2, 3, 4, 5, 6]);
    assert_eq!(neworigins_limits.apprentices, &[1, 3, 5, 7, 10, 15]);
    assert_eq!(neworigins_limits.taxes, &[0, 15, 30, 50, 75, 100]);
    assert_eq!(neworigins_limits.trades, &[0, 15, 30, 50, 75, 100]);
    assert_eq!(neworigins_limits.quartermasters, &[0, 2, 5, 9, 14, 25]);
    assert_eq!(neworigins_limits.tacticians, &[0, 1, 2, 4, 6, 10]);
    assert_eq!(neworigins_limits.martial, &[0, 10, 25, 40, 60, 90]);

    assert_eq!(standard.faction_types(), &["War", "Trade", "Magic"]);
    assert_eq!(neworigins.faction_types(), &["Martial", "Magic"]);
    assert_eq!(standard.game_defs().who_can_tax, flags::TAX_NORMAL);
    assert_eq!(
        neworigins.game_defs().who_can_tax,
        flags::TAX_NORMAL | flags::TAX_HORSE_AND_RIDING_SKILL | flags::TAX_MAGE_DAMAGE
    );
    assert_eq!(
        neworigins.game_defs().transit_report,
        flags::REPORT_SHOW_REGION
            | flags::REPORT_SHOW_ROADS
            | flags::REPORT_SHOW_BUILDINGS
            | flags::REPORT_SHOW_GUARDS
    );
    assert_eq!(
        neworigins.game_defs().transport,
        flags::ALLOW_TRANSPORT | flags::QM_AFFECT_COST | flags::QM_AFFECT_DIST
    );
    assert_eq!(
        standard.game_defs().faction_activity,
        FactionActivityRules::Default
    );
    assert_eq!(
        neworigins.game_defs().faction_activity,
        FactionActivityRules::MartialMerged
    );
    assert_eq!(
        standard.game_defs().battle_log_level,
        BattleLogLevel::Normal
    );
    assert_eq!(
        neworigins.game_defs().battle_log_level,
        BattleLogLevel::Verbose
    );
    assert_eq!(
        standard.game_defs().destroy_behavior,
        DestroyBehavior::Immediate
    );
    assert_eq!(
        neworigins.game_defs().destroy_behavior,
        DestroyBehavior::PerSkill
    );

    assert_eq!(BattleLogLevel::Normal as i32, 0);
    assert_eq!(BattleLogLevel::Detailed as i32, 1);
    assert_eq!(BattleLogLevel::Verbose as i32, 2);
    assert_eq!(FactionActivityRules::Default as i32, 0);
    assert_eq!(FactionActivityRules::Martial as i32, 1);
    assert_eq!(FactionActivityRules::MartialMerged as i32, 2);
    assert_eq!(DestroyBehavior::Immediate as i32, 0);
    assert_eq!(DestroyBehavior::PerFigure as i32, 1);
    assert_eq!(DestroyBehavior::PerSkill as i32, 2);

    assert_eq!(
        DIRECTION_NAMES,
        [
            "North",
            "Northeast",
            "Southeast",
            "South",
            "Southwest",
            "Northwest"
        ]
    );
    assert_eq!(DIRECTION_ABBREVIATIONS, ["N", "NE", "SE", "S", "SW", "NW"]);
    assert_eq!(
        MONTH_NAMES,
        [
            "January",
            "February",
            "March",
            "April",
            "May",
            "June",
            "July",
            "August",
            "September",
            "October",
            "November",
            "December"
        ]
    );
    assert_eq!(
        SEASON_NAMES,
        ["clear", "winter", "monsoon season", "blizzard"]
    );
}
