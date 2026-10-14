use std::fs;
use std::path::{Path, PathBuf};

use atlantis_engine::defs::{
    objects::{lookup_object, O_DUMMY},
    skills::parse_skill,
    ItemFlags, ItemId, ItemTypeFlags, TerrainId,
};
use atlantis_engine::edit::{
    add_or_regenerate_town, add_region_gate, add_region_object, create_unit, find_region,
    find_unit, move_unit_to_region, regenerate_markets, regenerate_region,
    regenerate_region_products, remove_market, remove_region_gate, remove_region_object,
    remove_town, rename_region, rename_region_object, rename_town, rename_unit,
    set_market_amount_range, set_market_population_range, set_market_prices, set_region_race,
    set_region_terrain, set_region_wages, set_unit_description, set_unit_faction, set_unit_guard,
    set_unit_item, set_unit_skill, set_unit_type, swap_market_type,
};
use atlantis_engine::model::{GuardMode, Market, MarketType, RegionId, UnitType};
use atlantis_engine::save::{load_save, save_json, save_legacy, LoadedGame};
use atlantis_engine::text::is_equal;

const DEFAULT_GAME_FILE: &str = "game.in";

pub fn run(mut arguments: Vec<String>) -> i32 {
    if arguments
        .first()
        .is_some_and(|argument| is_equal(argument, "edit"))
    {
        arguments.remove(0);
    } else if arguments
        .first()
        .is_some_and(|argument| argument == "--help")
    {
        print_general_help();
        return 0;
    } else {
        println!("error: expected the `edit` command. Try: atlantis --help");
        return 2;
    }

    if arguments.is_empty() || arguments == ["--help"] {
        print_edit_help();
        return 0;
    }
    if arguments.len() == 2 && arguments[1] == "--help" {
        match arguments[0].to_ascii_lowercase().as_str() {
            "region" => print_region_help(),
            "unit" => print_unit_help(),
            _ => print_edit_help(),
        }
        return 0;
    }

    match parse_arguments(arguments) {
        Ok(options) => run_edit(options),
        Err(error) => fail(&error),
    }
}

struct Options {
    command: Vec<String>,
    script: Option<PathBuf>,
    output: Option<PathBuf>,
    dry_run: bool,
}

fn parse_arguments(arguments: Vec<String>) -> Result<Options, String> {
    let mut command = Vec::new();
    let mut script = None;
    let mut output = None;
    let mut dry_run = false;
    let mut index = 0;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--dry-run" => dry_run = true,
            "--out" => {
                let Some(path) = arguments.get(index + 1) else {
                    return Err("--out needs a file. Try: atlantis edit --help".to_owned());
                };
                output = Some(PathBuf::from(path));
                index += 1;
            }
            "--script" => {
                let Some(path) = arguments.get(index + 1) else {
                    return Err("--script needs a file. Try: atlantis edit --help".to_owned());
                };
                if script.replace(PathBuf::from(path)).is_some() {
                    return Err("--script may be specified only once".to_owned());
                }
                index += 1;
            }
            token => command.push(token.to_owned()),
        }
        index += 1;
    }
    if script.is_some() && !command.is_empty() {
        return Err("--script cannot be combined with a single edit".to_owned());
    }
    if script.is_none() && command.is_empty() {
        return Err("an edit command is required. Try: atlantis edit --help".to_owned());
    }
    Ok(Options {
        command,
        script,
        output,
        dry_run,
    })
}

struct ScriptLine {
    number: usize,
    command: Vec<String>,
}

struct ScriptFailure {
    line: usize,
    message: String,
    summaries: Vec<String>,
    edit_count: usize,
}

fn run_edit(options: Options) -> i32 {
    let script = match options.script.as_deref() {
        Some(path) => {
            let contents = match fs::read_to_string(path) {
                Ok(contents) => contents,
                Err(error) => return fail(&format!("could not read {}: {error}", path.display())),
            };
            let edits = match parse_script(&contents) {
                Ok(edits) => edits,
                Err((line, error)) => {
                    return fail(&format!("{error} (line {line})"));
                }
            };
            if edits.is_empty() {
                return fail(&format!("{} has no edits", path.display()));
            }
            Some(edits)
        }
        None => None,
    };
    let is_script = script.is_some();

    let input_path = PathBuf::from(DEFAULT_GAME_FILE);
    let contents = match fs::read_to_string(&input_path) {
        Ok(contents) => contents,
        Err(error) => {
            return fail(&format!("could not read {}: {error}", input_path.display()));
        }
    };
    let json_format = contents.trim_start().starts_with('{');
    let loaded = match load_save(&contents) {
        Ok(loaded) => loaded,
        Err(error) => return fail(&format!("could not load {}: {error}", input_path.display())),
    };

    let (mut loaded, summaries) = if let Some(script) = script {
        match apply_script(loaded, &script) {
            Ok(result) => result,
            Err(failure) => {
                for summary in failure.summaries {
                    println!("{summary}");
                }
                println!("error: {} (line {})", failure.message, failure.line);
                println!("Nothing saved: 1 of {} edits failed.", failure.edit_count);
                return 2;
            }
        }
    } else {
        let mut staged = loaded.clone();
        match apply_one(&mut staged, &options.command) {
            Ok(summary) => (staged, vec![summary]),
            Err(error) => return fail(&error),
        }
    };

    if options.dry_run {
        for summary in summaries {
            println!("{summary}");
        }
        println!("Dry run: nothing saved.");
        return 0;
    }

    let mut messages = match save_game(
        &mut loaded,
        json_format,
        &input_path,
        options.output.as_deref(),
    ) {
        Ok(messages) => messages,
        Err(error) => return fail(&error),
    };
    if is_script {
        let count = summaries.len();
        let result = if count == 1 {
            "1 edit applied".to_owned()
        } else {
            format!("{count} edits applied")
        };
        messages[0] = format!("{result}. {}", messages[0]);
    }
    for summary in summaries {
        println!("{summary}");
    }
    for message in messages {
        println!("{message}");
    }
    0
}

