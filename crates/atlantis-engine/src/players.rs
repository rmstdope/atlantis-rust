use std::io::Cursor;

use crate::defs::{
    skills::{self, skill_flags},
    GameDefs, ItemFlags, ItemId, ItemTypeFlags, RulesetTables,
};
use crate::model::{
    Faction, FactionId, FactionItemShow, FactionSkillShow, Game, ModelError, Unit, UnitId, UnitType,
};
use crate::text::{get_token, is_equal, value, TextInput};
use crate::version::CURRENT_ATL_VER;

const PLAYERS_HEADER: &str = "AtlantisPlayerStatus";
const GAME_STATUS_UNINITIALIZED: i32 = 0;
const GAME_STATUS_NEW: i32 = 1;
const GAME_STATUS_RUNNING: i32 = 2;
const GAME_STATUS_FINISHED: i32 = 3;
const TEMPLATE_OFF: i32 = 0;
const TEMPLATE_SHORT: i32 = 1;
const TEMPLATE_LONG: i32 = 2;
const TEMPLATE_MAP: i32 = 3;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PlayersError {
    InvalidHeader,
    InvalidVersionLine,
    IncompatibleVersion(i32),
    InvalidGameStatus(String),
    InvalidDirective {
        directive: String,
        reason: &'static str,
    },
    UnsupportedDirective(String),
    MissingFactionField {
        faction_num: i32,
        field: &'static str,
    },
    InvalidTemplateFormat {
        faction_num: i32,
        format: i32,
    },
    Model(ModelError),
    Input(String),
}

