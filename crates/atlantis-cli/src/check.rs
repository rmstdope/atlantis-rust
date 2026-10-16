use std::fs;
use std::path::PathBuf;

use atlantis_engine::orders::check_orders;
use atlantis_engine::ruleset::Ruleset;
use atlantis_engine::save::load_save;
use atlantis_engine::text::is_equal;

const DEFAULT_GAME_FILE: &str = "game.in";

pub fn run(mut arguments: Vec<String>) -> i32 {
    if arguments
        .first()
        .is_some_and(|argument| is_equal(argument, "check"))
    {
        arguments.remove(0);
    }
    if arguments == ["--help"] {
        print_help();
        return 0;
    }

    let options = match parse_arguments(arguments) {
        Ok(options) => options,
        Err(error) => return fail(&error),
    };
    let input = match fs::read(&options.orders_file) {
        Ok(input) => input,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return fail(&format!(
                "there is no orders file {}",
                options.orders_file.display()
            ));
        }
        Err(error) => {
            return fail(&format!(
                "could not read orders file {}: {error}",
                options.orders_file.display()
            ));
        }
    };
    let ruleset = match choose_ruleset(options.ruleset.as_deref()) {
        Ok(ruleset) => ruleset,
        Err(error) => return fail(&error),
    };
    let result = match check_orders(ruleset, &input) {
        Ok(result) => result,
        Err(error) => return fail(&format!("could not check orders: {error}")),
    };

    if let Some(check_file) = options.check_file {
        if fs::write(&check_file, result.output).is_err() {
            return fail(&format!("could not write {}", check_file.display()));
        }
        let error_summary = match result.error_count {
            0 => "no errors".to_owned(),
            1 => "1 error".to_owned(),
            count => format!("{count} errors"),
        };
        println!(
            "Checked {} against {} rules: {error_summary}. Result written to {}",
            options.orders_file.display(),
            ruleset.name(),
            check_file.display()
        );
    } else {
        print!("{}", result.output);
    }

    i32::from(result.error_count > 0)
}

struct Options {
    orders_file: PathBuf,
    check_file: Option<PathBuf>,
    ruleset: Option<String>,
}

fn parse_arguments(arguments: Vec<String>) -> Result<Options, String> {
    let mut positional = Vec::new();
    let mut ruleset = None;
    let mut index = 0;

    while index < arguments.len() {
        match arguments[index].as_str() {
            "--ruleset" => {
                let Some(value) = arguments.get(index + 1) else {
                    return Err("--ruleset needs a value. Try: atlantis check --help".to_owned());
                };
                if ruleset.replace(value.clone()).is_some() {
                    return Err(
                        "--ruleset may be specified only once. Try: atlantis check --help"
                            .to_owned(),
                    );
                }
                index += 1;
            }
            option if option.starts_with("--") => {
                return Err(format!(
                    "unknown option {option:?}. Try: atlantis check --help"
                ));
            }
            argument => positional.push(PathBuf::from(argument)),
        }
        index += 1;
    }

    if positional.is_empty() {
        return Err("an orders file is required. Try: atlantis check --help".to_owned());
    }
    if positional.len() > 2 {
        return Err("at most two file paths may be given. Try: atlantis check --help".to_owned());
    }

    let orders_file = positional.remove(0);
    let check_file = positional.pop();
    Ok(Options {
        orders_file,
        check_file,
        ruleset,
    })
}

fn choose_ruleset(name: Option<&str>) -> Result<&'static dyn Ruleset, String> {
    if let Some(name) = name {
        return <dyn Ruleset>::from_name(name).ok_or_else(|| {
            format!("unknown ruleset {name}. Use --ruleset standard or --ruleset neworigins")
        });
    }

    let game_contents = match fs::read_to_string(DEFAULT_GAME_FILE) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(
                "no game.in here to tell which rules to check against. Add --ruleset standard or --ruleset neworigins"
                    .to_owned(),
            );
        }
        Err(error) => return Err(format!("could not load game.in: {error}")),
    };
    load_save(&game_contents)
        .map(|game| game.ruleset)
        .map_err(|error| format!("could not load game.in: {error}"))
}

fn fail(message: &str) -> i32 {
    println!("error: {message}");
    2
}

fn print_help() {
    println!(
        "Usage: atlantis check ORDERFILE [CHECKFILE] [--ruleset standard|neworigins]\n  \
Checks an orders file without running a turn. The result (each order echoed back,\n  \
errors marked *** Error: ... ***) goes to CHECKFILE, or to the screen if it is left out.\n  \
The rules come from game.in in this folder unless --ruleset is given."
    );
}
