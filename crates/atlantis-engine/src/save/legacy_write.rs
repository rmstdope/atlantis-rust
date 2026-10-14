use std::{
    collections::BTreeSet,
    fmt::{self, Write as _},
};

use crate::defs::{ItemTypeFlags, RulesetTables};
use crate::model::{
    Faction, FactionId, Game, ItemList, Market, Object, Quest, Region, SkillList, Unit,
};
use crate::ruleset::Ruleset;
use crate::text::is_equal;
use crate::version::CURRENT_ATL_VER;

use super::legacy::{LoadedGame, SaveError};

/// C++: `game.cpp` `Game::SaveGame`; writes the legacy positional save in memory.
pub fn save_legacy(loaded: &mut LoadedGame) -> Result<String, SaveError> {
    let mut output = String::new();
    let game = &loaded.game;
    let ruleset = loaded.ruleset;
    let defs = ruleset.game_defs();
    let tables = ruleset.tables();

    put_str(&mut output, "atlantis_game");
    put_line(&mut output, CURRENT_ATL_VER.packed());
    put_str(&mut output, ruleset.name());
    put_line(&mut output, defs.ruleset_version.packed());
    put_line(&mut output, game.year);
    put_line(&mut output, game.month);
    put_line(&mut output, loaded.rng.get_random(10_000));
    put_line(&mut output, game.faction_seq);
    put_line(&mut output, game.unit_seq);
    put_line(&mut output, game.ship_seq);
    put_line(
        &mut output,
        faction_number(game, game.guard_faction, "guard faction")?,
    );
    put_line(
        &mut output,
        faction_number(game, game.monster_faction, "monster faction")?,
    );
    put_line(&mut output, game.faction_ids().len());

    for faction_id in game.faction_ids() {
        let faction = game
            .faction(*faction_id)
            .ok_or_else(|| SaveError::InvalidValue {
                field: "faction",
                value: faction_id.index().to_string(),
            })?;
        write_faction(&mut output, faction, ruleset, &tables)?;
    }

    write_regions(&mut output, game, &tables, defs)?;
    write_quests(&mut output, &game.quests, &tables)?;
    Ok(output)
}

/// C++: `quests.cpp` `QuestList::WriteQuests`; destination sets serialize in sorted order.
fn write_quests(
    output: &mut String,
    quests: &[Quest],
    tables: &RulesetTables,
) -> Result<(), SaveError> {
    put_line(output, quests.len());
    for quest in quests {
        put_line(output, quest.quest_type);
        match quest.quest_type {
            0 => put_line(output, quest.target),
            1 => {
                write_quest_objective(output, quest, tables)?;
                put_line(output, quest.region_num);
            }
            2 => {
                put_str(output, &object_symbol_by_index(tables, quest.building)?);
                put_str(output, &quest.region_name);
            }
            3 => {
                put_str(output, &object_symbol_by_index(tables, quest.building)?);
                write_destinations(output, &quest.destinations);
            }
            5 => {
                put_line(output, quest.target);
                put_line(output, quest.region_num);
            }
            _ => {
                put_line(output, quest.target);
                write_quest_objective(output, quest, tables)?;
                put_str(output, &object_symbol_by_index(tables, quest.building)?);
                put_line(output, quest.region_num);
                put_str(output, &quest.region_name);
                write_destinations(output, &quest.destinations);
            }
        }
        write_item_list(output, &quest.rewards, tables)?;
    }
    Ok(())
}

/// C++: `quests.cpp` writes an objective through the positional item record.
fn write_quest_objective(
    output: &mut String,
    quest: &Quest,
    tables: &RulesetTables,
) -> Result<(), SaveError> {
    let symbol = item_symbol_by_index(tables, quest.objective_type, "quest objective")?;
    put_line(output, format_args!("{} {symbol}", quest.objective_num));
    Ok(())
}

fn object_symbol_by_index(tables: &RulesetTables, index: i32) -> Result<&'static str, SaveError> {
    let index = usize::try_from(index)
        .map_err(|_| invalid_index("quest building", index.unsigned_abs() as usize))?;
    tables
        .objects
        .get(index)
        .map(|definition| definition.name)
        .ok_or_else(|| invalid_index("quest building", index))
}

fn write_destinations(output: &mut String, destinations: &BTreeSet<String>) {
    put_line(output, destinations.len());
    for destination in destinations {
        put_str(output, destination);
    }
}

