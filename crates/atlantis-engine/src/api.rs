//! The turn API: one call runs a turn over in-memory inputs and returns every file it writes.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::orders::OrderParseError;
use crate::players::PlayersError;
use crate::save::{LoadedGame, SaveError};

/// Everything a turn reads. The original probes the disk for `orders.N` and `times.N`; here
/// they are data, so the same core runs in the CLI, tests and the browser.
pub struct TurnInput {
    /// The save, already loaded: its RNG has been reseeded from the save as `OpenGame` does.
    pub loaded_game: LoadedGame,
    /// `players.in`.
    pub players_input: String,
    /// `orders.N`, keyed by faction number; a missing key is a missing file.
    pub order_files: BTreeMap<i32, Vec<u8>>,
    /// The `times.N` names that already exist, for the Times file-naming draws (not yet reached:
    /// world events are unported).
    pub existing_times: BTreeSet<String>,
}

/// One file a turn writes, named as the original names it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TurnFile {
    pub name: String,
    pub contents: String,
}

/// What a turn produces: `files` in the order the original writes them (reports, templates,
/// `players.out`, `game.out`), and the console output from "Setting Up Turn..." to "done".
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TurnOutput {
    pub files: Vec<TurnFile>,
    pub engine_output: String,
}

/// Why a turn did not complete. Only `GameFinished` carries the console output; the others lose
/// it. `Players` follows C++ `ReadPlayers` failing after "Setting Up Turn..." and "Reading the
/// Gamemaster File...". `Orders` has no C++ counterpart (`ReadOrders` cannot fail; the parser
/// here reports malformed input). `Save` follows `main`'s `SaveGame` failing after the whole
/// turn's console. A caller reproducing the console adds `main`'s "Couldn't run the game!" or
/// "Couldn't save the game!".
#[derive(Debug)]
pub enum TurnError {
    /// C++ `RunGame` returns 0 after "This game is finished!", so nothing is saved.
    GameFinished {
        engine_output: String,
    },
    Players(PlayersError),
    Orders(OrderParseError),
    Save(SaveError),
    Model(crate::model::ModelError),
}

impl fmt::Display for TurnError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GameFinished { .. } => formatter.write_str("this game is finished"),
            Self::Players(error) => write!(formatter, "could not read players input: {error}"),
            Self::Orders(error) => write!(formatter, "could not read orders input: {error}"),
            Self::Save(error) => write!(formatter, "could not write game save: {error}"),
            Self::Model(error) => write!(formatter, "could not update turn model: {error}"),
        }
    }
}

impl std::error::Error for TurnError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::GameFinished { .. } => None,
            Self::Players(error) => Some(error),
            Self::Orders(error) => Some(error),
            Self::Save(error) => Some(error),
            Self::Model(error) => Some(error),
        }
    }
}

impl From<PlayersError> for TurnError {
    fn from(error: PlayersError) -> Self {
        Self::Players(error)
    }
}

impl From<OrderParseError> for TurnError {
    fn from(error: OrderParseError) -> Self {
        Self::Orders(error)
    }
}

impl From<SaveError> for TurnError {
    fn from(error: SaveError) -> Self {
        Self::Save(error)
    }
}

impl From<crate::model::ModelError> for TurnError {
    fn from(error: crate::model::ModelError) -> Self {
        Self::Model(error)
    }
}

/// C++: `main.cpp` `run`: `RunGame` then `SaveGame`, after the caller's `OpenGame`.
pub fn run_turn(input: TurnInput) -> Result<TurnOutput, TurnError> {
    crate::turn::run_game(input)
}
