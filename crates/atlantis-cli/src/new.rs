use std::time::{SystemTime, UNIX_EPOCH};

use atlantis_engine::newgame::{create_new_game, NewGameOptions, SurfaceGenerator};
use atlantis_engine::players::write_players;
use atlantis_engine::ruleset::{Ruleset, WorldNameStyle};
use atlantis_engine::save::save_legacy;
use atlantis_engine::text::is_equal;

pub fn run(mut arguments: Vec<String>) -> i32 {
    if arguments
        .first()
        .is_some_and(|argument| is_equal(argument, "new"))
    {
        arguments.remove(0);
    }
    if arguments.is_empty() || arguments == ["--help"] {
        print_help();
        return 0;
    }

    let options = match parse_arguments(arguments) {
        Ok(options) => options,
        Err(error) => return fail(&error),
    };
    let seed = match options.seed {
        Some(seed) => seed,
        None => match default_seed() {
            Ok(seed) => seed,
            Err(error) => return fail(&error),
        },
    };
    let mut loaded = match create_new_game(
        options.ruleset,
        NewGameOptions {
            seed,
            width: options.width,
            height: options.height,
            nexus_size: options.nexus_size,
            generator: options.generator,
        },
    ) {
        Ok(loaded) => loaded,
        Err(error) => return fail(&error.to_string()),
    };
    let game_output = match save_legacy(&mut loaded) {
        Ok(output) => output,
        Err(error) => return fail(&format!("could not save game: {error}")),
    };
    let players_output = match write_players(&loaded.game) {
        Ok(output) => output,
        Err(error) => return fail(&format!("could not write players: {error}")),
    };

    if let Err(error) = std::fs::write("game.out", game_output) {
        return fail(&format!("could not write game.out: {error}"));
    }
    if let Err(error) = std::fs::write("players.out", players_output) {
        return fail(&format!(
            "could not write players.out: {error}; game.out was already written"
        ));
    }
    0
}

struct Options {
    ruleset: &'static dyn Ruleset,
    seed: Option<i32>,
    width: i32,
    height: i32,
    nexus_size: i32,
    generator: SurfaceGenerator,
}

fn parse_arguments(arguments: Vec<String>) -> Result<Options, String> {
    let mut ruleset_name = None;
    let mut seed = None;
    let mut width = None;
    let mut height = None;
    let mut nexus_size = None;
    let mut generator = None;
    let mut index = 0;

    while index < arguments.len() {
        let option = arguments[index].as_str();
        let value = arguments.get(index + 1).cloned();
        match option {
            "--ruleset" => {
                let value = require_value(option, value)?;
                if ruleset_name.replace(value).is_some() {
                    return Err("--ruleset may be specified only once".to_owned());
                }
                index += 1;
            }
            "--seed" => {
                let value = parse_value(option, value)?;
                if seed.replace(value).is_some() {
                    return Err("--seed may be specified only once".to_owned());
                }
                index += 1;
            }
            "--width" => {
                let value = parse_value(option, value)?;
                if width.replace(value).is_some() {
                    return Err("--width may be specified only once".to_owned());
                }
                index += 1;
            }
            "--height" => {
                let value = parse_value(option, value)?;
                if height.replace(value).is_some() {
                    return Err("--height may be specified only once".to_owned());
                }
                index += 1;
            }
            "--nexus" => {
                let value = parse_value(option, value)?;
                if nexus_size.replace(value).is_some() {
                    return Err("--nexus may be specified only once".to_owned());
                }
                index += 1;
            }
            "--generator" => {
                let value = parse_value(option, value)?;
                if generator.replace(value).is_some() {
                    return Err("--generator may be specified only once".to_owned());
                }
                index += 1;
            }
            _ => {
                return Err(format!(
                    "unknown option {option:?}. Try: atlantis new --help"
                ))
            }
        }
        index += 1;
    }

    let ruleset_name =
        ruleset_name.ok_or_else(|| "--ruleset is required. Try: atlantis new --help".to_owned())?;
    let ruleset = <dyn Ruleset>::from_name(&ruleset_name)
        .ok_or_else(|| format!("unsupported ruleset {ruleset_name:?}"))?;
    let generator = match generator.unwrap_or(1) {
        1 => SurfaceGenerator::Classic,
        2 => SurfaceGenerator::Natural,
        number => return Err(format!("--generator must be 1 or 2 (got {number})")),
    };
    if generator == SurfaceGenerator::Natural
        && ruleset.world_name_style() != WorldNameStyle::NewOrigins
    {
        return Err("generator 2 requires the neworigins ruleset".to_owned());
    }

    Ok(Options {
        ruleset,
        seed,
        width: width.unwrap_or(64),
        height: height.unwrap_or(64),
        nexus_size: nexus_size.unwrap_or(1),
        generator,
    })
}

fn require_value(option: &str, value: Option<String>) -> Result<String, String> {
    value.ok_or_else(|| format!("{option} needs a value. Try: atlantis new --help"))
}

fn parse_value(option: &str, value: Option<String>) -> Result<i32, String> {
    let value = require_value(option, value)?;
    value
        .parse()
        .map_err(|_| format!("{option} must be an integer (got {value:?})"))
}

fn default_seed() -> Result<i32, String> {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("could not read the system clock: {error}"))?
        .as_secs();
    Ok(seed_from_unix_seconds(seconds))
}

fn seed_from_unix_seconds(seconds: u64) -> i32 {
    seconds as i32
}

fn fail(message: &str) -> i32 {
    println!("error: {message}");
    2
}

fn print_help() {
    println!(
        "Usage: atlantis new --ruleset standard|neworigins [--seed N] [--width W --height H] \\\n\
         [--nexus N] [--generator 1|2]\n\
         Defaults: 64x64 map, a single-hex nexus, and the classic generator.\n\
         Generator 2 is the NewOrigins natural generator."
    );
}

#[cfg(test)]
mod tests {
    use super::seed_from_unix_seconds;

    #[test]
    fn default_seed_narrows_epoch_seconds_like_cpp_int() {
        assert_eq!(seed_from_unix_seconds(i32::MAX as u64 + 1), i32::MIN);
    }
}
