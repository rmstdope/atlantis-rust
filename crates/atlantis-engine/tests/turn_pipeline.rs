use std::collections::{BTreeMap, BTreeSet};

use atlantis_engine::api::{run_turn, TurnError, TurnInput};
use atlantis_engine::model::{Game, Region, RegionArray, RegionLevelType};
use atlantis_engine::newgame::add_faction;
use atlantis_engine::npc::create_npc_factions;
use atlantis_engine::players::write_players;
use atlantis_engine::rng::Rng;
use atlantis_engine::ruleset::Ruleset;
use atlantis_engine::save::{load_legacy, LoadedGame};
use atlantis_engine::turn::pre_process_turn;

const TURN_0_ENGINE_OUTPUT: &str =
    include_str!("../../../fixtures/standard/turn_0/engine-output.txt");
const TURN_1_GAME: &str = include_str!("../../../fixtures/standard/turn_1/game.in");
const TURN_1_PLAYERS: &str = include_str!("../../../fixtures/standard/turn_1/players.in");
const TURN_1_ORDERS: &[u8] = include_bytes!("../../../fixtures/standard/turn_1/orders.3");

fn standard() -> &'static dyn Ruleset {
    <dyn Ruleset>::from_name("standard").expect("standard ruleset")
}

fn neworigins() -> &'static dyn Ruleset {
    <dyn Ruleset>::from_name("neworigins").expect("neworigins ruleset")
}

/// A standard game with no world: the Guardsmen (1), the Creatures (2) and one leaderless player
/// faction (3). Every unported phase walks units, regions or objects, so this is the turn the
/// pipeline can complete today. The player faction has no unit, so `FindDeadFactions` marks it
/// dead: it still gets its report and template, and is gone from `game.out`.
fn empty_world_game() -> LoadedGame {
    empty_world_game_for(standard())
}

fn empty_world_game_for(ruleset: &'static dyn Ruleset) -> LoadedGame {
    let mut game = Game::default();
    game.game_status = 1; // GAME_STATUS_NEW, as `atlantis new` leaves it.
    game.setup_unit_nums().expect("unit numbers");
    create_npc_factions(&mut game, ruleset);
    let mut rng = Rng::new(42);
    add_faction(&mut game, ruleset, true, None, &mut rng).expect("player faction");
    LoadedGame {
        game,
        ruleset,
        seed: 42,
        rng,
    }
}

#[test]
fn run_turn_completes_a_turn_with_no_queued_work() {
    let loaded = empty_world_game();
    let players_input = write_players(&loaded.game).expect("players.in");

    let output = run_turn(TurnInput {
        loaded_game: loaded,
        players_input,
        order_files: BTreeMap::new(),
        existing_times: BTreeSet::new(),
    })
    .expect("run the turn");

    let names = output
        .files
        .iter()
        .map(|file| file.name.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        names,
        [
            "report.1",
            "report.3",
            "template.3",
            "players.out",
            "game.out"
        ]
    );

    let run_game_output = &TURN_0_ENGINE_OUTPUT[TURN_0_ENGINE_OUTPUT
        .find("Setting Up Turn...")
        .expect("fixture prints the turn setup")..];
    assert_eq!(output.engine_output, run_game_output);

    let file = |name: &str| {
        &output
            .files
            .iter()
            .find(|file| file.name == name)
            .expect("file was produced")
            .contents
    };
    let saved = load_legacy(file("game.out")).expect("game.out reloads");
    assert_eq!((saved.game.year, saved.game.month), (1, 0));
    let saved_factions = saved
        .game
        .faction_ids()
        .iter()
        .map(|id| saved.game.faction(*id).expect("faction").num)
        .collect::<Vec<_>>();
    assert_eq!(saved_factions, [1, 2], "the dead player faction is deleted");
    let players = file("players.out");
    assert!(players.starts_with("AtlantisPlayerStatus"));
    assert!(
        players.contains("Faction: 3"),
        "players.out is written before deletion"
    );
}

#[test]
#[should_panic(expected = "unported turn phase RunFindOrders has work: unit")]
fn run_turn_stops_at_the_first_unported_phase_with_work() {
    let mut order_files = BTreeMap::new();
    order_files.insert(3, TURN_1_ORDERS.to_vec());

    let _ = run_turn(TurnInput {
        loaded_game: load_legacy(TURN_1_GAME).expect("load standard turn 1"),
        players_input: TURN_1_PLAYERS.to_owned(),
        order_files,
        existing_times: BTreeSet::new(),
    });
}