fn parse_script(contents: &str) -> Result<Vec<ScriptLine>, (usize, String)> {
    let mut edits = Vec::new();
    for (index, line) in contents.lines().enumerate() {
        let number = index + 1;
        let tokens = tokenize_script_line(line).map_err(|error| (number, error))?;
        if !tokens.is_empty() {
            edits.push(ScriptLine {
                number,
                command: tokens,
            });
        }
    }
    Ok(edits)
}

fn tokenize_script_line(line: &str) -> Result<Vec<String>, String> {
    let mut tokens = Vec::new();
    let mut token = String::new();
    let mut in_quotes = false;
    let mut escaped = false;
    let mut started = false;

    for character in line.chars() {
        if escaped {
            token.push(character);
            escaped = false;
            continue;
        }
        match character {
            '\\' if in_quotes => escaped = true,
            '"' => {
                in_quotes = !in_quotes;
                started = true;
            }
            '#' if !in_quotes => break,
            character if character.is_whitespace() && !in_quotes => {
                if started {
                    tokens.push(std::mem::take(&mut token));
                    started = false;
                }
            }
            character => {
                token.push(character);
                started = true;
            }
        }
    }
    if escaped {
        token.push('\\');
    }
    if in_quotes {
        return Err("unterminated quoted value".to_owned());
    }
    if started {
        tokens.push(token);
    }
    Ok(tokens)
}

fn apply_script(
    loaded: LoadedGame,
    edits: &[ScriptLine],
) -> Result<(LoadedGame, Vec<String>), ScriptFailure> {
    let mut staged = loaded;
    let mut summaries = Vec::with_capacity(edits.len());
    for edit in edits {
        let mut candidate = staged.clone();
        match apply_one(&mut candidate, &edit.command) {
            Ok(summary) => {
                staged = candidate;
                summaries.push(summary);
            }
            Err(message) => {
                return Err(ScriptFailure {
                    line: edit.number,
                    message,
                    summaries,
                    edit_count: edits.len(),
                });
            }
        }
    }
    Ok((staged, summaries))
}

fn apply_one(loaded: &mut LoadedGame, command: &[String]) -> Result<String, String> {
    match command.first().map(|target| target.to_ascii_lowercase()) {
        Some(target) if target == "region" => apply_region(loaded, command),
        Some(target) if target == "unit" => apply_unit(loaded, command),
        Some(target) => Err(format!(
            "unknown edit target {target:?}. Try: atlantis edit --help"
        )),
        None => Err("an edit command is required. Try: atlantis edit --help".to_owned()),
    }
}

