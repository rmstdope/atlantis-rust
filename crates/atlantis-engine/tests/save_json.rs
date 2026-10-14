use atlantis_engine::save::{load_json, load_legacy, load_save, save_json, save_legacy, SaveError};

const STANDARD_GAME: &str = include_str!("../../../fixtures/standard/turn_0/game.in");

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

#[test]
fn json_roundtrip_preserves_game_and_rng_state() {
    let mut control = load_legacy(STANDARD_GAME).expect("load control fixture");
    control.rng.get_random(23);
    let expected_sequence = (0..260)
        .map(|_| control.rng.get_random(10_000))
        .collect::<Vec<_>>();

    let mut original = load_legacy(STANDARD_GAME).expect("load standard fixture");
    original.rng.get_random(23);
    let expected_game = original.game.clone();
    let expected_seed = original.seed;
    let expected_ruleset = original.ruleset.name();

    let json = save_json(&original).expect("save JSON");
    let value: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    assert_eq!(value["schema_version"], 1);

    let mut restored = load_json(&json).expect("load JSON");
    assert_eq!(restored.game, expected_game);
    assert_eq!(restored.seed, expected_seed);
    assert_eq!(restored.ruleset.name(), expected_ruleset);
    let restored_sequence = (0..260)
        .map(|_| restored.rng.get_random(10_000))
        .collect::<Vec<_>>();
    let original_sequence = (0..260)
        .map(|_| original.rng.get_random(10_000))
        .collect::<Vec<_>>();
    assert_eq!(restored_sequence, expected_sequence);
    assert_eq!(original_sequence, expected_sequence);
}

#[test]
fn load_save_auto_detects_legacy_and_json() {
    let legacy = load_save(STANDARD_GAME).expect("auto-detect legacy save");
    let json = save_json(&legacy).expect("save JSON");
    let from_json = load_save(&format!(" \n\t{json}")).expect("auto-detect JSON save");
    let from_legacy = load_legacy(STANDARD_GAME).expect("load legacy save");

    assert_eq!(from_json.game, from_legacy.game);
    assert_eq!(from_json.seed, from_legacy.seed);
    assert_eq!(from_json.ruleset.name(), from_legacy.ruleset.name());
}

#[test]
fn json_load_rejects_unsupported_schema_version() {
    let loaded = load_legacy(STANDARD_GAME).expect("load standard fixture");
    let json = save_json(&loaded).expect("save JSON");
    let mut value: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    value["schema_version"] = 2.into();
    let unsupported = serde_json::to_string(&value).expect("serialize modified JSON");

    assert!(matches!(
        load_save(&unsupported),
        Err(SaveError::InvalidValue {
            field: "JSON schema version",
            value,
        }) if value == "2"
    ));
}

#[test]
fn json_roundtrip_matches_legacy_for_every_fixture() {
    for (index, (ruleset, input)) in GAME_FIXTURES.iter().enumerate() {
        let mut direct = load_legacy(input)
            .unwrap_or_else(|error| panic!("fixture {index} ({ruleset}) failed to load: {error}"));
        let expected = save_legacy(&mut direct)
            .unwrap_or_else(|error| panic!("fixture {index} ({ruleset}) failed to save: {error}"));

        let loaded = load_legacy(input).unwrap_or_else(|error| {
            panic!("fixture {index} ({ruleset}) failed to reload: {error}")
        });
        let json = save_json(&loaded).unwrap_or_else(|error| {
            panic!("fixture {index} ({ruleset}) failed JSON save: {error}")
        });
        let mut restored = load_save(&json).unwrap_or_else(|error| {
            panic!("fixture {index} ({ruleset}) failed JSON load: {error}")
        });
        let actual = save_legacy(&mut restored).unwrap_or_else(|error| {
            panic!("fixture {index} ({ruleset}) failed JSON-to-legacy save: {error}")
        });

        assert_eq!(
            actual, expected,
            "fixture {index} ({ruleset}) JSON round-trip differs from direct legacy round-trip"
        );
    }
}
