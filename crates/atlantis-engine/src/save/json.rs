use serde::{Deserialize, Serialize};

use crate::model::Game;
use crate::rng::Rng;
use crate::ruleset::Ruleset;

use super::legacy::{load_legacy, LoadedGame, SaveError};

const JSON_SCHEMA_VERSION: u32 = 1;

#[derive(Serialize)]
struct JsonSaveRef<'a> {
    schema_version: u32,
    ruleset: &'a str,
    seed: i32,
    game: &'a Game,
    rng: &'a Rng,
}

#[derive(Deserialize)]
struct JsonSave {
    schema_version: u32,
    ruleset: String,
    seed: i32,
    game: Game,
    rng: Rng,
}

/// Writes a versioned JSON save without advancing the random generator.
pub fn save_json(loaded: &LoadedGame) -> Result<String, SaveError> {
    let save = JsonSaveRef {
        schema_version: JSON_SCHEMA_VERSION,
        ruleset: loaded.ruleset.name(),
        seed: loaded.seed,
        game: &loaded.game,
        rng: &loaded.rng,
    };
    let mut output = serde_json::to_string_pretty(&save).map_err(|error| SaveError::Input {
        field: "JSON save",
        message: error.to_string(),
    })?;
    output.push('\n');
    Ok(output)
}

/// Loads a JSON save after validating its schema version and ruleset.
pub fn load_json(input: &str) -> Result<LoadedGame, SaveError> {
    let save: JsonSave = serde_json::from_str(input).map_err(|error| SaveError::Input {
        field: "JSON save",
        message: error.to_string(),
    })?;
    if save.schema_version != JSON_SCHEMA_VERSION {
        return Err(SaveError::InvalidValue {
            field: "JSON schema version",
            value: save.schema_version.to_string(),
        });
    }
    let ruleset =
        <dyn Ruleset>::from_name(&save.ruleset).ok_or_else(|| SaveError::UnsupportedRuleset {
            name: save.ruleset.clone(),
        })?;

    Ok(LoadedGame {
        game: save.game,
        ruleset,
        seed: save.seed,
        rng: save.rng,
    })
}

/// Loads JSON objects and otherwise delegates to the legacy positional reader.
pub fn load_save(input: &str) -> Result<LoadedGame, SaveError> {
    if input.trim_start().starts_with('{') {
        load_json(input)
    } else {
        load_legacy(input)
    }
}