fn apply_region(loaded: &mut LoadedGame, command: &[String]) -> Result<String, String> {
    if command.len() < 5 {
        return Err("region needs X Y Z and an edit. Try: atlantis edit region --help".to_owned());
    }
    let x = parse_number(&command[1], "X coordinate")?;
    let y = parse_number(&command[2], "Y coordinate")?;
    let z = parse_number(&command[3], "Z coordinate")?;
    let region_id = find_region(&loaded.game, x, y, z)
        .ok_or_else(|| format!("there is no region at ({x},{y},{z})"))?;
    let context = region_context(loaded, region_id);

    match command[4].to_ascii_lowercase().as_str() {
        "set-terrain" => {
            require_count(
                command,
                6,
                "set-terrain needs a terrain name. Try: atlantis edit region --help",
            )?;
            let tables = loaded.ruleset.tables();
            let terrain = tables
                .terrains
                .iter()
                .position(|definition| is_equal(definition.name, &command[5]))
                .ok_or_else(|| {
                    format!(
                        "unknown terrain {:?}. Valid: plain forest mountain swamp jungle desert tundra ...",
                        command[5]
                    )
                })?;
            let region = loaded
                .game
                .region(region_id)
                .ok_or_else(|| format!("there is no region at ({x},{y},{z})"))?;
            let old = region
                .terrain_type
                .and_then(|index| tables.terrains.get(index))
                .map(|definition| definition.name)
                .unwrap_or("none");
            let name = region.name.clone();
            let old = old.to_owned();
            set_region_terrain(loaded, region_id, terrain as TerrainId)
                .map_err(|error| error.to_string())?;
            let new = tables.terrains[terrain].name;
            Ok(format!(
                "Region ({x},{y}) \"{name}\": terrain {old} -> {new}"
            ))
        }
        "set-race" => {
            require_count(
                command,
                6,
                "set-race needs a race name. Try: atlantis edit region --help",
            )?;
            let race = if is_equal(&command[5], "none") {
                None
            } else {
                Some(
                    find_race(loaded, &command[5])
                        .ok_or_else(|| format!("unknown race {:?}", command[5]))?,
                )
            };
            let old = region_race_name(loaded, region_id);
            set_region_race(loaded, region_id, race).map_err(|error| error.to_string())?;
            Ok(format!(
                "{context}: race {old} -> {}",
                race.map(|id| item_name(loaded, id))
                    .unwrap_or_else(|| "none".to_owned())
            ))
        }
        "set-wages" => {
            require_count(
                command,
                6,
                "set-wages needs a maximum. Try: atlantis edit region --help",
            )?;
            let new = parse_number(&command[5], "wage maximum")?;
            let old = loaded
                .game
                .region(region_id)
                .expect("region was resolved")
                .max_wages;
            set_region_wages(loaded, region_id, new).map_err(|error| error.to_string())?;
            let actual = loaded
                .game
                .region(region_id)
                .expect("region remains after wage edit")
                .max_wages;
            Ok(format!("{context}: wages {old} -> {actual}"))
        }
        "set-name" => {
            let name = argument_text(command, 5, "set-name needs a region name")?;
            let old = loaded
                .game
                .region(region_id)
                .expect("region was resolved")
                .name
                .clone();
            rename_region(loaded, region_id, &name).map_err(|error| error.to_string())?;
            Ok(format!("{context}: name \"{old}\" -> \"{name}\""))
        }
        "regenerate-products" => {
            require_count(command, 5, "regenerate-products takes no arguments")?;
            let old = products_description(loaded, region_id);
            regenerate_region_products(loaded, region_id).map_err(|error| error.to_string())?;
            let new = products_description(loaded, region_id);
            Ok(format!("{context}: products {old} -> {new}"))
        }
        "regenerate-all" => {
            require_count(command, 5, "regenerate-all takes no arguments")?;
            let old = region_state_description(loaded, region_id);
            regenerate_region(loaded, region_id).map_err(|error| error.to_string())?;
            let new = region_state_description(loaded, region_id);
            Ok(format!("{context}: region state {old} -> {new}"))
        }
        "add-gate" => {
            require_count(command, 5, "add-gate takes no arguments")?;
            let old = loaded
                .game
                .region(region_id)
                .expect("region was resolved")
                .gate;
            add_region_gate(loaded, region_id).map_err(|error| error.to_string())?;
            let new = loaded
                .game
                .region(region_id)
                .expect("region remains after gate edit")
                .gate;
            Ok(format!(
                "{context}: gate {} -> {new}",
                if old == 0 {
                    "none".to_owned()
                } else {
                    old.to_string()
                }
            ))
        }
        "remove-gate" => {
            require_count(command, 5, "remove-gate takes no arguments")?;
            let old = loaded
                .game
                .region(region_id)
                .expect("region was resolved")
                .gate;
            remove_region_gate(loaded, region_id).map_err(|error| error.to_string())?;
            Ok(format!("{context}: gate {old} -> none"))
        }
        "add-town" | "regenerate-town" => {
            require_count(command, 5, "town regeneration takes no arguments")?;
            let old = town_description(loaded, region_id);
            add_or_regenerate_town(loaded, region_id).map_err(|error| error.to_string())?;
            let new = town_description(loaded, region_id);
            Ok(format!("{context}: town {old} -> {new}"))
        }
        "rename-town" => {
            let name = argument_text(command, 5, "rename-town needs a town name")?;
            let old = town_description(loaded, region_id);
            rename_town(loaded, region_id, &name).map_err(|error| error.to_string())?;
            Ok(format!(
                "{context}: town {old} -> {}",
                town_description(loaded, region_id)
            ))
        }
        "remove-town" => {
            require_count(command, 5, "remove-town takes no arguments")?;
            let old = town_description(loaded, region_id);
            remove_town(loaded, region_id).map_err(|error| error.to_string())?;
            Ok(format!("{context}: town {old} -> none"))
        }
        "add-object" => {
            require_count(
                command,
                6,
                "add-object needs an object type. Try: atlantis edit region --help",
            )?;
            let tables = loaded.ruleset.tables();
            let object_type = lookup_object(&command[5], &tables.objects)
                .ok_or_else(|| format!("unknown object type {:?}", command[5]))?;
            let object_id = add_region_object(loaded, region_id, object_type)
                .map_err(|error| error.to_string())?;
            let object = loaded
                .game
                .object(object_id)
                .expect("new object exists after adding");
            Ok(format!(
                "{context}: object none -> {}",
                object_description(object, &tables.objects)
            ))
        }
        "remove-object" => {
            require_count(
                command,
                6,
                "remove-object needs an index. Try: atlantis edit region --help",
            )?;
            let index = parse_object_index(&command[5])?;
            let old = object_at(loaded, region_id, index)?;
            let api_index = object_api_index(loaded, region_id, index)?;
            remove_region_object(loaded, region_id, api_index)
                .map_err(|error| error.to_string())?;
            Ok(format!("{context}: object {index} {old} -> removed"))
        }
        "rename-object" => {
            if command.len() < 7 {
                return Err(
                    "rename-object needs an index and name. Try: atlantis edit region --help"
                        .to_owned(),
                );
            }
            let index = parse_object_index(&command[5])?;
            let name = argument_text(command, 6, "rename-object needs a name")?;
            let old = object_at(loaded, region_id, index)?;
            let api_index = object_api_index(loaded, region_id, index)?;
            rename_region_object(loaded, region_id, api_index, &name)
                .map_err(|error| error.to_string())?;
            let new = object_at(loaded, region_id, index)?;
            Ok(format!("{context}: object {index} {old} -> {new}"))
        }
        "market" => apply_market(loaded, region_id, context, command),
        other => Err(format!(
            "unknown region edit {other:?}. Try: atlantis edit region --help"
        )),
    }
}