/// C++: `aregion.cpp` `ARegionList::WriteRegions`; keeps the region and neighbor list order.
fn write_regions(
    output: &mut String,
    game: &Game,
    tables: &RulesetTables,
    defs: &crate::defs::GameDefs,
) -> Result<(), SaveError> {
    put_line(output, game.region_ids().len());
    put_line(output, game.region_arrays.len());
    for array in &game.region_arrays {
        put_line(output, array.x);
        put_line(output, array.y);
        put_str(
            output,
            if array.name.is_empty() {
                "none"
            } else {
                &array.name
            },
        );
        put_line(output, array.level_type as i32);
    }
    put_line(output, game.number_of_gates);

    for region_id in game.region_ids() {
        let region = game
            .region(*region_id)
            .ok_or_else(|| invalid_index("region", region_id.index()))?;
        write_region(output, region, game, tables, defs)?;
    }

    put_str(output, "Neighbors");
    for region_id in game.region_ids() {
        let region = game
            .region(*region_id)
            .ok_or_else(|| invalid_index("region", region_id.index()))?;
        for neighbor in &region.neighbors {
            let number = neighbor
                .map(|id| {
                    game.region(id)
                        .map(|neighbor| neighbor.num)
                        .ok_or_else(|| invalid_index("neighbor region", id.index()))
                })
                .unwrap_or(Ok(-1))?;
            put_line(output, number);
        }
    }
    Ok(())
}

/// C++: `aregion.cpp` `ARegion::Writeout`; conditional gate month precedes the race.
fn write_region(
    output: &mut String,
    region: &Region,
    game: &Game,
    tables: &RulesetTables,
    defs: &crate::defs::GameDefs,
) -> Result<(), SaveError> {
    put_str(output, &region.name);
    put_line(output, region.num);
    let terrain = region
        .terrain_type
        .map(|index| {
            tables
                .terrains
                .get(index)
                .map(|terrain| terrain.terrain_type)
                .ok_or_else(|| invalid_index("terrain", index))
        })
        .unwrap_or(Ok("NO_TERRAIN"))?;
    put_str(output, terrain);
    put_line(output, region.building_seq);
    put_line(output, region.gate);
    if region.gate > 0 {
        put_line(output, region.gate_month);
    }
    put_str(
        output,
        &item_symbol_by_index(tables, region.race, "region race")?,
    );
    for value in [
        region.population,
        region.base_population,
        region.wages,
        region.max_wages,
        region.wealth,
        region.elevation,
        region.humidity,
        region.temperature,
        region.vegetation,
        region.culture,
        region.habitat,
        region.development,
        region.max_development,
    ] {
        put_line(output, value);
    }
    if let Some(town) = &region.town {
        put_line(output, 1);
        put_str(output, required_text(town.name.as_deref(), "town name")?);
        put_line(output, town.population);
        put_line(output, town.habitat);
    } else {
        put_line(output, 0);
    }
    for value in [region.xloc, region.yloc, region.zloc, region.visited] {
        put_line(output, value);
    }
    put_line(output, region.products.len());
    for production in &region.products {
        put_str(
            output,
            &item_symbol_by_index(tables, production.item_type, "production item")?,
        );
        put_line(output, production.amount);
        put_line(output, production.base_amount);
        if production.item_type == crate::defs::ItemId::Silver as i32 {
            put_str(
                output,
                skill_abbreviation(tables, production.skill, "silver production skill")?,
            );
        }
        put_line(output, production.productivity);
    }
    put_line(output, region.markets.len());
    for market in &region.markets {
        write_market(output, market, tables)?;
    }
    put_line(output, region.object_ids().len());
    for object_id in region.object_ids() {
        let object = game
            .object(*object_id)
            .ok_or_else(|| invalid_index("object", object_id.index()))?;
        write_object(output, object, game, tables, defs)?;
    }
    Ok(())
}

/// C++: `market.cpp` `MarketList::Writeout`.
fn write_market(
    output: &mut String,
    market: &Market,
    tables: &RulesetTables,
) -> Result<(), SaveError> {
    put_line(output, market.market_type as i32);
    put_str(
        output,
        &item_symbol_by_index(tables, market.item, "market item")?,
    );
    for value in [
        market.price,
        market.amount,
        market.min_population,
        market.max_population,
        market.min_amount,
        market.max_amount,
        market.base_price,
    ] {
        put_line(output, value);
    }
    Ok(())
}

