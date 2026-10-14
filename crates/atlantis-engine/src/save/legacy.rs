use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::io::Cursor;

use crate::defs::{
    objects::{lookup_object, O_FLEET},
    skills::lookup_skill,
    ItemFlags, ItemId, RulesetTables,
};
use crate::model::{
    Attitude, Faction, FactionId, Game, GuardMode, Item, ItemList, Market, MarketType, Object,
    Production, Quest, Region, RegionArray, RegionArrayError, RegionId, RegionLevelType, Skill,
    SkillList, TownInfo, Unit, UnitType,
};
use crate::rng::Rng;
use crate::ruleset::Ruleset;
use crate::text::{is_equal, TextInput};
use crate::version::{AtlVersion, CURRENT_ATL_VER};

const INITIAL_IO_SEED: i32 = 1783;

pub struct LoadedGame {
    pub game: Game,
    pub ruleset: &'static dyn Ruleset,
    pub seed: i32,
    pub rng: Rng,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SaveError {
    UnexpectedEof {
        field: &'static str,
        offset: usize,
    },
    InvalidInteger {
        field: &'static str,
        token: String,
        offset: usize,
    },
    InvalidCount {
        field: &'static str,
        count: i32,
    },
    InvalidValue {
        field: &'static str,
        value: String,
    },
    Input {
        field: &'static str,
        message: String,
    },
    InvalidSymbol {
        field: &'static str,
        symbol: String,
    },
    UnsupportedRuleset {
        name: String,
    },
    IncompatibleRuleset {
        saved: String,
        expected: &'static str,
    },
    IncompatibleEngineVersion {
        saved: u32,
        current: u32,
    },
    NewerEnginePatch {
        saved: u32,
        current: u32,
    },
    RulesetUpgradeRejected {
        saved: u32,
        current: u32,
    },
    RegionArray(RegionArrayError),
    Model(crate::model::ModelError),
}

impl fmt::Display for SaveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEof { field, offset } => {
                write!(
                    formatter,
                    "unexpected end of save while reading {field} at byte {offset}"
                )
            }
            Self::InvalidInteger {
                field,
                token,
                offset,
            } => write!(
                formatter,
                "invalid integer {token:?} for {field} at byte {offset}"
            ),
            Self::InvalidCount { field, count } => {
                write!(formatter, "invalid negative count {count} for {field}")
            }
            Self::InvalidValue { field, value } => {
                write!(formatter, "invalid value {value:?} for {field}")
            }
            Self::Input { field, message } => {
                write!(formatter, "could not read {field}: {message}")
            }
            Self::InvalidSymbol { field, symbol } => {
                write!(formatter, "unknown {field} symbol {symbol:?}")
            }
            Self::UnsupportedRuleset { name } => {
                write!(formatter, "unsupported save ruleset {name:?}")
            }
            Self::IncompatibleRuleset { saved, expected } => write!(
                formatter,
                "incompatible ruleset {saved:?}; expected {expected:?}"
            ),
            Self::IncompatibleEngineVersion { saved, current } => write!(
                formatter,
                "incompatible engine version {saved}; current version is {current}"
            ),
            Self::NewerEnginePatch { saved, current } => write!(
                formatter,
                "save engine patch {saved} is newer than current version {current}"
            ),
            Self::RulesetUpgradeRejected { saved, current } => write!(
                formatter,
                "cannot upgrade ruleset version {saved} to {current}"
            ),
            Self::RegionArray(error) => error.fmt(formatter),
            Self::Model(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for SaveError {}

/// C++: `game.cpp` `Game::OpenGame`; reads a legacy positional save from memory.
pub fn load_legacy(input: &str) -> Result<LoadedGame, SaveError> {
    let mut rng = Rng::new(INITIAL_IO_SEED);
    let mut reader = Reader::new(input);
    let header = read_header(&mut reader)?;
    rng.seed_random(header.seed);

    let tables = header.ruleset.tables();
    let defs = header.ruleset.game_defs();
    let mut game = Game::default();
    game.faction_seq = header.faction_seq;
    game.unit_seq = header.unit_seq;
    game.ship_seq = header.ship_seq;
    game.year = header.year;
    game.month = header.month;

    let factions = read_factions(
        &mut reader,
        &mut game,
        &tables,
        defs.required_experience != 0,
        header.ruleset.faction_types(),
        header.faction_count,
    )?;
    game.guard_faction = factions.get(&header.guard_faction_num).copied();
    game.monster_faction = factions.get(&header.monster_faction_num).copied();

    read_regions(&mut reader, &mut game, &tables, defs, &factions)?;
    game.quests = read_quests(&mut reader, &tables)?;
    game.setup_unit_nums().map_err(SaveError::Model)?;

    Ok(LoadedGame {
        game,
        ruleset: header.ruleset,
        seed: header.seed,
        rng,
    })
}

struct Header {
    ruleset: &'static dyn Ruleset,
    year: i32,
    month: i32,
    seed: i32,
    faction_seq: i32,
    unit_seq: u32,
    ship_seq: i32,
    guard_faction_num: i32,
    monster_faction_num: i32,
    faction_count: usize,
}

/// C++: `game.cpp` `Game::OpenGame`; applies the original engine and ruleset version gates.
fn read_header(reader: &mut Reader<'_>) -> Result<Header, SaveError> {
    let signature = reader.get_str("save signature")?;
    let signature_token = signature.split_whitespace().next().unwrap_or_default();
    if !is_equal(signature_token, "atlantis_game") {
        return Err(SaveError::InvalidValue {
            field: "save signature",
            value: signature,
        });
    }

    let engine_version = read_packed_version(reader, "engine version")?;
    let current_engine_version = CURRENT_ATL_VER.packed();
    let saved_engine = unpack_version(engine_version);
    if saved_engine.major() != CURRENT_ATL_VER.major()
        || saved_engine.minor() != CURRENT_ATL_VER.minor()
    {
        return Err(SaveError::IncompatibleEngineVersion {
            saved: engine_version,
            current: current_engine_version,
        });
    }
    if saved_engine.patch() > CURRENT_ATL_VER.patch() {
        return Err(SaveError::NewerEnginePatch {
            saved: engine_version,
            current: current_engine_version,
        });
    }

    let ruleset_name = reader.get_str("ruleset name")?;
    let ruleset =
        <dyn Ruleset>::from_name(&ruleset_name).ok_or_else(|| SaveError::UnsupportedRuleset {
            name: ruleset_name.clone(),
        })?;
    if !is_equal(&ruleset_name, ruleset.name()) {
        return Err(SaveError::IncompatibleRuleset {
            saved: ruleset_name,
            expected: ruleset.name(),
        });
    }

    let ruleset_version = read_packed_version(reader, "ruleset version")?;
    check_ruleset_version(ruleset_version, ruleset.game_defs().ruleset_version)?;

    let year = reader.get_int("year")?;
    let month = reader.get_int("month")?;
    let seed = reader.get_int("random seed")?;
    let faction_seq = reader.get_int("faction sequence")?;
    let unit_seq =
        u32::try_from(reader.get_int("unit sequence")?).map_err(|_| SaveError::InvalidValue {
            field: "unit sequence",
            value: "negative value".to_owned(),
        })?;
    let ship_seq = reader.get_int("ship sequence")?;
    let guard_faction_num = reader.get_int("guard faction")?;
    let monster_faction_num = reader.get_int("monster faction")?;
    let faction_count = reader.get_count("faction count")?;

    Ok(Header {
        ruleset,
        year,
        month,
        seed,
        faction_seq,
        unit_seq,
        ship_seq,
        guard_faction_num,
        monster_faction_num,
        faction_count,
    })
}

fn read_packed_version(reader: &mut Reader<'_>, field: &'static str) -> Result<u32, SaveError> {
    u32::try_from(reader.get_int(field)?).map_err(|_| SaveError::InvalidValue {
        field,
        value: "negative version".to_owned(),
    })
}

fn unpack_version(packed: u32) -> AtlVersion {
    AtlVersion::new(
        ((packed >> 16) & 0xff) as u8,
        ((packed >> 8) & 0xff) as u8,
        (packed & 0xff) as u8,
    )
}

/// C++: `game.cpp` `Game::OpenGame`; minor and patch upgrades are accepted without migration.
fn check_ruleset_version(saved: u32, current: AtlVersion) -> Result<(), SaveError> {
    let saved_version = unpack_version(saved);
    let current_packed = current.packed();
    if saved_version.major() < current.major() {
        return Err(SaveError::RulesetUpgradeRejected {
            saved,
            current: current_packed,
        });
    }
    // The original rejects major upgrades but accepts minor and patch upgrades.
    Ok(())
}

/// C++: `faction.cpp` `Faction::Readin`; self-attitudes are discarded.
fn read_factions(
    reader: &mut Reader<'_>,
    game: &mut Game,
    tables: &RulesetTables,
    required_experience: bool,
    faction_types: &[&str],
    faction_count: usize,
) -> Result<BTreeMap<i32, FactionId>, SaveError> {
    let mut faction_ids = BTreeMap::new();
    for _ in 0..faction_count {
        let mut faction = Faction {
            num: reader.get_int("faction number")?,
            faction_types: faction_types
                .iter()
                .map(|name| Ok(((*name).to_owned(), reader.get_int("faction type")?)))
                .collect::<Result<Vec<_>, SaveError>>()?,
            last_change: reader.get_int("faction last change")?,
            last_orders: reader.get_int("faction last orders")?,
            unclaimed: reader.get_int("faction unclaimed")?,
            name: Some(reader.get_str("faction name")?),
            address: Some(reader.get_str("faction address")?),
            password: Some(reader.get_str("faction password")?),
            times: reader.get_int("faction times")?,
            show_unit_attitudes: reader.get_int("show unit attitudes")?,
            template_format: reader.get_int("template format")?,
            skills: read_skill_list(reader, tables, required_experience)?,
            items: read_item_list(reader, tables)?,
            default_attitude: reader.get_int("default attitude")?,
            ..Faction::default()
        };
        let attitude_count = reader.get_count("attitude count")?;
        for _ in 0..attitude_count {
            let attitude = Attitude {
                faction_num: reader.get_int("attitude faction")?,
                attitude: reader.get_int("attitude value")?,
            };
            if attitude.faction_num != faction.num {
                faction.attitudes.push(attitude);
            }
        }
        let number = faction.num;
        let id = game.add_faction(faction);
        faction_ids.entry(number).or_insert(id);
    }
    Ok(faction_ids)
}

/// C++: `items.cpp` `ItemList::Readin`; unknown, nonpositive, and disabled items are dropped.
fn read_item_list(reader: &mut Reader<'_>, tables: &RulesetTables) -> Result<ItemList, SaveError> {
    let count = reader.get_count("item count")?;
    let mut items = ItemList::default();
    for _ in 0..count {
        let (number, symbol) = read_item_record(reader, "item")?;
        let Some(item_type) = tables.item_families.lookup_item(&symbol) else {
            continue;
        };
        if number < 1 || tables.item_families.items[item_type].flags & ItemFlags::DISABLED != 0 {
            continue;
        }
        let item_type = ItemId::from_index(item_type).ok_or_else(|| SaveError::InvalidValue {
            field: "item table index",
            value: item_type.to_string(),
        })?;
        items.push(Item {
            item_type,
            num: number,
            selling: 0,
            checked: false,
        });
    }
    Ok(items)
}

/// C++: `skills.cpp` `SkillList::Readin`; saved experience is conditional on the ruleset.
fn read_skill_list(
    reader: &mut Reader<'_>,
    tables: &RulesetTables,
    required_experience: bool,
) -> Result<SkillList, SaveError> {
    let count = reader.get_count("skill count")?;
    let mut skills = SkillList::default();
    for _ in 0..count {
        let line = reader.get_str("skill record")?;
        let mut tokens = line.split_whitespace();
        let abbreviation = tokens.next().ok_or_else(|| SaveError::InvalidValue {
            field: "skill record",
            value: line.clone(),
        })?;
        let days = parse_line_integer(tokens.next(), "skill days", &line)?;
        let exp = if required_experience {
            parse_line_integer(tokens.next(), "skill experience", &line)?
        } else {
            0
        };
        let days = u32::try_from(days).map_err(|_| SaveError::InvalidValue {
            field: "skill days",
            value: line.clone(),
        })?;
        let exp = u32::try_from(exp).map_err(|_| SaveError::InvalidValue {
            field: "skill experience",
            value: line.clone(),
        })?;
        if days == 0 && exp == 0 {
            continue;
        }
        let skill_type = lookup_skill(&tables.skills.skills, abbreviation).ok_or_else(|| {
            SaveError::InvalidSymbol {
                field: "skill",
                symbol: abbreviation.to_owned(),
            }
        })?;
        skills.push(Skill {
            skill_type,
            days,
            exp,
        });
    }
    Ok(skills)
}

fn parse_line_integer(
    token: Option<&str>,
    field: &'static str,
    line: &str,
) -> Result<i32, SaveError> {
    token
        .ok_or_else(|| SaveError::InvalidValue {
            field,
            value: line.to_owned(),
        })?
        .parse()
        .map_err(|_| SaveError::InvalidValue {
            field,
            value: line.to_owned(),
        })
}

/// C++: `items.cpp` `Item::Readin`; `lookup_item` preserves the illusion `i` prefix.
fn read_item_record(
    reader: &mut Reader<'_>,
    field: &'static str,
) -> Result<(i32, String), SaveError> {
    let line = reader.get_str(field)?;
    let mut tokens = line.split_whitespace();
    let number = parse_line_integer(tokens.next(), "item number", &line)?;
    let symbol = tokens.next().ok_or_else(|| SaveError::InvalidValue {
        field,
        value: line.clone(),
    })?;
    Ok((number, symbol.to_owned()))
}

/// C++: `aregion.cpp` `ARegionList::ReadRegions`; links neighbors after all regions load.
fn read_regions(
    reader: &mut Reader<'_>,
    game: &mut Game,
    tables: &RulesetTables,
    defs: &crate::defs::GameDefs,
    factions: &BTreeMap<i32, FactionId>,
) -> Result<(), SaveError> {
    let region_count = reader.get_count("region count")?;
    let level_count = reader.get_count("region level count")?;
    for _ in 0..level_count {
        let x = reader.get_int("region array width")?;
        let y = reader.get_int("region array height")?;
        let name = reader.get_str("region array name")?;
        let raw_level_type = reader.get_int("region level type")?;
        let level_type = match raw_level_type {
            0 => RegionLevelType::Nexus,
            1 => RegionLevelType::Surface,
            2 => RegionLevelType::Underworld,
            3 => RegionLevelType::Underdeep,
            _ => {
                return Err(SaveError::InvalidValue {
                    field: "region level type",
                    value: raw_level_type.to_string(),
                })
            }
        };
        let mut array = RegionArray::new(x, y, level_type).map_err(SaveError::RegionArray)?;
        if !is_equal(&name, "none") {
            array.name = name;
        }
        game.add_region_array(array);
    }
    game.number_of_gates = reader.get_int("region gate count")?;

    let mut region_ids = Vec::with_capacity(region_count);
    for _ in 0..region_count {
        let (region, object_count) = read_region(reader, tables, defs)?;
        let region_id = game.add_region(region);
        let (zloc, xloc, yloc) = {
            let region = game.region(region_id).expect("new region exists");
            (region.zloc, region.xloc, region.yloc)
        };
        let array = usize::try_from(zloc)
            .ok()
            .and_then(|index| game.region_arrays.get_mut(index))
            .ok_or_else(|| SaveError::InvalidValue {
                field: "region level index",
                value: zloc.to_string(),
            })?;
        array
            .set_region(xloc, yloc, region_id)
            .map_err(SaveError::RegionArray)?;
        region_ids.push(region_id);

        for _ in 0..object_count {
            read_object(reader, game, region_id, tables, defs, factions)?;
        }
    }

    let _neighbors_marker = reader.get_str("neighbors marker")?;
    for region_id in region_ids {
        let mut neighbors = [None; 6];
        for neighbor in &mut neighbors {
            let number = reader.get_int("neighbor region number")?;
            if number != -1 {
                *neighbor = game.get_region(number);
            }
        }
        game.region_mut(region_id)
            .expect("loaded region exists")
            .neighbors = neighbors;
    }
    Ok(())
}

/// C++: `aregion.cpp` `ARegion::Readin`; town development is recomputed from saved region data.
fn read_region(
    reader: &mut Reader<'_>,
    tables: &RulesetTables,
    defs: &crate::defs::GameDefs,
) -> Result<(Region, usize), SaveError> {
    let mut region = Region::default();
    region.name = reader.get_str("region name")?;
    region.num = reader.get_int("region number")?;
    let terrain_name = reader.get_str("region terrain")?;
    region.terrain_type = if is_equal(&terrain_name, "NO_TERRAIN") {
        None
    } else {
        tables
            .terrains
            .iter()
            .position(|terrain| is_equal(terrain.terrain_type, &terrain_name))
    };
    region.building_seq = reader.get_int("building sequence")?;
    region.gate = reader.get_int("region gate")?;
    if region.gate > 0 {
        region.gate_month = reader.get_int("gate month")?;
    }
    let race = reader.get_str("region race")?;
    region.race = tables
        .item_families
        .lookup_item(&race)
        .and_then(|item| i32::try_from(item).ok())
        .unwrap_or(-1);
    region.population = reader.get_int("region population")?;
    region.base_population = reader.get_int("region base population")?;
    region.wages = reader.get_int("region wages")?;
    region.max_wages = reader.get_int("region maximum wages")?;
    region.wealth = reader.get_int("region wealth")?;
    region.elevation = reader.get_int("region elevation")?;
    region.humidity = reader.get_int("region humidity")?;
    region.temperature = reader.get_int("region temperature")?;
    region.vegetation = reader.get_int("region vegetation")?;
    region.culture = reader.get_int("region culture")?;
    region.habitat = reader.get_int("region habitat")?;
    region.development = reader.get_int("region development")?;
    region.max_development = reader.get_int("region maximum development")?;
    if reader.get_int("town flag")? != 0 {
        region.town = Some(TownInfo {
            name: Some(reader.get_str("town name")?),
            population: reader.get_int("town population")?,
            habitat: reader.get_int("town habitat")?,
            ..TownInfo::default()
        });
    }
    region.xloc = reader.get_int("region x coordinate")?;
    region.yloc = reader.get_int("region y coordinate")?;
    region.zloc = reader.get_int("region level")?;
    region.visited = reader.get_int("region visited")?;

    let production_count = reader.get_count("production count")?;
    region.products = read_productions(reader, tables, production_count)?;
    let market_count = reader.get_count("market count")?;
    region.markets = read_markets(reader, tables, market_count)?;
    let object_count = reader.get_count("object count")?;
    region.building_seq = 1;
    if let Some(town) = &mut region.town {
        let terrain = region
            .terrain_type
            .and_then(|terrain| tables.terrains.get(terrain))
            .ok_or_else(|| SaveError::InvalidValue {
                field: "town terrain",
                value: region
                    .terrain_type
                    .map_or_else(|| "NO_TERRAIN".to_owned(), |terrain| terrain.to_string()),
            })?;
        let mut level = 1;
        let mut base_development = 1;
        let mut previous = 0;
        while level <= terrain.wages {
            previous += 1;
            base_development += 1;
            if previous > level {
                level += 1;
                previous = 0;
            }
        }
        base_development = (defs.maintenance_cost + base_development) / 2;
        town.development = (region.development - base_development).clamp(0, 100);
    }
    Ok((region, object_count))
}

/// C++: `production.cpp` `ProductionList::Readin`; silver alone stores its production skill.
fn read_productions(
    reader: &mut Reader<'_>,
    tables: &RulesetTables,
    count: usize,
) -> Result<Vec<Production>, SaveError> {
    let mut productions = Vec::with_capacity(count);
    for _ in 0..count {
        let item_symbol = reader.get_str("production item")?;
        let item_type = tables
            .item_families
            .lookup_item(&item_symbol)
            .and_then(|item| i32::try_from(item).ok())
            .unwrap_or(-1);
        let amount = reader.get_int("production amount")?;
        let base_amount = reader.get_int("production base amount")?;
        let skill = if item_type == ItemId::Silver as i32 {
            lookup_skill(
                &tables.skills.skills,
                &reader.get_str("silver production skill")?,
            )
        } else {
            usize::try_from(item_type)
                .ok()
                .and_then(|index| tables.item_families.items.get(index))
                .and_then(|item| item.production_skill.as_deref())
                .and_then(|skill| lookup_skill(&tables.skills.skills, skill))
        }
        .and_then(|skill| i32::try_from(skill).ok())
        .unwrap_or(-1);
        let productivity = reader.get_int("production productivity")?;
        productions.push(Production {
            item_type,
            amount,
            base_amount,
            skill,
            productivity,
            activity: 0,
        });
    }
    Ok(productions)
}

/// C++: `market.cpp` `MarketList::Readin`.
fn read_markets(
    reader: &mut Reader<'_>,
    tables: &RulesetTables,
    count: usize,
) -> Result<Vec<Market>, SaveError> {
    let mut markets = Vec::with_capacity(count);
    for _ in 0..count {
        let raw_type = reader.get_int("market type")?;
        let market_type = match raw_type {
            0 => MarketType::Buy,
            1 => MarketType::Sell,
            _ => {
                return Err(SaveError::InvalidValue {
                    field: "market type",
                    value: raw_type.to_string(),
                })
            }
        };
        let item_symbol = reader.get_str("market item")?;
        let item = tables
            .item_families
            .lookup_item(&item_symbol)
            .and_then(|item| i32::try_from(item).ok())
            .unwrap_or(-1);
        markets.push(Market {
            market_type,
            item,
            price: reader.get_int("market price")?,
            amount: reader.get_int("market amount")?,
            min_population: reader.get_int("market minimum population")?,
            max_population: reader.get_int("market maximum population")?,
            min_amount: reader.get_int("market minimum amount")?,
            max_amount: reader.get_int("market maximum amount")?,
            base_price: reader.get_int("market base price")?,
            activity: 0,
        });
    }
    Ok(markets)
}

/// C++: `object.cpp` `Object::Readin`; prevdir is cleared when trivial portage is allowed.
fn read_object(
    reader: &mut Reader<'_>,
    game: &mut Game,
    region_id: RegionId,
    tables: &RulesetTables,
    defs: &crate::defs::GameDefs,
    factions: &BTreeMap<i32, FactionId>,
) -> Result<(), SaveError> {
    let num = reader.get_int("object number")?;
    let object_name = reader.get_str("object type")?;
    let object_type =
        lookup_object(&object_name, &tables.objects).ok_or(SaveError::InvalidSymbol {
            field: "object",
            symbol: object_name,
        })?;
    let incomplete = reader.get_int("object incomplete")?;
    let name = reader.get_str("object name")?;
    let description = reader.get_str("object description")?;
    let inner = reader.get_int("object inner")?;
    let mut prevdir = reader.get_int("object previous direction")?;
    if defs.allow_trivial_portage != 0 {
        prevdir = -1;
    }
    let runes = reader.get_int("object runes")?;
    let unit_count = reader.get_count("object unit count")?;
    let mut object = Object::new(defs.phased_move_offset % defs.max_speed);
    object.num = num;
    object.object_type = i32::try_from(object_type).map_err(|_| SaveError::InvalidValue {
        field: "object type",
        value: object_type.to_string(),
    })?;
    object.incomplete = incomplete;
    object.name = Some(name);
    object.describe = if is_equal(&description, "none") {
        None
    } else {
        Some(description)
    };
    object.inner = inner;
    object.prevdir = prevdir;
    object.runes = runes;
    object.mages = tables.objects[object_type].maximum_mages;
    let object_id = game
        .add_object(region_id, object)
        .map_err(SaveError::Model)?;

    for _ in 0..unit_count {
        let unit = read_unit(reader, tables, defs.required_experience != 0, factions)?;
        if let Some(faction_id) = unit.faction {
            let is_npc = game.faction(faction_id).is_some_and(|faction| {
                faction
                    .faction_types
                    .first()
                    .is_some_and(|(_, kind)| *kind == -1)
            });
            game.add_unit(object_id, unit).map_err(SaveError::Model)?;
            if !is_npc {
                game.region_mut(region_id)
                    .expect("containing region exists")
                    .visited = 1;
            }
        }
    }

    if object_type == O_FLEET {
        let ship_count = reader.get_count("fleet ship count")?;
        for _ in 0..ship_count {
            let (number, symbol) = read_item_record(reader, "fleet ship")?;
            if let Some(item_type) = tables.item_families.lookup_item(&symbol) {
                let item_type =
                    ItemId::from_index(item_type).ok_or_else(|| SaveError::InvalidValue {
                        field: "item table index",
                        value: item_type.to_string(),
                    })?;
                game.object_mut(object_id)
                    .expect("new object exists")
                    .ships
                    .set_num(item_type, number);
            }
        }
    }

    let region = game
        .region_mut(region_id)
        .expect("containing region exists");
    if num >= region.building_seq {
        region.building_seq = num.checked_add(1).ok_or_else(|| SaveError::InvalidValue {
            field: "building sequence",
            value: num.to_string(),
        })?;
    }
    Ok(())
}

/// C++: `unit.cpp` `Unit::Readin`; units without a saved faction are consumed but not attached.
fn read_unit(
    reader: &mut Reader<'_>,
    tables: &RulesetTables,
    required_experience: bool,
    factions: &BTreeMap<i32, FactionId>,
) -> Result<Unit, SaveError> {
    let name = reader.get_str("unit name")?;
    let description = reader.get_str("unit description")?;
    let num = reader.get_int("unit number")?;
    let raw_unit_type = reader.get_int("unit type")?;
    let unit_type = match raw_unit_type {
        0 => UnitType::Normal,
        1 => UnitType::Mage,
        2 => UnitType::Guard,
        3 => UnitType::WanderingMonster,
        4 => UnitType::GuardMage,
        5 => UnitType::Apprentice,
        _ => {
            return Err(SaveError::InvalidValue {
                field: "unit type",
                value: raw_unit_type.to_string(),
            })
        }
    };
    let faction_num = reader.get_int("unit faction")?;
    let faction = factions.get(&faction_num).copied();
    let raw_guard = reader.get_int("unit guard mode")?;
    let guard = normalize_guard_mode(raw_guard)?;
    let reveal = reader.get_int("unit reveal")?;
    let free = reader.get_int("unit free")?;
    let ready_item = lookup_item_number(tables, &reader.get_str("unit ready item")?);
    let mut ready_weapon = [-1; 4];
    let mut ready_armor = [-1; 4];
    for index in 0..4 {
        ready_weapon[index] = lookup_item_number(tables, &reader.get_str("unit ready weapon")?);
        ready_armor[index] = lookup_item_number(tables, &reader.get_str("unit ready armor")?);
    }
    let flags = reader.get_int("unit flags")?;
    let items = read_item_list(reader, tables)?;
    let skills = read_skill_list(reader, tables, required_experience)?;
    let combat = lookup_skill(&tables.skills.skills, &reader.get_str("unit combat skill")?)
        .and_then(|skill| i32::try_from(skill).ok())
        .unwrap_or(-1);
    let saved_movement = reader.get_int("saved unit movement")?;
    let saved_move_dir = reader.get_int("saved unit move direction")?;
    let visited_count = reader.get_count("unit visited count")?;
    let mut visited = BTreeSet::new();
    for _ in 0..visited_count {
        visited.insert(reader.get_str("unit visited entry")?);
    }

    Ok(Unit {
        name: Some(name),
        describe: {
            if is_equal(&description, "none") {
                None
            } else {
                Some(description)
            }
        },
        num,
        unit_type,
        faction,
        guard,
        reveal,
        free,
        ready_item,
        ready_weapon,
        ready_armor,
        flags,
        items,
        skills,
        combat,
        saved_movement,
        saved_move_dir,
        visited,
        ..Unit::default()
    })
}

/// C++: `unit.cpp` `Unit::Readin`; ADVANCE becomes NONE and SET becomes GUARD.
fn normalize_guard_mode(value: i32) -> Result<GuardMode, SaveError> {
    match value {
        0 | 4 => Ok(GuardMode::None),
        1 | 3 => Ok(GuardMode::Guard),
        2 => Ok(GuardMode::Avoid),
        _ => Err(SaveError::InvalidValue {
            field: "unit guard mode",
            value: value.to_string(),
        }),
    }
}

fn lookup_item_number(tables: &RulesetTables, symbol: &str) -> i32 {
    tables
        .item_families
        .lookup_item(symbol)
        .and_then(|item| i32::try_from(item).ok())
        .unwrap_or(-1)
}

/// C++: `quests.cpp` `QuestList::ReadQuests`; destinations use ordered set semantics.
fn read_quests(reader: &mut Reader<'_>, tables: &RulesetTables) -> Result<Vec<Quest>, SaveError> {
    let count = reader.get_count("quest count")?;
    let mut quests = Vec::with_capacity(count);
    for _ in 0..count {
        let mut quest = Quest {
            quest_type: reader.get_int("quest type")?,
            ..Quest::default()
        };
        match quest.quest_type {
            0 => quest.target = reader.get_int("quest target")?,
            1 => {
                (quest.objective_num, quest.objective_type) = read_objective(reader, tables)?;
                quest.region_num = reader.get_int("quest region")?;
            }
            2 => {
                quest.building = lookup_object_number(tables, &reader.get_str("quest building")?);
                quest.region_name = reader.get_str("quest region name")?;
            }
            3 => {
                quest.building = lookup_object_number(tables, &reader.get_str("quest building")?);
                read_destinations(reader, &mut quest.destinations)?;
            }
            5 => {
                quest.target = reader.get_int("quest target")?;
                quest.region_num = reader.get_int("quest region")?;
            }
            _ => {
                quest.target = reader.get_int("quest target")?;
                (quest.objective_num, quest.objective_type) = read_objective(reader, tables)?;
                quest.building = lookup_object_number(tables, &reader.get_str("quest building")?);
                quest.region_num = reader.get_int("quest region")?;
                quest.region_name = reader.get_str("quest region name")?;
                read_destinations(reader, &mut quest.destinations)?;
            }
        }
        let reward_count = reader.get_count("quest reward count")?;
        for _ in 0..reward_count {
            let (number, symbol) = read_item_record(reader, "quest reward")?;
            let item_type = tables.item_families.lookup_item(&symbol).ok_or_else(|| {
                SaveError::InvalidSymbol {
                    field: "quest reward item",
                    symbol: symbol.clone(),
                }
            })?;
            let item_type =
                ItemId::from_index(item_type).ok_or_else(|| SaveError::InvalidValue {
                    field: "item table index",
                    value: item_type.to_string(),
                })?;
            quest.rewards.push(Item {
                item_type,
                num: number,
                selling: 0,
                checked: false,
            });
        }
        quests.push(quest);
    }
    Ok(quests)
}

/// C++: `items.cpp` `Item::Readin`, used for the quest objective record.
fn read_objective(
    reader: &mut Reader<'_>,
    tables: &RulesetTables,
) -> Result<(i32, i32), SaveError> {
    let (number, symbol) = read_item_record(reader, "quest objective")?;
    let item_type = tables
        .item_families
        .lookup_item(&symbol)
        .and_then(|item| i32::try_from(item).ok())
        .unwrap_or(-1);
    Ok((number, item_type))
}

/// C++: `quests.cpp` `QuestList::ReadQuests`; duplicate destinations are discarded.
fn read_destinations(
    reader: &mut Reader<'_>,
    destinations: &mut BTreeSet<String>,
) -> Result<(), SaveError> {
    let count = reader.get_count("quest destination count")?;
    for _ in 0..count {
        destinations.insert(reader.get_str("quest destination")?);
    }
    Ok(())
}

fn lookup_object_number(tables: &RulesetTables, symbol: &str) -> i32 {
    lookup_object(symbol, &tables.objects)
        .and_then(|object| i32::try_from(object).ok())
        .unwrap_or(-1)
}

struct Reader<'a> {
    input: TextInput<Cursor<&'a [u8]>>,
}

impl<'a> Reader<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input: TextInput::new(Cursor::new(input.as_bytes())),
        }
    }

    fn get_int(&mut self, field: &'static str) -> Result<i32, SaveError> {
        let offset = self.offset(field)?;
        self.input
            .get_int()
            .map_err(|error| SaveError::Input {
                field,
                message: error.to_string(),
            })?
            .ok_or_else(|| SaveError::InvalidInteger {
                field,
                token: "end of input or invalid integer".to_owned(),
                offset,
            })
    }

    fn get_count(&mut self, field: &'static str) -> Result<usize, SaveError> {
        let count = self.get_int(field)?;
        usize::try_from(count).map_err(|_| SaveError::InvalidCount { field, count })
    }

    fn get_str(&mut self, field: &'static str) -> Result<String, SaveError> {
        let offset = self.offset(field)?;
        self.input
            .get_str()
            .map_err(|error| SaveError::Input {
                field,
                message: error.to_string(),
            })?
            .ok_or(SaveError::UnexpectedEof { field, offset })
    }

    fn offset(&mut self, field: &'static str) -> Result<usize, SaveError> {
        self.input
            .position()
            .map(|offset| offset as usize)
            .map_err(|error| SaveError::Input {
                field,
                message: error.to_string(),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        normalize_guard_mode, read_header, read_item_list, read_quests, read_skill_list, Reader,
        SaveError,
    };
    use crate::defs::{ItemFlags, ItemId};
    use crate::model::GuardMode;
    use crate::ruleset::Ruleset;

    #[test]
    fn header_reading_selects_rulesets_and_applies_version_gates() {
        let mut reader = Reader::new(
            "atlantis_game\n328197\nWyreth\n131072\n1\n-1\n6456\n3\n140\n100\n1\n2\n0\n",
        );
        let header = read_header(&mut reader).expect("read header");
        assert_eq!(header.ruleset.name(), "Wyreth");
        assert_eq!(header.seed, 6456);
        assert_eq!(header.faction_count, 0);

        let mut reader = Reader::new("atlantis_game\n262661\n");
        assert!(matches!(
            read_header(&mut reader),
            Err(SaveError::IncompatibleEngineVersion { .. })
        ));

        let mut reader = Reader::new("atlantis_game\n328198\n");
        assert!(matches!(
            read_header(&mut reader),
            Err(SaveError::NewerEnginePatch { .. })
        ));

        let mut reader = Reader::new("atlantis_game\n328197\nUnknown\n");
        assert!(matches!(
            read_header(&mut reader),
            Err(SaveError::UnsupportedRuleset { .. })
        ));

        let mut reader = Reader::new("atlantis_game\n328197\nWyreth\n65536\n");
        assert!(matches!(
            read_header(&mut reader),
            Err(SaveError::RulesetUpgradeRejected { .. })
        ));
    }

    #[test]
    fn legacy_lists_filter_items_and_zero_skills_using_active_tables() {
        let ruleset = <dyn Ruleset>::from_name("Wyreth").expect("standard ruleset");
        let tables = ruleset.tables();
        let disabled_abbreviation = tables
            .item_families
            .items
            .iter()
            .find(|item| item.flags & ItemFlags::DISABLED != 0)
            .and_then(|item| item.abbreviation.as_deref())
            .expect("disabled item with an abbreviation");
        let item_input = format!("3\n5 SILV\n0 WOOD\n1 {disabled_abbreviation}\n");
        let mut reader = Reader::new(&item_input);
        let items = read_item_list(&mut reader, &tables).expect("read items");
        assert_eq!(items.entries().len(), 1);
        assert_eq!(items.entries()[0].item_type, ItemId::Silver);
        assert_eq!(items.entries()[0].num, 5);

        let mut reader = Reader::new("2\nMINI 0\nMINI 10\n");
        let skills = read_skill_list(&mut reader, &tables, false).expect("read skills");
        assert_eq!(skills.entries().len(), 1);
        assert_eq!(skills.entries()[0].days, 10);
    }

    #[test]
    fn legacy_quests_keep_type_specific_fields_and_sorted_destinations() {
        let ruleset = <dyn Ruleset>::from_name("Wyreth").expect("standard ruleset");
        let tables = ruleset.tables();
        let mut reader = Reader::new(
            "6\n\
             0\n42\n2\n2 SILV\n3 WOOD\n\
             1\n3 SILV\n7\n0\n\
             2\nTower\nSomewhere\n0\n\
             3\nTower\n2\nzeta\nalpha\n0\n\
             4\n12\n3 WOOD\nTower\n9\nNear\n2\nzeta\nalpha\n0\n\
             5\n44\n10\n0\n",
        );
        let quests = read_quests(&mut reader, &tables).expect("read quests");
        assert_eq!(quests.len(), 6);
        assert_eq!(quests[0].target, 42);
        assert_eq!(
            quests[0]
                .rewards
                .entries()
                .iter()
                .map(|reward| (reward.item_type, reward.num))
                .collect::<Vec<_>>(),
            [(ItemId::Silver, 2), (ItemId::Wood, 3)]
        );
        assert_eq!(
            (
                quests[1].objective_type,
                quests[1].objective_num,
                quests[1].region_num
            ),
            (ItemId::Silver as i32, 3, 7)
        );
        assert_eq!(quests[2].region_name, "Somewhere");
        assert_eq!(
            quests[3]
                .destinations
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            ["alpha", "zeta"]
        );
        assert_eq!(quests[4].quest_type, 4);
        assert_eq!(quests[4].target, 12);
        assert_eq!(quests[4].building, crate::defs::objects::O_TOWER as i32);
        assert_eq!(quests[4].region_name, "Near");
        assert_eq!(quests[4].destinations.len(), 2);
        assert_eq!((quests[5].target, quests[5].region_num), (44, 10));
    }

    #[test]
    fn guard_advance_and_set_values_are_normalized_when_loaded() {
        assert_eq!(normalize_guard_mode(4), Ok(GuardMode::None));
        assert_eq!(normalize_guard_mode(3), Ok(GuardMode::Guard));
    }
}