fn apply_market(
    loaded: &mut LoadedGame,
    region_id: RegionId,
    context: String,
    command: &[String],
) -> Result<String, String> {
    let Some(action) = command.get(5).map(|value| value.to_ascii_lowercase()) else {
        return Err("market needs an operation. Try: atlantis edit region --help".to_owned());
    };
    if action == "regenerate" {
        require_count(command, 6, "market regenerate takes no arguments")?;
        let old = markets_description(loaded, region_id);
        regenerate_markets(loaded, region_id).map_err(|error| error.to_string())?;
        return Ok(format!(
            "{context}: markets {old} -> {}",
            markets_description(loaded, region_id)
        ));
    }
    let expected = if matches!(action.as_str(), "population" | "amount" | "prices") {
        9
    } else if matches!(action.as_str(), "swap" | "remove") {
        7
    } else {
        return Err(format!(
            "unknown market operation {action:?}. Try: atlantis edit region --help"
        ));
    };
    require_count(
        command,
        expected,
        "market operation has missing arguments. Try: atlantis edit region --help",
    )?;
    let item =
        find_item(loaded, &command[6]).ok_or_else(|| format!("unknown item {:?}", command[6]))?;
    let item_label = item_name(loaded, item);
    let old = market_item_description(loaded, region_id, item);
    match action.as_str() {
        "population" => {
            let minimum = parse_number(&command[7], "minimum population")?;
            let maximum = parse_number(&command[8], "maximum population")?;
            set_market_population_range(loaded, region_id, item, minimum, maximum)
                .map_err(|error| error.to_string())?;
        }
        "amount" => {
            let minimum = parse_number(&command[7], "minimum amount")?;
            let maximum = parse_number(&command[8], "maximum amount")?;
            set_market_amount_range(loaded, region_id, item, minimum, maximum)
                .map_err(|error| error.to_string())?;
        }
        "prices" => {
            let price = parse_number(&command[7], "market price")?;
            let base = parse_number(&command[8], "base price")?;
            set_market_prices(loaded, region_id, item, price, base)
                .map_err(|error| error.to_string())?;
        }
        "swap" => swap_market_type(loaded, region_id, item).map_err(|error| error.to_string())?,
        "remove" => {
            remove_market(loaded, region_id, item).map_err(|error| error.to_string())?;
        }
        _ => unreachable!("market operation was checked above"),
    }
    Ok(format!(
        "{context}: market {item_label} {old} -> {}",
        market_item_description(loaded, region_id, item)
    ))
}