#[test]
#[should_panic(expected = "unported turn phase RunQuitOrders has work: faction 3")]
fn run_turn_quits_inactive_factions_into_the_unported_quit_phase() {
    let mut loaded = empty_world_game();
    let max_inactive = loaded.ruleset.game_defs().max_inactive_turns;
    assert_ne!(max_inactive, -1, "standard removes inactive factions");
    let player = *loaded.game.faction_ids().last().expect("player faction");
    // Turn 1 runs next; a faction whose last orders are MAX_INACTIVE_TURNS old is quit by the GM.
    loaded.game.faction_mut(player).expect("player").last_orders = 1 - max_inactive;
    let players_input = write_players(&loaded.game).expect("players.in");

    let _ = run_turn(TurnInput {
        loaded_game: loaded,
        players_input,
        order_files: BTreeMap::new(),
        existing_times: BTreeSet::new(),
    });
}

#[test]
fn run_turn_refuses_a_finished_game_without_saving() {
    let loaded = empty_world_game();
    let players_input = write_players(&loaded.game)
        .expect("players.in")
        .replace("GameStatus: New", "GameStatus: Finished");

    let error = run_turn(TurnInput {
        loaded_game: loaded,
        players_input,
        order_files: BTreeMap::new(),
        existing_times: BTreeSet::new(),
    })
    .expect_err("a finished game does not run");

    match error {
        TurnError::GameFinished { engine_output } => assert_eq!(
            engine_output,
            "Setting Up Turn...\nReading the Gamemaster File...\nThis game is finished!\n"
        ),
        other => panic!("unexpected error {other}"),
    }
}

#[test]
#[should_panic(expected = "unported turn phase CheckVictory has work")]
fn run_turn_does_not_complete_a_neworigins_turn_with_unported_victory_and_times() {
    // neworigins' CheckVictory and WriteWorldEvents act even in an empty world.
    let loaded = empty_world_game_for(neworigins());
    let players_input = write_players(&loaded.game).expect("players.in");

    let _ = run_turn(TurnInput {
        loaded_game: loaded,
        players_input,
        order_files: BTreeMap::new(),
        existing_times: BTreeSet::new(),
    });
}

fn surface_game(month: i32, rows: i32) -> Game {
    let mut game = Game::default();
    game.month = month;
    let mut surface = RegionArray::new(8, rows, RegionLevelType::Surface).expect("surface");
    for y in 0..rows {
        let mut state = Region::default();
        // Hex grids hold a region only where x + y is even.
        state.xloc = y % 2;
        state.yloc = y;
        state.zloc = 1;
        let region = game.add_region(state);
        surface.set_region(y % 2, y, region).expect("place region");
    }
    game.add_region_array(RegionArray::new(1, 1, RegionLevelType::Nexus).expect("nexus"));
    game.add_region_array(surface);
    game
}

fn preprocess(game: &mut Game) {
    let ruleset = standard();
    let tables = ruleset.tables();
    pre_process_turn(game, ruleset, &tables, &mut Rng::new(1)).expect("preprocess");
}

#[test]
fn pre_process_turn_rolls_month_eleven_into_the_next_year() {
    let mut game = surface_game(11, 8);
    game.year = 3;

    preprocess(&mut game);

    assert_eq!((game.year, game.month), (4, 0));
}

#[test]
fn pre_process_turn_sets_weather_by_latitude() {
    // C++: ARegionList::GetWeather; (3 * (y + 1)) / 9 picks the northern, middle or southern third.
    let weather = |month: i32| {
        let mut game = surface_game(month - 1, 9);
        preprocess(&mut game);
        game.region_ids()
            .iter()
            .map(|id| game.region(*id).expect("region").weather)
            .collect::<Vec<_>>()
    };
    // January (month 0): winter in the north, monsoon in the middle, normal in the south.
    assert_eq!(weather(0), [1, 1, 2, 2, 2, 0, 0, 0, 0]);
    // May (month 4): normal in the north and middle, winter in the south.
    assert_eq!(weather(4), [0, 0, 0, 0, 0, 1, 1, 1, 1]);
}