/// C++: `object.cpp` `Object::Writeout`; fleet records carry their ship list after their units.
fn write_object(
    output: &mut String,
    object: &Object,
    game: &Game,
    tables: &RulesetTables,
    defs: &crate::defs::GameDefs,
) -> Result<(), SaveError> {
    let object_index = usize::try_from(object.object_type)
        .map_err(|_| invalid_index("object type", object.object_type.unsigned_abs() as usize))?;
    let definition = tables
        .objects
        .get(object_index)
        .ok_or_else(|| invalid_index("object type", object_index))?;
    let is_fleet = object
        .is_fleet(&tables.objects)
        .ok_or_else(|| invalid_index("object type", object_index))?;
    put_line(output, object.num);
    put_str(output, definition.name);
    put_line(output, object.incomplete);
    put_str(
        output,
        required_text(object.name.as_deref(), "object name")?,
    );
    put_str(output, object.describe.as_deref().unwrap_or("none"));
    put_line(output, object.inner);
    let prevdir = if defs.prevent_sail_through != 0 && defs.allow_trivial_portage == 0 {
        object.prevdir
    } else {
        -1
    };
    put_line(output, prevdir);
    put_line(output, object.runes);
    put_line(output, object.unit_ids().len());
    for unit_id in object.unit_ids() {
        let unit = game
            .unit(*unit_id)
            .ok_or_else(|| invalid_index("unit", unit_id.index()))?;
        write_unit(output, unit, game, tables, defs)?;
    }
    if is_fleet {
        write_item_list(output, &object.ships, tables)?;
    }
    Ok(())
}

/// C++: `faction.cpp` `Faction::Writeout`.

/// C++: `unit.cpp` `Unit::Writeout`; visited locations use ordered set iteration.
fn write_unit(
    output: &mut String,
    unit: &Unit,
    game: &Game,
    tables: &RulesetTables,
    defs: &crate::defs::GameDefs,
) -> Result<(), SaveError> {
    put_str(output, required_text(unit.name.as_deref(), "unit name")?);
    put_str(output, unit.describe.as_deref().unwrap_or("none"));
    put_line(output, unit.num);
    put_line(output, unit.unit_type as i32);
    let faction_id = unit.faction.ok_or_else(|| SaveError::InvalidValue {
        field: "unit faction",
        value: "missing".to_owned(),
    })?;
    put_line(
        output,
        faction_number(game, Some(faction_id), "unit faction")?,
    );
    put_line(output, unit.guard as i32);
    put_line(output, unit.reveal);
    put_line(output, unit.free);
    put_str(
        output,
        &item_symbol_by_index(tables, unit.ready_item, "ready item")?,
    );
    for (weapon, armor) in unit.ready_weapon.iter().zip(unit.ready_armor) {
        put_str(
            output,
            &item_symbol_by_index(tables, *weapon, "ready weapon")?,
        );
        put_str(output, &item_symbol_by_index(tables, armor, "ready armor")?);
    }
    put_line(output, unit.flags);
    write_item_list(output, &unit.items, tables)?;
    write_skill_list(output, &unit.skills, tables, defs.required_experience != 0)?;
    put_str(
        output,
        skill_abbreviation(tables, unit.combat, "combat skill")?,
    );
    put_line(output, unit.saved_movement);
    put_line(output, unit.saved_move_dir);
    put_line(output, unit.visited.len());
    for visited in &unit.visited {
        put_str(output, visited);
    }
    Ok(())
}

fn write_faction(
    output: &mut String,
    faction: &Faction,
    ruleset: &dyn Ruleset,
    tables: &RulesetTables,
) -> Result<(), SaveError> {
    let faction_types = ruleset.faction_types();
    if faction.faction_types.len() != faction_types.len() {
        return Err(SaveError::InvalidValue {
            field: "faction type count",
            value: faction.faction_types.len().to_string(),
        });
    }
    put_line(output, faction.num);
    for ((name, value), expected_name) in faction.faction_types.iter().zip(faction_types) {
        if !is_equal(name, expected_name) {
            return Err(SaveError::InvalidValue {
                field: "faction type",
                value: name.clone(),
            });
        }
        put_line(output, value);
    }
    put_line(output, faction.last_change);
    put_line(output, faction.last_orders);
    put_line(output, faction.unclaimed);
    put_str(
        output,
        required_text(faction.name.as_deref(), "faction name")?,
    );
    put_str(
        output,
        required_text(faction.address.as_deref(), "faction address")?,
    );
    put_str(
        output,
        required_text(faction.password.as_deref(), "faction password")?,
    );
    put_line(output, faction.times);
    put_line(output, faction.show_unit_attitudes);
    put_line(output, faction.template_format);
    write_skill_list(
        output,
        &faction.skills,
        tables,
        ruleset.game_defs().required_experience != 0,
    )?;
    write_item_list(output, &faction.items, tables)?;
    put_line(output, faction.default_attitude);
    put_line(output, faction.attitudes.len());
    for attitude in &faction.attitudes {
        put_line(output, attitude.faction_num);
        put_line(output, attitude.attitude);
    }
    Ok(())
}