fn apply_unit(loaded: &mut LoadedGame, command: &[String]) -> Result<String, String> {
    if command.len() < 2 {
        return Err("unit needs a number and an edit. Try: atlantis edit unit --help".to_owned());
    }
    if is_equal(&command[1], "create") {
        return create_unit_command(loaded, command);
    }
    let number = parse_number(&command[1], "unit number")?;
    let unit_id = find_unit(&loaded.game, number)
        .ok_or_else(|| format!("there is no unit numbered {number}"))?;
    let context = unit_context(loaded, unit_id);

    match command
        .get(2)
        .map(|action| action.to_ascii_lowercase())
        .as_deref()
    {
        Some("set-item") => {
            require_count(
                command,
                5,
                "set-item needs an item and amount. Try: atlantis edit unit --help",
            )?;
            let item_id = find_item(loaded, &command[3])
                .ok_or_else(|| format!("unknown item {:?}", command[3]))?;
            let item_name = item_name(loaded, item_id);
            let amount = parse_number(&command[4], "item amount")?;
            let unit = loaded
                .game
                .unit(unit_id)
                .ok_or_else(|| format!("there is no unit numbered {number}"))?;
            let old_amount = unit.items.get_num(item_id);
            set_unit_item(loaded, unit_id, item_id, amount).map_err(|error| error.to_string())?;
            Ok(format!("{context}: {item_name} {old_amount} -> {amount}"))
        }
        Some("set-skill") => {
            require_count(
                command,
                5,
                "set-skill needs a skill and days per man. Try: atlantis edit unit --help",
            )?;
            let tables = loaded.ruleset.tables();
            let skill = parse_skill(&tables.skills.skills, &command[3])
                .ok_or_else(|| format!("unknown skill {:?}", command[3]))?;
            let name = tables.skills.skills[skill]
                .name
                .or(tables.skills.skills[skill].abbreviation)
                .unwrap_or(&command[3]);
            let days = parse_number(&command[4], "skill days per man")?;
            let old = skill_days_per_man(loaded, unit_id, skill);
            set_unit_skill(loaded, unit_id, skill, days).map_err(|error| error.to_string())?;
            Ok(format!(
                "{context}: skill {} days per man {old} -> {days}",
                name.to_lowercase()
            ))
        }
        Some("set-faction") => {
            require_count(
                command,
                4,
                "set-faction needs a faction number. Try: atlantis edit unit --help",
            )?;
            let faction_number = parse_number(&command[3], "faction number")?;
            let faction_id = find_faction(loaded, faction_number)
                .ok_or_else(|| format!("there is no faction numbered {faction_number}"))?;
            let old = unit_faction_number(loaded, unit_id);
            set_unit_faction(loaded, unit_id, faction_id).map_err(|error| error.to_string())?;
            Ok(format!("{context}: faction {old} -> {faction_number}"))
        }
        Some("set-type") => {
            require_count(
                command,
                4,
                "set-type needs a unit type. Try: atlantis edit unit --help",
            )?;
            let new = parse_unit_type(&command[3])
                .ok_or_else(|| format!("unknown unit type {:?}. Valid: normal mage guard wandering-monster guard-mage apprentice", command[3]))?;
            let old = loaded
                .game
                .unit(unit_id)
                .expect("unit was resolved")
                .unit_type;
            set_unit_type(loaded, unit_id, new).map_err(|error| error.to_string())?;
            Ok(format!(
                "{context}: type {} -> {}",
                unit_type_name(old),
                unit_type_name(new)
            ))
        }
        Some("set-location") => {
            require_count(
                command,
                6,
                "set-location needs X Y Z. Try: atlantis edit unit --help",
            )?;
            let x = parse_number(&command[3], "X coordinate")?;
            let y = parse_number(&command[4], "Y coordinate")?;
            let z = parse_number(&command[5], "Z coordinate")?;
            let region_id = find_region(&loaded.game, x, y, z)
                .ok_or_else(|| format!("there is no region at ({x},{y},{z})"))?;
            let old = unit_location(loaded, unit_id);
            move_unit_to_region(loaded, unit_id, region_id).map_err(|error| error.to_string())?;
            Ok(format!("{context}: location {old} -> ({x},{y},{z})"))
        }
        Some("set-name") => {
            let name = argument_text(command, 3, "set-name needs a unit name")?;
            let old = loaded
                .game
                .unit(unit_id)
                .and_then(|unit| unit.name.clone())
                .unwrap_or_else(|| "none".to_owned());
            rename_unit(loaded, unit_id, &name).map_err(|error| error.to_string())?;
            Ok(format!("{context}: name \"{old}\" -> \"{name}\""))
        }
        Some("set-description") => {
            let description = argument_text(command, 3, "set-description needs text")?;
            let old = loaded
                .game
                .unit(unit_id)
                .and_then(|unit| unit.describe.clone())
                .unwrap_or_else(|| "none".to_owned());
            set_unit_description(loaded, unit_id, Some(&description))
                .map_err(|error| error.to_string())?;
            Ok(format!(
                "{context}: description \"{old}\" -> \"{description}\""
            ))
        }
        Some("clear-description") => {
            require_count(command, 3, "clear-description takes no arguments")?;
            let old = loaded
                .game
                .unit(unit_id)
                .and_then(|unit| unit.describe.clone())
                .unwrap_or_else(|| "none".to_owned());
            set_unit_description(loaded, unit_id, None).map_err(|error| error.to_string())?;
            Ok(format!("{context}: description \"{old}\" -> none"))
        }
        Some("set-guard") => {
            require_count(
                command,
                4,
                "set-guard needs a guard mode. Try: atlantis edit unit --help",
            )?;
            let new = parse_guard_mode(&command[3]).ok_or_else(|| {
                format!(
                    "unknown guard mode {:?}. Valid: none guard avoid set advance",
                    command[3]
                )
            })?;
            let old = loaded.game.unit(unit_id).expect("unit was resolved").guard;
            set_unit_guard(loaded, unit_id, new).map_err(|error| error.to_string())?;
            Ok(format!(
                "{context}: guard {} -> {}",
                guard_mode_name(old),
                guard_mode_name(new)
            ))
        }
        Some(other) => Err(format!(
            "unknown unit edit {other:?}. Try: atlantis edit unit --help"
        )),
        None => Err("unit needs an edit. Try: atlantis edit unit --help".to_owned()),
    }
}

fn create_unit_command(loaded: &mut LoadedGame, command: &[String]) -> Result<String, String> {
    require_count(
        command,
        6,
        "unit create needs a faction number and region X Y Z. Try: atlantis edit unit --help",
    )?;
    let faction_number = parse_number(&command[2], "faction number")?;
    let x = parse_number(&command[3], "X coordinate")?;
    let y = parse_number(&command[4], "Y coordinate")?;
    let z = parse_number(&command[5], "Z coordinate")?;
    let faction_id = find_faction(loaded, faction_number)
        .ok_or_else(|| format!("there is no faction numbered {faction_number}"))?;
    let region_id = find_region(&loaded.game, x, y, z)
        .ok_or_else(|| format!("there is no region at ({x},{y},{z})"))?;
    let unit_id = create_unit(loaded, faction_id, region_id).map_err(|error| error.to_string())?;
    let unit = loaded
        .game
        .unit(unit_id)
        .expect("created unit exists after operation");
    Ok(format!(
        "Unit {}: created none -> faction {faction_number} in ({x},{y},{z})",
        unit.num
    ))
}