impl std::fmt::Display for PlayersError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidHeader => formatter.write_str("players.in has an invalid header"),
            Self::InvalidVersionLine => {
                formatter.write_str("players.in has an invalid Version: line")
            }
            Self::IncompatibleVersion(version) => write!(
                formatter,
                "players.in engine version {version} is incompatible with this version"
            ),
            Self::InvalidGameStatus(status) => {
                write!(formatter, "players.in has an invalid game status: {status}")
            }
            Self::InvalidDirective { directive, reason } => {
                write!(
                    formatter,
                    "invalid players.in directive {directive}: {reason}"
                )
            }
            Self::UnsupportedDirective(directive) => write!(
                formatter,
                "players.in directive {directive} is not implemented yet"
            ),
            Self::MissingFactionField { faction_num, field } => write!(
                formatter,
                "faction {faction_num} is missing the required {field} field"
            ),
            Self::InvalidTemplateFormat {
                faction_num,
                format,
            } => write!(
                formatter,
                "faction {faction_num} has invalid template format {format}"
            ),
            Self::Model(error) => error.fmt(formatter),
            Self::Input(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for PlayersError {}

/// C++: `game.cpp` `Game::ReadPlayers` and `Game::ReadPlayersLine`.
pub fn read_players(
    game: &mut Game,
    input: &str,
    globals: &GameDefs,
    tables: &RulesetTables,
) -> Result<(), PlayersError> {
    let mut input = TextInput::new(Cursor::new(input.as_bytes()));
    let Some(header) = next_line(&mut input)? else {
        return Err(PlayersError::InvalidHeader);
    };
    if !is_equal(&header, PLAYERS_HEADER) {
        return Err(PlayersError::InvalidHeader);
    }

    let Some(mut version_line) = next_line(&mut input)? else {
        return Err(PlayersError::InvalidVersionLine);
    };
    let version_key = get_token(&mut version_line).ok_or(PlayersError::InvalidVersionLine)?;
    let version_value = get_token(&mut version_line).ok_or(PlayersError::InvalidVersionLine)?;
    if !is_equal(&version_key, "Version:") {
        return Err(PlayersError::InvalidVersionLine);
    }
    let version = value(&version_value);
    if !compatible_version(version) {
        return Err(PlayersError::IncompatibleVersion(version));
    }

    if next_line(&mut input)?.is_none() {
        return Err(PlayersError::InvalidDirective {
            directive: "TurnNumber:".to_owned(),
            reason: "missing header line",
        });
    }
    let Some(mut status_line) = next_line(&mut input)? else {
        return Err(PlayersError::InvalidGameStatus(String::new()));
    };
    let status_key = get_token(&mut status_line)
        .ok_or_else(|| PlayersError::InvalidGameStatus(String::new()))?;
    let status = get_token(&mut status_line)
        .ok_or_else(|| PlayersError::InvalidGameStatus(String::new()))?;
    if !is_equal(&status_key, "GameStatus:") {
        return Err(PlayersError::InvalidGameStatus(status));
    }
    game.game_status = if is_equal(&status, "New") {
        GAME_STATUS_NEW
    } else if is_equal(&status, "Running") {
        GAME_STATUS_RUNNING
    } else if is_equal(&status, "Finished") {
        GAME_STATUS_FINISHED
    } else {
        return Err(PlayersError::InvalidGameStatus(status));
    };

    let mut selected_faction = None;
    while let Some(mut line) = next_line(&mut input)? {
        let Some(token) = get_token(&mut line) else {
            continue;
        };

        if is_equal(&token, "Faction:") {
            let number = get_token(&mut line)
                .ok_or_else(|| invalid_directive("Faction:", "missing faction number"))?;
            if is_equal(&number, "new") {
                return Err(PlayersError::UnsupportedDirective(
                    "Faction: new".to_owned(),
                ));
            }
            let faction_number = value(&number);
            selected_faction = game.faction_ids().iter().copied().find(|id| {
                game.faction(*id)
                    .is_some_and(|faction| faction.num == faction_number)
            });
            if let Some(faction_id) = selected_faction {
                let turn_number = game.turn_number();
                game.faction_mut(faction_id)
                    .expect("selected faction exists")
                    .start_turn = turn_number;
            }
            continue;
        }

        if let Some(faction_id) = selected_faction {
            read_players_line(game, faction_id, &token, &line, globals, tables)?;
        }
    }
    Ok(())
}

/// C++: `game.cpp` `Game::WritePlayers`; `faction.cpp` `Faction::WriteFacInfo`.
pub fn write_players(game: &Game) -> Result<String, PlayersError> {
    let status = match game.game_status {
        GAME_STATUS_NEW => "New",
        GAME_STATUS_RUNNING => "Running",
        GAME_STATUS_FINISHED => "Finished",
        GAME_STATUS_UNINITIALIZED => {
            return Err(PlayersError::InvalidDirective {
                directive: "GameStatus:".to_owned(),
                reason: "game status is uninitialized",
            })
        }
        status => {
            return Err(PlayersError::InvalidGameStatus(status.to_string()));
        }
    };

    let mut output = format!(
        "{PLAYERS_HEADER}\nVersion: {}\nTurnNumber: {}\nGameStatus: {status}\n\n",
        CURRENT_ATL_VER.packed(),
        game.turn_number()
    );
    for faction_id in game.faction_ids() {
        let faction = game.faction(*faction_id).expect("ordered faction exists");
        write_faction_info(&mut output, faction)?;
    }
    Ok(output)
}

fn write_faction_info(output: &mut String, faction: &Faction) -> Result<(), PlayersError> {
    let name = required_faction_field(faction, faction.name.as_deref(), "Name")?;
    let address = required_faction_field(faction, faction.address.as_deref(), "Email")?;
    let password = required_faction_field(faction, faction.password.as_deref(), "Password")?;
    let template = match faction.template_format {
        TEMPLATE_OFF => "off",
        TEMPLATE_SHORT => "short",
        TEMPLATE_LONG => "long",
        TEMPLATE_MAP => "map",
        format => {
            return Err(PlayersError::InvalidTemplateFormat {
                faction_num: faction.num,
                format,
            })
        }
    };

    output.push_str(&format!("Faction: {}\n", faction.num));
    output.push_str(&format!("Name: {name}\n"));
    output.push_str(&format!("Email: {address}\n"));
    output.push_str(&format!("Password: {password}\n"));
    output.push_str(&format!("LastOrders: {}\n", faction.last_orders));
    output.push_str(&format!("FirstTurn: {}\n", faction.start_turn));
    output.push_str(&format!("SendTimes: {}\n", faction.times));
    output.push_str(&format!("Template: {template}\n"));
    output.push_str("Battle: na\n");
    for extra_line in &faction.extra_players {
        output.push_str(extra_line);
        output.push('\n');
    }
    Ok(())
}

fn read_players_line(
    game: &mut Game,
    faction_id: FactionId,
    token: &str,
    remainder: &str,
    globals: &GameDefs,
    tables: &RulesetTables,
) -> Result<(), PlayersError> {
    let faction = game
        .faction_mut(faction_id)
        .expect("selected faction exists");
    if is_equal(token, "Name:") {
        let name = remainder.trim();
        if !name.is_empty() {
            faction.name = Some(name.to_owned());
        }
    } else if is_equal(token, "RewardTimes") {
        if globals.times_reward != 0 {
            faction_event(
                faction,
                format!("Times reward of {} silver.", globals.times_reward),
            );
            faction.unclaimed = faction.unclaimed.wrapping_add(globals.times_reward);
        }
    } else if is_equal(token, "Email:") {
        let mut remainder = remainder.to_owned();
        if let Some(address) = get_token(&mut remainder) {
            faction.address = Some(address);
        }
    } else if is_equal(token, "Password:") {
        let password = remainder.trim();
        faction.password = Some(if password.is_empty() {
            "none".to_owned()
        } else {
            password.to_owned()
        });
    } else if is_equal(token, "Battle:") {
        faction.battle_log_format = 0;
    } else if is_equal(token, "Template:") {
        let mut remainder = remainder.to_owned();
        let template = get_token(&mut remainder)
            .and_then(|template| parse_template(&template))
            .unwrap_or(TEMPLATE_LONG);
        faction.template_format = template;
    } else if is_equal(token, "Reward:") {
        let amount = required_value(token, remainder)?;
        faction_event(faction, format!("Reward of {amount} silver."));
        faction.unclaimed = faction.unclaimed.wrapping_add(amount);
    } else if is_equal(token, "SendTimes:") {
        faction.times = required_value(token, remainder)?;
    } else if is_equal(token, "LastOrders:") {
        let last_orders = required_value(token, remainder)?;
        if globals.lastorders_maintained_by_scripts != 0 {
            faction.last_orders = last_orders;
        }
    } else if is_equal(token, "FirstTurn:") {
        faction.start_turn = required_value(token, remainder)?;
    } else if is_equal(token, "Loc:") {
        set_location(game, faction_id, remainder)?;
    } else if is_equal(token, "NewUnit:") {
        create_unit(game, faction_id, remainder, globals)?;
    } else if is_equal(token, "Item:") {
        give_item(game, faction_id, remainder, tables)?;
    } else if is_equal(token, "Skill:") {
        teach_skill(game, faction_id, remainder, globals, tables)?;
    } else if is_equal(token, "Order:") {
        return Err(PlayersError::UnsupportedDirective(token.to_owned()));
    } else {
        game.faction_mut(faction_id)
            .expect("selected faction exists")
            .extra_players
            .push(format!("{token}{remainder}"));
    }
    Ok(())
}

fn set_location(
    game: &mut Game,
    faction_id: FactionId,
    remainder: &str,
) -> Result<(), PlayersError> {
    let mut remainder = remainder.to_owned();
    let x = next_value(&mut remainder, "Loc:")?;
    let y = next_value(&mut remainder, "Loc:")?;
    let z = next_value(&mut remainder, "Loc:")?;
    let Some(region_id) = game.get_region_at(x, y, z) else {
        return Err(invalid_directive(
            "Loc:",
            "coordinates do not identify a region",
        ));
    };
    game.faction_mut(faction_id)
        .expect("selected faction exists")
        .region = Some(region_id);
    Ok(())
}

fn create_unit(
    game: &mut Game,
    faction_id: FactionId,
    remainder: &str,
    globals: &GameDefs,
) -> Result<(), PlayersError> {
    let alias = required_value("NewUnit:", remainder)?;
    if alias == 0 {
        return Err(invalid_directive("NewUnit:", "alias must be nonzero"));
    }
    let region_id = game
        .faction(faction_id)
        .and_then(|faction| faction.region)
        .ok_or_else(|| invalid_directive("NewUnit:", "requires a valid Loc: first"))?;
    let object_id = game
        .get_dummy(region_id)
        .ok_or_else(|| invalid_directive("NewUnit:", "location has no dummy object"))?;
    let move_points = globals
        .phased_move_offset
        .checked_rem(globals.max_speed)
        .ok_or_else(|| invalid_directive("NewUnit:", "ruleset has an invalid maximum speed"))?;

    game.setup_unit_nums().map_err(PlayersError::Model)?;
    let unit_id = game
        .get_new_unit(Some(faction_id), 0, move_points)
        .map_err(PlayersError::Model)?;
    let unit = game.unit_mut(unit_id).expect("new unit exists");
    unit.gm_alias = alias;
    game.attach_unit(object_id, unit_id)
        .map_err(PlayersError::Model)?;
    unit_event(
        game,
        faction_id,
        unit_id,
        "Is given to your faction.".to_owned(),
    );
    Ok(())
}

fn give_item(
    game: &mut Game,
    faction_id: FactionId,
    remainder: &str,
    tables: &RulesetTables,
) -> Result<(), PlayersError> {
    let mut remainder = remainder.to_owned();
    let unit_tag = next_token(&mut remainder, "Item:")?;
    let amount = next_value(&mut remainder, "Item:")?;
    if amount == 0 {
        return Err(invalid_directive("Item:", "item amount must be nonzero"));
    }
    let item_name = next_token(&mut remainder, "Item:")?;
    let item_index = tables
        .item_families
        .parse_all_items(&item_name)
        .ok_or_else(|| invalid_directive("Item:", "unknown item"))?;
    let item_type = ItemId::from_index(item_index)
        .ok_or_else(|| invalid_directive("Item:", "item table index is out of range"))?;
    let unit_id = player_unit(game, faction_id, &unit_tag, "Item:")?;
    let (gm_alias, item_count) = {
        let unit = game.unit(unit_id).expect("resolved unit exists");
        (
            unit.gm_alias,
            unit.items.get_num(item_type).wrapping_add(amount),
        )
    };
    let description = if gm_alias == 0 {
        Some(item_string(tables, item_index, amount)?)
    } else {
        None
    };
    game.unit_mut(unit_id)
        .expect("resolved unit exists")
        .items
        .set_num(item_type, item_count);
    if let Some(description) = description {
        unit_event(
            game,
            faction_id,
            unit_id,
            format!("Is given {description} by the gods."),
        );
    }
    discover_item(game, faction_id, item_type, item_index, false, true, tables);
    Ok(())
}

fn teach_skill(
    game: &mut Game,
    faction_id: FactionId,
    remainder: &str,
    globals: &GameDefs,
    tables: &RulesetTables,
) -> Result<(), PlayersError> {
    let mut remainder = remainder.to_owned();
    let unit_tag = next_token(&mut remainder, "Skill:")?;
    let skill_name = next_token(&mut remainder, "Skill:")?;
    let skill_id = skills::parse_skill(&tables.skills.skills, &skill_name)
        .ok_or_else(|| invalid_directive("Skill:", "unknown or disabled skill"))?;
    let days_per_man = next_value(&mut remainder, "Skill:")?;
    let unit_id = player_unit(game, faction_id, &unit_tag, "Skill:")?;
    let men = unit_men(game.unit(unit_id).expect("resolved unit exists"), tables);
    let days = days_per_man.wrapping_mul(men);
    if days == 0 {
        return Err(invalid_directive("Skill:", "days must be nonzero"));
    }

    {
        let unit = game.unit_mut(unit_id).expect("resolved unit exists");
        let new_days = unit.skills.get_days(skill_id).wrapping_add(days as u32);
        unit.skills.set_days(skill_id, new_days);
        adjust_skills(unit, globals, tables);
    }

    let level = {
        let unit = game.unit(unit_id).expect("resolved unit exists");
        real_skill_level(unit, skill_id, tables)
    };
    let shown = game
        .faction(faction_id)
        .expect("selected faction exists")
        .skills
        .get_days(skill_id);
    if level > i32::try_from(shown).unwrap_or(i32::MAX) {
        let shown_level = u32::try_from(level)
            .map_err(|_| invalid_directive("Skill:", "skill level is out of range"))?;
        let faction = game
            .faction_mut(faction_id)
            .expect("selected faction exists");
        faction.skills.set_days(skill_id, shown_level);
        faction.shows.push(FactionSkillShow {
            skill_type: skill_id,
            level,
        });
    }

    let skill = &tables.skills.skills[skill_id];
    let skill_name = skill.name.unwrap_or("unknown");
    let abbreviation = skill.abbreviation.unwrap_or("????");
    let gm_alias = {
        let unit = game.unit_mut(unit_id).expect("resolved unit exists");
        if skill.flags & skill_flags::MAGIC != 0 {
            unit.unit_type = UnitType::Mage;
        }
        if skill.flags & skill_flags::APPRENTICE != 0 && unit.unit_type == UnitType::Normal {
            unit.unit_type = UnitType::Apprentice;
        }
        unit.gm_alias
    };
    if gm_alias == 0 {
        unit_event(
            game,
            faction_id,
            unit_id,
            format!("Is taught {days} days of {skill_name} [{abbreviation}] by the gods."),
        );
    }
    Ok(())
}

fn player_unit(
    game: &Game,
    faction_id: FactionId,
    unit_tag: &str,
    directive: &str,
) -> Result<UnitId, PlayersError> {
    let unit_id = find_player_unit(game, faction_id, unit_tag)
        .ok_or_else(|| invalid_directive(directive, "unit was not found"))?;
    let unit = game.unit(unit_id).expect("resolved unit exists");
    if unit.faction != Some(faction_id) {
        return Err(invalid_directive(
            directive,
            "unit does not belong to the selected faction",
        ));
    }
    Ok(unit_id)
}

fn find_player_unit(game: &Game, faction_id: FactionId, unit_tag: &str) -> Option<UnitId> {
    let (unit_number, gm_alias) = if let Some(alias) = unit_tag.strip_prefix("gm") {
        (None, Some(value(alias)))
    } else {
        (Some(value(unit_tag)), None)
    };
    for region_id in game.region_ids() {
        let Some(region) = game.region(*region_id) else {
            continue;
        };
        for object_id in region.object_ids() {
            let Some(object) = game.object(*object_id) else {
                continue;
            };
            for unit_id in object.unit_ids() {
                let Some(unit) = game.unit(*unit_id) else {
                    continue;
                };
                if unit_number.is_some_and(|number| unit.num == number)
                    || gm_alias.is_some_and(|alias| {
                        unit.faction == Some(faction_id) && unit.gm_alias == alias
                    })
                {
                    return Some(*unit_id);
                }
            }
        }
    }
    None
}

pub(crate) fn discover_item(
    game: &mut Game,
    faction_id: FactionId,
    item_type: ItemId,
    item_index: usize,
    force: bool,
    full: bool,
    tables: &RulesetTables,
) {
    let faction = game
        .faction_mut(faction_id)
        .expect("selected faction exists");
    let seen = faction.items.get_num(item_type);
    let mut force = force;
    let mut full = full;
    if seen == 0 {
        faction.items.set_num(item_type, if full { 2 } else { 1 });
        force = true;
    } else if seen == 1 {
        if full {
            faction.items.set_num(item_type, 2);
        }
        force = true;
    } else {
        full = true;
    }
    if !force {
        return;
    }
    if tables.item_families.items[item_index].flags & ItemFlags::DISABLED == 0 {
        faction.item_shows.push(FactionItemShow { item_type, full });
    }
    if !full {
        return;
    }

    let Some(grant_skill) = tables.item_families.items[item_index]
        .grant_skill
        .as_deref()
    else {
        return;
    };
    let Some(skill_id) = skills::lookup_skill(&tables.skills.skills, grant_skill) else {
        return;
    };
    if tables.skills.skills[skill_id].flags & skill_flags::DISABLED != 0 {
        return;
    }
    let max_grant = tables.item_families.items[item_index].max_grant;
    for level in 1..=max_grant {
        let Ok(shown_level) = u32::try_from(level) else {
            continue;
        };
        if shown_level > faction.skills.get_days(skill_id) {
            faction.skills.set_days(skill_id, shown_level);
            faction.shows.push(FactionSkillShow {
                skill_type: skill_id,
                level,
            });
        }
    }
}

fn faction_event(faction: &mut Faction, event: String) {
    if !is_npc(faction) {
        faction.events.push(event);
    }
}

pub(crate) fn unit_event(game: &mut Game, faction_id: FactionId, unit_id: UnitId, event: String) {
    let unit = game.unit(unit_id).expect("resolved unit exists");
    let name = unit
        .name
        .clone()
        .unwrap_or_else(|| format!("Unit ({})", unit.num));
    faction_event(
        game.faction_mut(faction_id)
            .expect("selected faction exists"),
        format!("{name}: {event}"),
    );
}

fn item_string(
    tables: &RulesetTables,
    item_index: usize,
    amount: i32,
) -> Result<String, PlayersError> {
    let item = tables
        .item_families
        .items
        .get(item_index)
        .ok_or_else(|| invalid_directive("Item:", "item table index is out of range"))?;
    let name = if amount == 1 {
        item.name.as_deref()
    } else {
        item.plural.as_deref()
    }
    .ok_or_else(|| invalid_directive("Item:", "item name is missing from the ruleset"))?;
    let abbreviation = item
        .abbreviation
        .as_deref()
        .ok_or_else(|| invalid_directive("Item:", "item abbreviation is missing"))?;
    if amount == 1 {
        Ok(format!("{name} [{abbreviation}]"))
    } else {
        Ok(format!("{amount} {name} [{abbreviation}]"))
    }
}

fn adjust_skills(unit: &mut Unit, globals: &GameDefs, tables: &RulesetTables) {
    let men = unit_men(unit, tables);
    let leaders = unit.items.entries().iter().fold(0_i32, |total, entry| {
        let is_leader = tables
            .item_families
            .items
            .get(entry.item_type.index())
            .is_some_and(|item| item.item_type & ItemTypeFlags::LEADER != 0);
        if is_leader {
            total.wrapping_add(entry.num)
        } else {
            total
        }
    });
    if globals.skill_limit_nonleaders != 0 && leaders == 0 && unit.skills.entries().len() > 1 {
        let mut highest_skill = None;
        let mut highest_days = 0;
        for skill in unit.skills.entries() {
            if skill.days > highest_days {
                highest_days = skill.days;
                highest_skill = Some(skill.skill_type);
            }
        }
        if let Some(highest_skill) = highest_skill {
            let highest_is_magic =
                tables.skills.skills[highest_skill].flags & skill_flags::MAGIC != 0;
            let removed: Vec<_> = unit
                .skills
                .entries()
                .iter()
                .filter(|skill| skill.skill_type != highest_skill)
                .filter(|skill| {
                    let is_magic =
                        tables.skills.skills[skill.skill_type].flags & skill_flags::MAGIC != 0;
                    !(highest_is_magic && is_magic)
                        && !(globals.required_experience != 0 && skill.exp > 0)
                })
                .map(|skill| skill.skill_type)
                .collect();
            for skill in removed {
                unit.skills.remove(skill);
            }
        }
    }

    let skill_ids: Vec<_> = unit
        .skills
        .entries()
        .iter()
        .map(|skill| skill.skill_type)
        .collect();
    for skill_id in skill_ids {
        let max = skill_max(unit, skill_id, globals, tables);
        if real_skill_level(unit, skill_id, tables) >= max {
            let days = days_by_level(max).wrapping_mul(men);
            unit.skills.set_days(skill_id, days as u32);
        }
    }
}

pub(crate) fn unit_men(unit: &Unit, tables: &RulesetTables) -> i32 {
    unit.items.entries().iter().fold(0_i32, |total, entry| {
        let is_man = tables
            .item_families
            .items
            .get(entry.item_type.index())
            .is_some_and(|item| item.item_type & ItemTypeFlags::MAN != 0);
        if is_man {
            total.wrapping_add(entry.num)
        } else {
            total
        }
    })
}

pub(crate) fn real_skill_level(unit: &Unit, skill_id: usize, tables: &RulesetTables) -> i32 {
    let men = unit_men(unit, tables);
    if men == 0 {
        return 0;
    }
    let days = unit.skills.get_days(skill_id) as i32 / men;
    let mut threshold = 30;
    let mut level = 0;
    let mut days = days;
    while days >= threshold {
        level += 1;
        days -= threshold;
        threshold += 30;
    }
    level
}

fn skill_max(unit: &Unit, skill_id: usize, globals: &GameDefs, tables: &RulesetTables) -> i32 {
    let Some(skill) = tables.skills.skills.get(skill_id) else {
        return 0;
    };
    if skill.flags & skill_flags::DISABLED != 0 {
        return 0;
    }
    let mut max = 0;
    for entry in unit.items.entries() {
        let Some(item) = tables.item_families.items.get(entry.item_type.index()) else {
            continue;
        };
        if item.flags & ItemFlags::DISABLED != 0 || item.item_type & ItemTypeFlags::MAN == 0 {
            continue;
        }
        let Some(abbreviation) = item.abbreviation.as_deref() else {
            continue;
        };
        let Some(race) = tables.item_families.races.iter().find(|race| {
            race.abbreviation
                .as_deref()
                .is_some_and(|abbr| is_equal(abbr, abbreviation))
        }) else {
            continue;
        };
        let skill_max = if globals.mage_nonleaders == 0
            && skill.flags & skill_flags::MAGIC != 0
            && item.item_type & ItemTypeFlags::LEADER == 0
        {
            0
        } else if race.skills.iter().flatten().any(|race_skill| {
            skill
                .abbreviation
                .is_some_and(|abbreviation| is_equal(abbreviation, race_skill))
                || skill.flags & skill_flags::MAGIC != 0 && is_equal("MANI", race_skill)
        }) {
            race.special_level
        } else {
            race.default_level
        };
        if (max == 0 && skill_max > max) || skill_max < max {
            max = skill_max;
        }
    }
    max
}

fn days_by_level(level: i32) -> i32 {
    (1..=level).fold(0_i32, |days, level| days.wrapping_add(level * 30))
}

fn next_line<R: std::io::BufRead>(
    input: &mut TextInput<R>,
) -> Result<Option<String>, PlayersError> {
    input
        .get_order_line()
        .map_err(|error| PlayersError::Input(error.to_string()))
}

fn compatible_version(version: i32) -> bool {
    let version = version as u32;
    let current = CURRENT_ATL_VER.packed();
    ((version >> 16) & 0xff) == ((current >> 16) & 0xff)
        && ((version >> 8) & 0xff) == ((current >> 8) & 0xff)
        && (version & 0xff) <= (current & 0xff)
}

fn parse_template(template: &str) -> Option<i32> {
    if is_equal(template, "off") {
        Some(TEMPLATE_OFF)
    } else if is_equal(template, "short") {
        Some(TEMPLATE_SHORT)
    } else if is_equal(template, "long") {
        Some(TEMPLATE_LONG)
    } else if is_equal(template, "map") {
        Some(TEMPLATE_MAP)
    } else {
        None
    }
}

fn required_value(directive: &str, remainder: &str) -> Result<i32, PlayersError> {
    let mut remainder = remainder.to_owned();
    next_value(&mut remainder, directive)
}

fn next_token(remainder: &mut String, directive: &str) -> Result<String, PlayersError> {
    get_token(remainder).ok_or_else(|| invalid_directive(directive, "missing value"))
}

fn next_value(remainder: &mut String, directive: &str) -> Result<i32, PlayersError> {
    next_token(remainder, directive).map(|token| value(&token))
}

fn invalid_directive(directive: &str, reason: &'static str) -> PlayersError {
    PlayersError::InvalidDirective {
        directive: directive.to_owned(),
        reason,
    }
}

fn required_faction_field<'a>(
    faction: &Faction,
    value: Option<&'a str>,
    field: &'static str,
) -> Result<&'a str, PlayersError> {
    value.ok_or(PlayersError::MissingFactionField {
        faction_num: faction.num,
        field,
    })
}

fn is_npc(faction: &Faction) -> bool {
    faction.faction_types.iter().any(|(faction_type, value)| {
        (*value == -1) && (is_equal(faction_type, "war") || is_equal(faction_type, "martial"))
    })
}