/// C++: `items.cpp` `ItemList::Writeout`.
fn write_item_list(
    output: &mut String,
    items: &ItemList,
    tables: &RulesetTables,
) -> Result<(), SaveError> {
    put_line(output, items.entries().len());
    for item in items.entries() {
        let symbol = item_symbol_by_index(tables, item.item_type.index() as i32, "item")?;
        put_line(output, format_args!("{} {symbol}", item.num));
    }
    Ok(())
}

fn item_symbol_by_index(
    tables: &RulesetTables,
    index: i32,
    field: &'static str,
) -> Result<String, SaveError> {
    if index == -1
        && matches!(
            field,
            "market item" | "region race" | "ready item" | "ready weapon" | "ready armor"
        )
    {
        return Ok(if field == "region race" {
            "NO_RACE".to_owned()
        } else {
            "NO_ITEM".to_owned()
        });
    }
    let index =
        usize::try_from(index).map_err(|_| invalid_index(field, index.unsigned_abs() as usize))?;
    let definition = tables
        .item_families
        .items
        .get(index)
        .ok_or_else(|| invalid_index(field, index))?;
    let abbreviation = definition
        .abbreviation
        .as_deref()
        .filter(|abbreviation| !abbreviation.is_empty())
        .ok_or_else(|| invalid_index("item abbreviation", index))?;
    Ok(if definition.item_type & ItemTypeFlags::ILLUSION != 0 {
        format!("i{abbreviation}")
    } else {
        abbreviation.to_owned()
    })
}

fn skill_abbreviation(
    tables: &RulesetTables,
    index: i32,
    field: &'static str,
) -> Result<&'static str, SaveError> {
    if index == -1 {
        return Ok("NO_SKILL");
    }
    let index =
        usize::try_from(index).map_err(|_| invalid_index(field, index.unsigned_abs() as usize))?;
    tables
        .skills
        .skills
        .get(index)
        .and_then(|definition| definition.abbreviation)
        .ok_or_else(|| invalid_index(field, index))
}

/// C++: `skills.cpp` `SkillList::Writeout`.
fn write_skill_list(
    output: &mut String,
    skills: &SkillList,
    tables: &RulesetTables,
    required_experience: bool,
) -> Result<(), SaveError> {
    put_line(output, skills.entries().len());
    for skill in skills.entries() {
        let abbreviation = tables
            .skills
            .skills
            .get(skill.skill_type)
            .and_then(|definition| definition.abbreviation)
            .ok_or_else(|| invalid_index("skill", skill.skill_type))?;
        if required_experience {
            put_line(
                output,
                format_args!("{abbreviation} {} {}", skill.days, skill.exp),
            );
        } else {
            put_line(output, format_args!("{abbreviation} {}", skill.days));
        }
    }
    Ok(())
}

fn faction_number(
    game: &Game,
    faction_id: Option<FactionId>,
    field: &'static str,
) -> Result<i32, SaveError> {
    faction_id
        .map(|id| {
            game.faction(id)
                .map(|faction| faction.num)
                .ok_or_else(|| invalid_index(field, id.index()))
        })
        .unwrap_or(Ok(-1))
}

fn required_text<'a>(value: Option<&'a str>, field: &'static str) -> Result<&'a str, SaveError> {
    value.ok_or_else(|| SaveError::InvalidValue {
        field,
        value: "missing".to_owned(),
    })
}

fn invalid_index(field: &'static str, index: usize) -> SaveError {
    SaveError::InvalidValue {
        field,
        value: index.to_string(),
    }
}

fn put_line(output: &mut String, value: impl fmt::Display) {
    writeln!(output, "{value}").expect("writing into a String cannot fail");
}

fn put_str(output: &mut String, value: &str) {
    output.push_str(value);
    output.push('\n');
}