fn require_count(command: &[String], expected: usize, message: &str) -> Result<(), String> {
    if command.len() == expected {
        Ok(())
    } else {
        Err(message.to_owned())
    }
}

fn argument_text(command: &[String], start: usize, message: &str) -> Result<String, String> {
    let value = command
        .get(start..)
        .unwrap_or_default()
        .join(" ")
        .trim()
        .to_owned();
    if value.is_empty() {
        Err(message.to_owned())
    } else {
        Ok(value)
    }
}

fn find_item(loaded: &LoadedGame, token: &str) -> Option<ItemId> {
    loaded
        .ruleset
        .tables()
        .item_families
        .items
        .iter()
        .enumerate()
        .find(|(_, item)| {
            item.flags & ItemFlags::DISABLED == 0
                && (item
                    .name
                    .as_deref()
                    .is_some_and(|name| is_equal(name, token))
                    || item
                        .abbreviation
                        .as_deref()
                        .is_some_and(|abbreviation| is_equal(abbreviation, token)))
        })
        .and_then(|(index, _)| ItemId::from_index(index))
}

fn find_race(loaded: &LoadedGame, token: &str) -> Option<ItemId> {
    loaded
        .ruleset
        .tables()
        .item_families
        .items
        .iter()
        .enumerate()
        .find(|(_, item)| {
            item.item_type & ItemTypeFlags::MAN != 0
                && item.flags & ItemFlags::DISABLED == 0
                && (item
                    .name
                    .as_deref()
                    .is_some_and(|name| is_equal(name, token))
                    || item
                        .abbreviation
                        .as_deref()
                        .is_some_and(|abbreviation| is_equal(abbreviation, token)))
        })
        .and_then(|(index, _)| ItemId::from_index(index))
}

fn item_name(loaded: &LoadedGame, item: ItemId) -> String {
    loaded
        .ruleset
        .tables()
        .item_families
        .items
        .get(item.index())
        .and_then(|definition| definition.name.as_deref())
        .map(str::to_lowercase)
        .unwrap_or_else(|| item.index().to_string())
}

fn region_context(loaded: &LoadedGame, region_id: RegionId) -> String {
    let region = loaded
        .game
        .region(region_id)
        .expect("resolved region exists");
    format!(
        "Region ({},{}) \"{}\"",
        region.xloc, region.yloc, region.name
    )
}

fn region_race_name(loaded: &LoadedGame, region_id: RegionId) -> String {
    let race = loaded
        .game
        .region(region_id)
        .expect("resolved region exists")
        .race;
    usize::try_from(race)
        .ok()
        .and_then(ItemId::from_index)
        .map(|item| item_name(loaded, item))
        .unwrap_or_else(|| "none".to_owned())
}

fn products_description(loaded: &LoadedGame, region_id: RegionId) -> String {
    let region = loaded
        .game
        .region(region_id)
        .expect("resolved region exists");
    let products = region
        .products
        .iter()
        .map(|product| {
            let name = usize::try_from(product.item_type)
                .ok()
                .and_then(ItemId::from_index)
                .map(|item| item_name(loaded, item))
                .unwrap_or_else(|| product.item_type.to_string());
            format!("{name} {}", product.amount)
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{products}]")
}

fn region_state_description(loaded: &LoadedGame, region_id: RegionId) -> String {
    let region = loaded
        .game
        .region(region_id)
        .expect("resolved region exists");
    let terrain = region
        .terrain_type
        .and_then(|id| loaded.ruleset.tables().terrains.get(id).map(|def| def.name))
        .unwrap_or("none");
    format!(
        "terrain {terrain}, race {}, wages {}, population {}, products {}, town {}, markets {}",
        region_race_name(loaded, region_id),
        region.max_wages,
        region.population,
        products_description(loaded, region_id),
        town_description(loaded, region_id),
        markets_description(loaded, region_id)
    )
}

fn town_description(loaded: &LoadedGame, region_id: RegionId) -> String {
    loaded
        .game
        .region(region_id)
        .and_then(|region| region.town.as_ref())
        .map(|town| {
            format!(
                "{} (population {})",
                town.name.as_deref().unwrap_or("unnamed"),
                town.population
            )
        })
        .unwrap_or_else(|| "none".to_owned())
}

fn market_item_description(loaded: &LoadedGame, region_id: RegionId, item: ItemId) -> String {
    let region = loaded
        .game
        .region(region_id)
        .expect("resolved region exists");
    let markets = region
        .markets
        .iter()
        .filter(|market| market.item == item as i32)
        .map(|market| market_description(market, &item_name(loaded, item)))
        .collect::<Vec<_>>();
    if markets.is_empty() {
        "none".to_owned()
    } else {
        markets.join(", ")
    }
}

fn markets_description(loaded: &LoadedGame, region_id: RegionId) -> String {
    let region = loaded
        .game
        .region(region_id)
        .expect("resolved region exists");
    let tables = loaded.ruleset.tables();
    let markets = region
        .markets
        .iter()
        .map(|market| {
            let name = usize::try_from(market.item)
                .ok()
                .and_then(ItemId::from_index)
                .and_then(|item| {
                    tables
                        .item_families
                        .items
                        .get(item.index())
                        .and_then(|definition| definition.name.as_deref())
                })
                .unwrap_or("unknown");
            market_description(market, name)
        })
        .collect::<Vec<_>>();
    format!("[{}]", markets.join(", "))
}

fn market_description(market: &Market, item_name: &str) -> String {
    let kind = match market.market_type {
        MarketType::Buy => "buy",
        MarketType::Sell => "sell",
    };
    format!(
        "{kind} {item_name} price {} amount {} population {}..{} range {}..{}",
        market.price,
        market.amount,
        market.min_population,
        market.max_population,
        market.min_amount,
        market.max_amount
    )
}

fn parse_object_index(token: &str) -> Result<usize, String> {
    let index = token
        .parse::<usize>()
        .map_err(|_| format!("invalid object index {token:?}"))?;
    if index == 0 || index.checked_add(1).is_none() {
        return Err("object index starts at 1 and excludes the region dummy".to_owned());
    }
    Ok(index)
}

fn object_at(
    loaded: &LoadedGame,
    region_id: RegionId,
    displayed_index: usize,
) -> Result<String, String> {
    let api_index = object_api_index(loaded, region_id, displayed_index)?;
    let object_id = loaded
        .game
        .region(region_id)
        .and_then(|region| region.object_ids().get(api_index))
        .copied()
        .ok_or_else(|| format!("region object {displayed_index} does not exist"))?;
    let object = loaded
        .game
        .object(object_id)
        .ok_or_else(|| format!("region object {displayed_index} does not exist"))?;
    Ok(object_description(object, &loaded.ruleset.tables().objects))
}

fn object_api_index(
    loaded: &LoadedGame,
    region_id: RegionId,
    displayed_index: usize,
) -> Result<usize, String> {
    let ordinal = displayed_index
        .checked_sub(1)
        .ok_or_else(|| "object index starts at 1 and excludes the region dummy".to_owned())?;
    let object_id = loaded
        .game
        .region(region_id)
        .ok_or_else(|| format!("region object {displayed_index} does not exist"))?
        .object_ids()
        .iter()
        .copied()
        .filter(|object_id| {
            loaded
                .game
                .object(*object_id)
                .is_some_and(|object| object.object_type != O_DUMMY as i32)
        })
        .nth(ordinal)
        .ok_or_else(|| format!("region object {displayed_index} does not exist"))?;
    loaded
        .game
        .region(region_id)
        .and_then(|region| {
            region
                .object_ids()
                .iter()
                .position(|candidate| *candidate == object_id)
        })
        .ok_or_else(|| format!("region object {displayed_index} does not exist"))
}

fn object_description(
    object: &atlantis_engine::model::Object,
    object_defs: &[atlantis_engine::defs::ObjectDef],
) -> String {
    let kind = usize::try_from(object.object_type)
        .ok()
        .and_then(|index| object_defs.get(index))
        .map(|definition| definition.name)
        .unwrap_or("object");
    format!("{kind} ({})", object.name.as_deref().unwrap_or(kind))
}

fn find_faction(loaded: &LoadedGame, number: i32) -> Option<atlantis_engine::model::FactionId> {
    loaded.game.faction_ids().iter().copied().find(|id| {
        loaded
            .game
            .faction(*id)
            .is_some_and(|faction| faction.num == number)
    })
}

fn unit_context(loaded: &LoadedGame, unit_id: atlantis_engine::model::UnitId) -> String {
    let unit = loaded.game.unit(unit_id).expect("resolved unit exists");
    format!(
        "Unit {} ({})",
        unit.name.as_deref().unwrap_or("Unit"),
        unit.num
    )
}

fn unit_faction_number(loaded: &LoadedGame, unit_id: atlantis_engine::model::UnitId) -> String {
    loaded
        .game
        .unit(unit_id)
        .and_then(|unit| unit.faction)
        .and_then(|faction_id| loaded.game.faction(faction_id))
        .map(|faction| faction.num.to_string())
        .unwrap_or_else(|| "none".to_owned())
}

fn unit_location(loaded: &LoadedGame, unit_id: atlantis_engine::model::UnitId) -> String {
    loaded
        .game
        .unit(unit_id)
        .and_then(|unit| unit.object)
        .and_then(|object_id| loaded.game.object(object_id))
        .and_then(|object| object.region)
        .and_then(|region_id| loaded.game.region(region_id))
        .map(|region| format!("({},{},{})", region.xloc, region.yloc, region.zloc))
        .unwrap_or_else(|| "none".to_owned())
}

fn skill_days_per_man(
    loaded: &LoadedGame,
    unit_id: atlantis_engine::model::UnitId,
    skill: usize,
) -> u32 {
    let tables = loaded.ruleset.tables();
    let unit = loaded.game.unit(unit_id).expect("resolved unit exists");
    let men: i32 = unit
        .items
        .entries()
        .iter()
        .filter(|entry| {
            tables
                .item_families
                .items
                .get(entry.item_type.index())
                .is_some_and(|item| item.item_type & ItemTypeFlags::MAN != 0)
        })
        .map(|entry| entry.num)
        .sum();
    if men <= 0 {
        0
    } else {
        unit.skills.get_days(skill) / men as u32
    }
}

fn parse_unit_type(token: &str) -> Option<UnitType> {
    [
        ("normal", UnitType::Normal),
        ("mage", UnitType::Mage),
        ("guard", UnitType::Guard),
        ("wandering-monster", UnitType::WanderingMonster),
        ("guard-mage", UnitType::GuardMage),
        ("apprentice", UnitType::Apprentice),
    ]
    .into_iter()
    .find(|(name, _)| is_equal(token, name))
    .map(|(_, unit_type)| unit_type)
}

fn unit_type_name(unit_type: UnitType) -> &'static str {
    match unit_type {
        UnitType::Normal => "normal",
        UnitType::Mage => "mage",
        UnitType::Guard => "guard",
        UnitType::WanderingMonster => "wandering-monster",
        UnitType::GuardMage => "guard-mage",
        UnitType::Apprentice => "apprentice",
    }
}

fn parse_guard_mode(token: &str) -> Option<GuardMode> {
    [
        ("none", GuardMode::None),
        ("guard", GuardMode::Guard),
        ("avoid", GuardMode::Avoid),
        ("set", GuardMode::Set),
        ("advance", GuardMode::Advance),
    ]
    .into_iter()
    .find(|(name, _)| is_equal(token, name))
    .map(|(_, guard)| guard)
}

fn guard_mode_name(guard: GuardMode) -> &'static str {
    match guard {
        GuardMode::None => "none",
        GuardMode::Guard => "guard",
        GuardMode::Avoid => "avoid",
        GuardMode::Set => "set",
        GuardMode::Advance => "advance",
    }
}

fn parse_number(token: &str, description: &str) -> Result<i32, String> {
    token
        .parse::<i32>()
        .map_err(|_| format!("invalid {description} {token:?}"))
}

fn save_game(
    loaded: &mut LoadedGame,
    json_format: bool,
    input_path: &Path,
    output_path: Option<&Path>,
) -> Result<Vec<String>, String> {
    let serialized = if json_format {
        save_json(loaded)
    } else {
        save_legacy(loaded)
    }
    .map_err(|error| error.to_string())?;

    if let Some(output_path) = output_path {
        let same_path = match (fs::canonicalize(input_path), fs::canonicalize(output_path)) {
            (Ok(input), Ok(output)) => input == output,
            _ => input_path == output_path,
        };
        if same_path {
            return Err("--out must be different from the input file".to_owned());
        }
        fs::write(output_path, serialized)
            .map_err(|error| format!("could not write {}: {error}", output_path.display()))?;
        return Ok(vec![format!("Saved {}", output_path.display())]);
    }

    let backup_path = backup_path(input_path);
    fs::copy(input_path, &backup_path).map_err(|error| {
        format!(
            "could not keep previous version as {}: {error}",
            backup_path.display()
        )
    })?;
    fs::write(input_path, serialized)
        .map_err(|error| format!("could not write {}: {error}", input_path.display()))?;
    Ok(vec![
        format!("Saved {}", input_path.display()),
        format!("Previous version kept as {}", backup_path.display()),
    ])
}

fn backup_path(path: &Path) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(".bak");
    PathBuf::from(name)
}

fn fail(message: &str) -> i32 {
    let separator = if message.ends_with("...") || message.ends_with('.') {
        " "
    } else {
        ". "
    };
    println!("error: {message}{separator}Nothing was changed.");
    2
}

fn print_general_help() {
    println!("Usage: atlantis <command> ...\nCommands: edit");
}

fn print_edit_help() {
    println!(
        "Usage: atlantis edit <what> ... [--out FILE] [--dry-run]\n\
         region X Y Z   set-terrain TERRAIN | set-race RACE | set-wages AMOUNT | set-name NAME\n\
                        regenerate-products | regenerate-all | add-gate | remove-gate\n\
                        add-town | regenerate-town | remove-town | rename-town NAME\n\
                        add-object TYPE | remove-object INDEX | rename-object INDEX NAME\n\
                        market population ITEM MIN MAX | market amount ITEM MIN MAX\n\
                        market prices ITEM PRICE BASE | market swap ITEM | market remove ITEM\n\
                        market regenerate\n\
         unit NUM       set-item ITEM AMOUNT | set-skill SKILL DAYS | set-faction FACTION_ID\n\
                        set-type TYPE | set-location X Y Z | set-name NAME | set-description TEXT\n\
                        set-guard MODE\n\
         unit create FACTION_ID X Y Z\n\
         --script FILE  apply every line of FILE, all or nothing (# starts a comment)\n\
         Names are not case-sensitive; item and skill abbreviations (SILV) work as well as full names."
    );
}

fn print_region_help() {
    println!(
        "Region edits:\n\
         set-terrain TERRAIN | set-race RACE | set-wages AMOUNT | set-name NAME\n\
         regenerate-products | regenerate-all | add-gate | remove-gate\n\
         add-town | regenerate-town | remove-town | rename-town NAME\n\
         add-object TYPE | remove-object INDEX | rename-object INDEX NAME\n\
         market population ITEM MIN MAX | market amount ITEM MIN MAX\n\
         market prices ITEM PRICE BASE | market swap ITEM | market remove ITEM\n\
         market regenerate"
    );
}

fn print_unit_help() {
    println!(
        "Unit edits:\n\
         set-item ITEM AMOUNT | set-skill SKILL DAYS | set-faction FACTION_ID\n\
         set-type TYPE | set-location X Y Z | set-name NAME | set-description TEXT\n\
         clear-description | set-guard MODE | create FACTION_ID X Y Z"
    );
}
