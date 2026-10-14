use std::fmt;

use crate::defs::objects::{
    object_flags, O_DUMMY, O_FLEET, O_ROADN, O_ROADNE, O_ROADNW, O_ROADS, O_ROADSE, O_ROADSW,
};
use crate::defs::terrains::{R_LAKE, R_OCEAN};
use crate::defs::{ItemFlags, ItemId, ItemTypeFlags, TerrainId};
use crate::model::{
    FactionId, Game, GuardMode, Market, MarketType, ModelError, Object, ObjectId, Production,
    Region, RegionId, TownInfo, UnitId, UnitType,
};
use crate::save::LoadedGame;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EditError {
    Model(ModelError),
    MissingFaction(FactionId),
    InvalidObjectType(usize),
    DisabledObjectType(usize),
    InvalidTerrain(TerrainId),
    InvalidRace(ItemId),
    InvalidMarketItem(ItemId),
    InvalidObjectIndex(usize),
    InvalidMarketRange,
    MarketAmountOverflow,
    InvalidMarketPrice,
    MarketNotFound(ItemId),
    InvalidUnitItem(ItemId),
    InvalidSkill(usize),
    InvalidSkillDays,
    RegionHasNoDummy(RegionId),
    TownNotFound(RegionId),
    TownsDisabled,
    GatesDisabled,
    GateAlreadyExists(RegionId),
    GateNotFound(RegionId),
    GateStateInconsistent,
    InvalidEconomyParameters,
    DummyObjectCannotBeRenamed,
    EmptyName,
    InvalidMoveSpeed,
    SequenceOverflow,
}

impl fmt::Display for EditError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Model(error) => error.fmt(formatter),
            Self::MissingFaction(id) => write!(formatter, "faction {} does not exist", id.index()),
            Self::InvalidObjectType(object_type) => {
                write!(formatter, "object type {object_type} does not exist")
            }
            Self::DisabledObjectType(object_type) => {
                write!(formatter, "object type {object_type} is disabled")
            }
            Self::InvalidTerrain(terrain) => write!(formatter, "terrain {terrain} does not exist"),
            Self::InvalidRace(race) => {
                write!(formatter, "item {} is not an enabled race", race.index())
            }
            Self::InvalidMarketItem(item) => {
                write!(formatter, "item {} is not enabled", item.index())
            }
            Self::InvalidObjectIndex(index) => {
                write!(formatter, "region object {index} does not exist")
            }
            Self::InvalidMarketRange => {
                formatter.write_str("market minimum must be less than maximum")
            }
            Self::MarketAmountOverflow => {
                formatter.write_str("market amount arithmetic exceeds the supported range")
            }
            Self::InvalidMarketPrice => {
                formatter.write_str("market prices must be greater than zero")
            }
            Self::MarketNotFound(item) => {
                write!(formatter, "market for item {} does not exist", item.index())
            }
            Self::InvalidUnitItem(item) => {
                write!(formatter, "item {} is not enabled", item.index())
            }
            Self::InvalidSkill(skill) => {
                write!(formatter, "skill {skill} does not exist or is disabled")
            }
            Self::InvalidSkillDays => formatter.write_str("skill days must be non-negative"),
            Self::RegionHasNoDummy(region) => {
                write!(formatter, "region {} has no dummy object", region.index())
            }
            Self::TownNotFound(region) => {
                write!(formatter, "region {} has no town", region.index())
            }
            Self::TownsDisabled => formatter.write_str("towns are disabled by the ruleset"),
            Self::GatesDisabled => formatter.write_str("gates are disabled by the ruleset"),
            Self::GateAlreadyExists(region) => {
                write!(formatter, "region {} already has a gate", region.index())
            }
            Self::GateNotFound(region) => {
                write!(formatter, "region {} has no gate", region.index())
            }
            Self::GateStateInconsistent => {
                formatter.write_str("the game's gate count is inconsistent")
            }
            Self::InvalidEconomyParameters => {
                formatter.write_str("the ruleset has an invalid economy fraction")
            }
            Self::DummyObjectCannotBeRenamed => {
                formatter.write_str("the region dummy object cannot be renamed")
            }
            Self::EmptyName => formatter.write_str("name cannot be empty"),
            Self::InvalidMoveSpeed => formatter.write_str("ruleset max speed must be positive"),
            Self::SequenceOverflow => {
                formatter.write_str("object sequence exceeds supported range")
            }
        }
    }
}

impl std::error::Error for EditError {}

impl From<ModelError> for EditError {
    fn from(error: ModelError) -> Self {
        Self::Model(error)
    }
}

/// C++: `edit.cpp` `EditGameFindRegion`; coordinates use the region array's wrapping lookup.
pub fn find_region(game: &Game, x: i32, y: i32, z: i32) -> Option<RegionId> {
    game.get_region_at(x, y, z)
}

/// C++: `edit.cpp` `EditGameFindUnit`; unit numbers use the game number index.
pub fn find_unit(game: &Game, number: i32) -> Option<UnitId> {
    game.get_unit_id(number)
}

/// C++: `edit.cpp` `EditGameCreateUnit`; initializes a visible leader in the region dummy.
pub fn create_unit(
    loaded: &mut LoadedGame,
    faction_id: FactionId,
    region_id: RegionId,
) -> Result<UnitId, EditError> {
    if loaded.game.faction(faction_id).is_none() {
        return Err(EditError::MissingFaction(faction_id));
    }
    let dummy_id = loaded
        .game
        .get_dummy(region_id)
        .ok_or(EditError::Model(ModelError::MissingRegion(region_id)))?;
    let defs = loaded.ruleset.game_defs();
    if defs.max_speed <= 0 {
        return Err(EditError::InvalidMoveSpeed);
    }
    let movement_points = defs.phased_move_offset % defs.max_speed;
    let unit_id = loaded
        .game
        .get_new_unit(Some(faction_id), 0, movement_points)?;
    let unit = loaded
        .game
        .unit_mut(unit_id)
        .expect("new unit exists after allocation");
    unit.items.set_num(ItemId::Leaders, 1);
    unit.reveal = 2;
    loaded.game.attach_unit(dummy_id, unit_id)?;
    Ok(unit_id)
}

/// C++: `edit.cpp` `EditGameUnitMove`; moves a unit to a region's dummy object.
pub fn move_unit_to_region(
    loaded: &mut LoadedGame,
    unit_id: UnitId,
    region_id: RegionId,
) -> Result<(), EditError> {
    if loaded.game.region(region_id).is_none() {
        return Err(EditError::Model(ModelError::MissingRegion(region_id)));
    }
    let dummy_id = loaded
        .game
        .get_dummy(region_id)
        .ok_or(EditError::RegionHasNoDummy(region_id))?;
    loaded.game.attach_unit(dummy_id, unit_id)?;
    Ok(())
}

/// C++: `edit.cpp` `EditGameUnitItems`; zero removes the item and faction knowledge is recorded.
pub fn set_unit_item(
    loaded: &mut LoadedGame,
    unit_id: UnitId,
    item: ItemId,
    amount: i32,
) -> Result<(), EditError> {
    if !item_enabled(&loaded.ruleset.tables(), item.index()) {
        return Err(EditError::InvalidUnitItem(item));
    }
    let faction_id = loaded
        .game
        .unit(unit_id)
        .ok_or(EditError::Model(ModelError::MissingUnit(unit_id)))?
        .faction;
    loaded
        .game
        .unit_mut(unit_id)
        .expect("unit was checked before item edit")
        .items
        .set_num(item, amount);
    if let Some(faction_id) = faction_id {
        if let Some(faction) = loaded.game.faction_mut(faction_id) {
            faction.items.set_num(item, 1);
        }
    }
    Ok(())
}

/// C++: `edit.cpp` `EditGameUnitSkills`; edits per-man days and faction skill visibility.
pub fn set_unit_skill(
    loaded: &mut LoadedGame,
    unit_id: UnitId,
    skill: usize,
    days_per_man: i32,
) -> Result<(), EditError> {
    let tables = loaded.ruleset.tables();
    let skill_flags = tables
        .skills
        .skills
        .get(skill)
        .filter(|definition| definition.flags & crate::defs::skills::skill_flags::DISABLED == 0)
        .map(|definition| definition.flags)
        .ok_or(EditError::InvalidSkill(skill))?;
    if days_per_man < 0 {
        return Err(EditError::InvalidSkillDays);
    }
    let unit = loaded
        .game
        .unit(unit_id)
        .ok_or(EditError::Model(ModelError::MissingUnit(unit_id)))?;
    let faction_id = unit.faction;
    let men = unit_men(unit, &tables);
    let total_days = days_per_man
        .checked_mul(men)
        .and_then(|days| u32::try_from(days).ok())
        .ok_or(EditError::InvalidSkillDays)?;
    {
        let unit = loaded
            .game
            .unit_mut(unit_id)
            .expect("unit was checked before skill edit");
        if skill_flags & crate::defs::skills::skill_flags::MAGIC != 0 {
            unit.unit_type = UnitType::Mage;
        }
        if skill_flags & crate::defs::skills::skill_flags::APPRENTICE != 0
            && unit.unit_type == UnitType::Normal
        {
            unit.unit_type = UnitType::Apprentice;
        }
        unit.skills.set_days(skill, total_days);
    }
    if let Some(faction_id) = faction_id {
        let level = if men > 0 {
            level_by_days((total_days / men as u32) as i32)
        } else {
            0
        };
        if let Some(faction) = loaded.game.faction_mut(faction_id) {
            if level as u32 > faction.skills.get_days(skill) {
                faction.skills.set_days(skill, level as u32);
            }
        }
    }
    Ok(())
}

fn unit_men(unit: &crate::model::Unit, tables: &crate::defs::RulesetTables) -> i32 {
    unit.items
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
        .sum()
}

fn level_by_days(mut days_per_man: i32) -> i32 {
    let mut days = 30;
    let mut level = 0;
    while days_per_man >= days {
        level += 1;
        days_per_man -= days;
        days += 30;
    }
    level
}

/// C++: `edit.cpp` `EditGameUnitDetails`; changes the unit's controlling faction.
pub fn set_unit_faction(
    loaded: &mut LoadedGame,
    unit_id: UnitId,
    faction_id: FactionId,
) -> Result<(), EditError> {
    if loaded.game.faction(faction_id).is_none() {
        return Err(EditError::MissingFaction(faction_id));
    }
    loaded
        .game
        .unit_mut(unit_id)
        .ok_or(EditError::Model(ModelError::MissingUnit(unit_id)))?
        .faction = Some(faction_id);
    Ok(())
}

/// C++: `edit.cpp` `EditGameUnitDetails`; assigns one of the original unit types.
pub fn set_unit_type(
    loaded: &mut LoadedGame,
    unit_id: UnitId,
    unit_type: UnitType,
) -> Result<(), EditError> {
    loaded
        .game
        .unit_mut(unit_id)
        .ok_or(EditError::Model(ModelError::MissingUnit(unit_id)))?
        .unit_type = unit_type;
    Ok(())
}

/// C++: `edit.cpp` `EditGameUnitDetails`; replaces the unit's name.
pub fn rename_unit(loaded: &mut LoadedGame, unit_id: UnitId, name: &str) -> Result<(), EditError> {
    if name.trim().is_empty() {
        return Err(EditError::EmptyName);
    }
    loaded
        .game
        .unit_mut(unit_id)
        .ok_or(EditError::Model(ModelError::MissingUnit(unit_id)))?
        .name = Some(name.trim().to_owned());
    Ok(())
}

/// C++: the unit's persisted `describe` field; `None` clears it.
pub fn set_unit_description(
    loaded: &mut LoadedGame,
    unit_id: UnitId,
    description: Option<&str>,
) -> Result<(), EditError> {
    loaded
        .game
        .unit_mut(unit_id)
        .ok_or(EditError::Model(ModelError::MissingUnit(unit_id)))?
        .describe = description.map(str::to_owned);
    Ok(())
}

/// C++: the unit's persisted guard mode.
pub fn set_unit_guard(
    loaded: &mut LoadedGame,
    unit_id: UnitId,
    guard: GuardMode,
) -> Result<(), EditError> {
    loaded
        .game
        .unit_mut(unit_id)
        .ok_or(EditError::Model(ModelError::MissingUnit(unit_id)))?
        .guard = guard;
    Ok(())
}

/// C++: `edit.cpp` `EditGameRegionObjects`; appends a named building or fleet.
pub fn add_region_object(
    loaded: &mut LoadedGame,
    region_id: RegionId,
    object_type: usize,
) -> Result<ObjectId, EditError> {
    let tables = loaded.ruleset.tables();
    let definition = tables
        .objects
        .get(object_type)
        .ok_or(EditError::InvalidObjectType(object_type))?;
    if definition.flags & object_flags::DISABLED != 0 {
        return Err(EditError::DisabledObjectType(object_type));
    }
    let is_fleet = object_type == O_FLEET || definition.sailors > 0;
    let region = loaded
        .game
        .region(region_id)
        .ok_or(EditError::Model(ModelError::MissingRegion(region_id)))?;
    let number = if is_fleet {
        loaded.game.ship_seq
    } else {
        region.building_seq
    };
    let next_number = number.checked_add(1).ok_or(EditError::SequenceOverflow)?;
    let mut object = Object::new(default_movement_points(loaded)?);
    object.object_type =
        i32::try_from(object_type).map_err(|_| EditError::InvalidObjectType(object_type))?;
    object.incomplete = 0;
    object.inner = -1;
    object.num = number;
    object.name = Some(format!(
        "{} [{number}]",
        if is_fleet { "Fleet" } else { "Building" }
    ));
    let object_id = loaded.game.add_object(region_id, object)?;
    if is_fleet {
        loaded.game.ship_seq = next_number;
    } else {
        loaded
            .game
            .region_mut(region_id)
            .expect("region was checked before adding object")
            .building_seq = next_number;
    }
    Ok(object_id)
}

/// C++: `edit.cpp` `EditGameRegionObjects`; list removal leaves the allocated object untouched.
pub fn remove_region_object(
    loaded: &mut LoadedGame,
    region_id: RegionId,
    index: usize,
) -> Result<ObjectId, EditError> {
    let object_id = loaded
        .game
        .region(region_id)
        .ok_or(EditError::Model(ModelError::MissingRegion(region_id)))?
        .object_ids()
        .get(index)
        .copied()
        .ok_or(EditError::InvalidObjectIndex(index))?;
    loaded.game.detach_region_object(region_id, object_id)?;
    Ok(object_id)
}

/// C++: `edit.cpp` `EditGameRegionObjects`; renames non-dummy objects with their number suffix.
pub fn rename_region_object(
    loaded: &mut LoadedGame,
    region_id: RegionId,
    index: usize,
    name: &str,
) -> Result<(), EditError> {
    if index == 0 {
        return Err(EditError::DummyObjectCannotBeRenamed);
    }
    if name.trim().is_empty() {
        return Err(EditError::EmptyName);
    }
    let object_id = loaded
        .game
        .region(region_id)
        .ok_or(EditError::Model(ModelError::MissingRegion(region_id)))?
        .object_ids()
        .get(index)
        .copied()
        .ok_or(EditError::InvalidObjectIndex(index))?;
    let object = loaded
        .game
        .object_mut(object_id)
        .ok_or(EditError::Model(ModelError::MissingObject(object_id)))?;
    object.name = Some(format!("{} [{}]", name.trim(), object.num));
    Ok(())
}

fn default_movement_points(loaded: &LoadedGame) -> Result<i32, EditError> {
    let defs = loaded.ruleset.game_defs();
    if defs.max_speed <= 0 {
        return Err(EditError::InvalidMoveSpeed);
    }
    Ok(defs.phased_move_offset % defs.max_speed)
}

/// C++: `edit.cpp` `EditGameRegionTerrain`; assigns an enabled terrain definition.
pub fn set_region_terrain(
    loaded: &mut LoadedGame,
    region_id: RegionId,
    terrain: TerrainId,
) -> Result<(), EditError> {
    if loaded.ruleset.tables().terrains.get(terrain).is_none() {
        return Err(EditError::InvalidTerrain(terrain));
    }
    loaded
        .game
        .region_mut(region_id)
        .ok_or(EditError::Model(ModelError::MissingRegion(region_id)))?
        .terrain_type = Some(terrain);
    Ok(())
}

/// C++: `edit.cpp` `EditGameRegionTerrain`; `None` represents an unset race.
pub fn set_region_race(
    loaded: &mut LoadedGame,
    region_id: RegionId,
    race: Option<ItemId>,
) -> Result<(), EditError> {
    let tables = loaded.ruleset.tables();
    if let Some(race) = race {
        let item = tables
            .item_families
            .items
            .get(race.index())
            .filter(|item| item.item_type & ItemTypeFlags::MAN != 0)
            .filter(|item| item.flags & ItemFlags::DISABLED == 0);
        if item.is_none() {
            return Err(EditError::InvalidRace(race));
        }
    }
    transactional_edit(loaded, |staged| {
        staged
            .game
            .region_mut(region_id)
            .ok_or(EditError::Model(ModelError::MissingRegion(region_id)))?
            .race = race.map_or(-1, |race| race as i32);
        refresh_region_edit_state(staged, region_id)
    })
}

/// C++: `edit.cpp` `EditGameRegionTerrain`; the editor's `n` operation replaces the name.
pub fn rename_region(
    loaded: &mut LoadedGame,
    region_id: RegionId,
    name: &str,
) -> Result<(), EditError> {
    if name.trim().is_empty() {
        return Err(EditError::EmptyName);
    }
    loaded
        .game
        .region_mut(region_id)
        .ok_or(EditError::Model(ModelError::MissingRegion(region_id)))?
        .name = name.trim().to_owned();
    Ok(())
}

/// C++: `edit.cpp` `EditGameRegionTerrain`; a zero wage value leaves the maximum unchanged.
pub fn set_region_wages(
    loaded: &mut LoadedGame,
    region_id: RegionId,
    maximum: i32,
) -> Result<(), EditError> {
    transactional_edit(loaded, |staged| {
        let region = staged
            .game
            .region_mut(region_id)
            .ok_or(EditError::Model(ModelError::MissingRegion(region_id)))?;
        if maximum != 0 {
            let change = maximum - region.max_wages;
            region.max_wages = maximum;
            region.wages += change;
        }
        refresh_region_edit_state(staged, region_id)
    })
}

/// C++: `edit.cpp` `EditGameRegionTerrain` and `economy.cpp` `ARegion::SetupProds`.
pub fn regenerate_region_products(
    loaded: &mut LoadedGame,
    region_id: RegionId,
) -> Result<(), EditError> {
    let region = loaded
        .game
        .region(region_id)
        .ok_or(EditError::Model(ModelError::MissingRegion(region_id)))?;
    let terrain_id = region
        .terrain_type
        .ok_or(EditError::InvalidTerrain(usize::MAX))?;
    let tables = loaded.ruleset.tables();
    let terrain = tables
        .terrains
        .get(terrain_id)
        .ok_or(EditError::InvalidTerrain(terrain_id))?;
    let coastal = is_coastal(
        &loaded.game,
        region,
        &tables.terrains,
        loaded.ruleset.game_defs(),
    );
    let mut generated = Vec::new();
    let defs = loaded.ruleset.game_defs();

    if defs.food_items_exist != 0 && terrain.economy != 0 {
        let food_choice = loaded
            .rng
            .get_random(2 + i32::from(defs.coastal_fish != 0 && coastal));
        let food = match food_choice {
            0 => Some(ItemId::Grain),
            1 => Some(ItemId::Livestock),
            2 => Some(ItemId::Fish),
            _ => None,
        };
        if let Some(food) = food.filter(|food| item_enabled(&tables, *food as usize)) {
            generated.push(Production {
                item_type: food as i32,
                amount: terrain.economy,
                base_amount: terrain.economy,
                skill: -1,
                ..Production::default()
            });
        }
    }

    for product in terrain.products {
        if product.item < 0 || !item_enabled(&tables, product.item as usize) {
            continue;
        }
        let chance = product.chance;
        if loaded.rng.get_random(100) < chance {
            generated.push(Production {
                item_type: product.item,
                amount: product.amount,
                base_amount: product.amount,
                skill: -1,
                ..Production::default()
            });
        }
    }

    let region = loaded
        .game
        .region_mut(region_id)
        .expect("region was checked before random generation");
    region
        .products
        .retain(|product| product.item_type == ItemId::Silver as i32);
    region.products.extend(generated);
    Ok(())
}

/// C++: `edit.cpp` `g`; regenerates products, population, development, town, and markets.
pub fn regenerate_region(loaded: &mut LoadedGame, region_id: RegionId) -> Result<(), EditError> {
    transactional_edit(loaded, |staged| regenerate_region_inner(staged, region_id))
}

fn regenerate_region_inner(loaded: &mut LoadedGame, region_id: RegionId) -> Result<(), EditError> {
    let region = loaded
        .game
        .region_mut(region_id)
        .ok_or(EditError::Model(ModelError::MissingRegion(region_id)))?;
    region.town = None;
    region.markets.clear();
    region.products.clear();
    regenerate_region_products(loaded, region_id)?;
    setup_edit_region(loaded, region_id)?;
    refresh_region_edit_state(loaded, region_id)
}

fn setup_edit_region(loaded: &mut LoadedGame, region_id: RegionId) -> Result<(), EditError> {
    let tables = loaded.ruleset.tables();
    let defs = loaded.ruleset.game_defs();
    let mut region = loaded
        .game
        .region(region_id)
        .ok_or(EditError::Model(ModelError::MissingRegion(region_id)))?
        .clone();
    let terrain_id = region
        .terrain_type
        .ok_or(EditError::InvalidTerrain(usize::MAX))?;
    let terrain = *tables
        .terrains
        .get(terrain_id)
        .ok_or(EditError::InvalidTerrain(terrain_id))?;
    let coastal = is_coastal(&loaded.game, &region, &tables.terrains, defs);
    region.habitat = terrain.population + 1;
    if region.habitat > 1 {
        region.habitat *= 5;
    }
    if region.habitat < 100 && terrain.similar_type != R_OCEAN as i32 {
        region.habitat = 100;
    }
    let population = terrain.population;
    let mut minimum_wages = terrain.wages + defs.maintenance_cost - 10;
    if minimum_wages < 0 {
        minimum_wages = 0;
    }
    if population == 0 {
        region.population = 0;
        region.base_population = 0;
        region.wages = 0;
        region.max_wages = 0;
        region.wealth = 0;
        *loaded
            .game
            .region_mut(region_id)
            .expect("region was checked before regeneration") = region;
        return Ok(());
    }
    if region.race == -1 || defs.grow_races == 0 {
        region.race = -1;
        let noncoastal_races = terrain.races.len() as i32;
        let all_races = noncoastal_races + terrain.coastal_races.len() as i32;
        loop {
            let selected =
                loaded
                    .rng
                    .get_random(if coastal { all_races } else { noncoastal_races });
            let race = if coastal && selected >= noncoastal_races {
                terrain.coastal_races[(selected - noncoastal_races) as usize]
            } else {
                terrain.races[selected as usize]
            };
            if race >= 0 && item_enabled(&tables, race as usize) {
                region.race = race;
                break;
            }
        }
    }
    region.habitat = region.habitat * 2 / 3 + loaded.rng.get_random(region.habitat / 3);
    region.habitat = adjust_region_habitat(region.habitat, region.race, &terrain, coastal, &tables);
    region.base_population = region.habitat / 3;
    region.population =
        region.habitat * (60 + loaded.rng.get_random(6) + loaded.rng.get_random(6)) / 100;
    let mut level = 1;
    region.development = 1;
    let mut previous = 0;
    while level < minimum_wages {
        region.development += 1;
        previous += 1;
        if previous > level {
            level += 1;
            previous = 0;
        }
    }
    region.development += loaded.rng.get_random(25);
    region.max_development = region.development;

    let mut add_town = false;
    if defs.towns_exist != 0 {
        let adjacent = if defs.towns_not_adjacent != 0 {
            region
                .neighbors
                .iter()
                .flatten()
                .filter(|neighbor| {
                    loaded
                        .game
                        .region(**neighbor)
                        .is_some_and(|neighbor| neighbor.town.is_some())
                })
                .count() as i32
        } else {
            0
        };
        let town_probability = if defs.town_probability < 1 {
            100
        } else {
            defs.town_probability
        };
        let mut town_chance = 80000 / town_probability;
        if defs.less_arctic_towns != 0 {
            for direction in [0, 3] {
                let pole_distance = pole_distance(&loaded.game, &region, direction);
                if pole_distance < 9 {
                    town_chance +=
                        25 * (9 - pole_distance) * (9 - pole_distance) * defs.less_arctic_towns;
                }
            }
        }
        let spread = defs.town_spread.min(100);
        let mut town_probability = (terrain.economy * 4 * (100 - spread) + 100 * spread) / 100;
        if adjacent > 0 {
            town_probability = town_probability * (100 - defs.towns_not_adjacent) / 100;
        }
        add_town = loaded.rng.get_random(town_chance) < town_probability;
    }
    if add_town {
        let level = determine_town_size(loaded);
        let mut town = TownInfo {
            name: Some("Newtown".to_owned()),
            ..TownInfo::default()
        };
        set_town_type(loaded, &mut region, &mut town, level);
        region.town = Some(town);
        region.max_development = region.development;
    }
    *loaded
        .game
        .region_mut(region_id)
        .expect("region was checked before regeneration") = region;
    if add_town {
        setup_city_market(loaded, region_id)?;
        remove_town_lairs(loaded, region_id)?;
    }
    Ok(())
}

fn is_native_race(race: i32, terrain: &crate::defs::TerrainDef, coastal: bool) -> bool {
    terrain.races.contains(&race) || (coastal && terrain.coastal_races.contains(&race))
}

/// C++: `edit.cpp` applies the home-terrain bonus separately from the non-native penalty.
fn adjust_region_habitat(
    mut habitat: i32,
    race: i32,
    terrain: &crate::defs::TerrainDef,
    coastal: bool,
    tables: &crate::defs::RulesetTables,
) -> i32 {
    let home_terrain = usize::try_from(race)
        .ok()
        .and_then(|race| tables.item_families.items.get(race))
        .and_then(|item| item.abbreviation.as_deref())
        .and_then(|abbreviation| {
            tables
                .item_families
                .races
                .iter()
                .find(|race| race.abbreviation.as_deref() == Some(abbreviation))
        })
        .map(|race| race.terrain);
    if home_terrain == Some(terrain.similar_type) {
        habitat = habitat * 9 / 8;
    }
    if !is_native_race(race, terrain, coastal) {
        habitat = habitat * 4 / 5;
    }
    habitat
}

fn pole_distance(game: &Game, region: &Region, direction: usize) -> i32 {
    let mut distance = 1;
    let mut next = region.neighbors[direction];
    while let Some(region_id) = next {
        distance += 1;
        next = game
            .region(region_id)
            .and_then(|neighbor| neighbor.neighbors[direction]);
    }
    distance
}

/// C++: `edit.cpp` `EditGameRegionMarkets`; sets or creates a market's population range.
pub fn set_market_population_range(
    loaded: &mut LoadedGame,
    region_id: RegionId,
    item: ItemId,
    minimum: i32,
    maximum: i32,
) -> Result<(), EditError> {
    validate_market_edit(loaded, region_id, item)?;
    if minimum >= maximum {
        return Err(EditError::InvalidMarketRange);
    }
    let region = loaded
        .game
        .region(region_id)
        .expect("region was validated above");
    let population = checked_market_region_population(region)?;
    let updated_amounts = region
        .markets
        .iter()
        .enumerate()
        .filter(|(_, market)| market.item == item as i32)
        .map(|(index, market)| {
            let mut updated = market.clone();
            updated.min_population = minimum;
            updated.max_population = maximum;
            Ok((index, market_amount(&updated, population)?))
        })
        .collect::<Result<Vec<_>, EditError>>()?;
    if !updated_amounts.is_empty() {
        let region = loaded
            .game
            .region_mut(region_id)
            .expect("region was validated above");
        for (index, amount) in updated_amounts {
            let market = &mut region.markets[index];
            market.min_population = minimum;
            market.max_population = maximum;
            market.amount = amount;
        }
        return Ok(());
    }
    let tables = loaded.ruleset.tables();
    let base = tables.item_families.items[item.index()].base_price;
    let price = (base * (100 + loaded.rng.get_random(50))) / 100;
    let mut market = Market {
        market_type: MarketType::Sell,
        item: item as i32,
        price,
        min_population: minimum,
        max_population: maximum,
        ..Market::default()
    };
    market.base_price = price;
    loaded
        .game
        .region_mut(region_id)
        .expect("region was validated above")
        .markets
        .push(market);
    Ok(())
}

/// C++: `edit.cpp` `EditGameRegionMarkets`; sets or creates a market's amount range.
pub fn set_market_amount_range(
    loaded: &mut LoadedGame,
    region_id: RegionId,
    item: ItemId,
    minimum: i32,
    maximum: i32,
) -> Result<(), EditError> {
    validate_market_edit(loaded, region_id, item)?;
    if minimum >= maximum {
        return Err(EditError::InvalidMarketRange);
    }
    let region = loaded
        .game
        .region(region_id)
        .expect("region was validated above");
    let population = checked_market_region_population(region)?;
    let updated_amounts = region
        .markets
        .iter()
        .enumerate()
        .filter(|(_, market)| market.item == item as i32)
        .map(|(index, market)| {
            let mut updated = market.clone();
            updated.min_amount = minimum;
            updated.max_amount = maximum;
            Ok((index, market_amount(&updated, population)?))
        })
        .collect::<Result<Vec<_>, EditError>>()?;
    if !updated_amounts.is_empty() {
        let region = loaded
            .game
            .region_mut(region_id)
            .expect("region was validated above");
        for (index, amount) in updated_amounts {
            let market = &mut region.markets[index];
            market.min_amount = minimum;
            market.max_amount = maximum;
            market.amount = amount;
        }
        return Ok(());
    }
    let scaled_amount = maximum
        .checked_mul(population)
        .ok_or(EditError::MarketAmountOverflow)?
        / 5000;
    let amount = minimum
        .checked_add(scaled_amount)
        .ok_or(EditError::MarketAmountOverflow)?;
    let tables = loaded.ruleset.tables();
    let base = tables.item_families.items[item.index()].base_price;
    let price = (base * (100 + loaded.rng.get_random(50))) / 100;
    let mut market = Market {
        market_type: MarketType::Sell,
        item: item as i32,
        price,
        amount,
        max_population: 5000,
        min_amount: minimum,
        max_amount: maximum,
        ..Market::default()
    };
    market.base_price = price;
    loaded
        .game
        .region_mut(region_id)
        .expect("region was validated above")
        .markets
        .push(market);
    Ok(())
}

/// C++: `edit.cpp` `EditGameRegionMarkets`; sets or creates a market's current and base prices.
pub fn set_market_prices(
    loaded: &mut LoadedGame,
    region_id: RegionId,
    item: ItemId,
    price: i32,
    base_price: i32,
) -> Result<(), EditError> {
    validate_market_edit(loaded, region_id, item)?;
    if price < 1 || base_price < 1 {
        return Err(EditError::InvalidMarketPrice);
    }
    let region = loaded
        .game
        .region_mut(region_id)
        .expect("region was validated above");
    if region
        .markets
        .iter()
        .any(|market| market.item == item as i32)
    {
        for market in region
            .markets
            .iter_mut()
            .filter(|market| market.item == item as i32)
        {
            market.price = price;
            market.base_price = base_price;
        }
    } else {
        region.markets.push(Market {
            market_type: MarketType::Sell,
            item: item as i32,
            price,
            max_population: 5000,
            base_price,
            ..Market::default()
        });
    }
    Ok(())
}

/// C++: `edit.cpp` `EditGameRegionMarkets`; swaps the first market and removes same-item duplicates.
pub fn swap_market_type(
    loaded: &mut LoadedGame,
    region_id: RegionId,
    item: ItemId,
) -> Result<(), EditError> {
    validate_market_edit(loaded, region_id, item)?;
    let region = loaded
        .game
        .region_mut(region_id)
        .expect("region was validated above");
    let mut found = false;
    region.markets.retain_mut(|market| {
        if market.item != item as i32 {
            return true;
        }
        if found {
            return false;
        }
        found = true;
        market.market_type = match market.market_type {
            MarketType::Buy => MarketType::Sell,
            MarketType::Sell => MarketType::Buy,
        };
        true
    });
    if found {
        Ok(())
    } else {
        Err(EditError::MarketNotFound(item))
    }
}

/// C++: `edit.cpp` `EditGameRegionMarkets`; removes every market entry for an item.
pub fn remove_market(
    loaded: &mut LoadedGame,
    region_id: RegionId,
    item: ItemId,
) -> Result<usize, EditError> {
    validate_market_edit(loaded, region_id, item)?;
    let region = loaded
        .game
        .region_mut(region_id)
        .expect("region was validated above");
    let old_len = region.markets.len();
    region.markets.retain(|market| market.item != item as i32);
    Ok(old_len - region.markets.len())
}

fn validate_market_edit(
    loaded: &LoadedGame,
    region_id: RegionId,
    item: ItemId,
) -> Result<(), EditError> {
    loaded
        .game
        .region(region_id)
        .ok_or(EditError::Model(ModelError::MissingRegion(region_id)))?;
    if !item_enabled(&loaded.ruleset.tables(), item.index()) {
        return Err(EditError::InvalidMarketItem(item));
    }
    Ok(())
}

fn item_enabled(tables: &crate::defs::RulesetTables, item: usize) -> bool {
    tables
        .item_families
        .items
        .get(item)
        .is_some_and(|definition| definition.flags & ItemFlags::DISABLED == 0)
}

fn market_amount(market: &Market, population: i32) -> Result<i32, EditError> {
    if population <= market.min_population {
        Ok(market.min_amount)
    } else if population >= market.max_population {
        Ok(market.max_amount)
    } else {
        let amount_range = market
            .max_amount
            .checked_sub(market.min_amount)
            .ok_or(EditError::MarketAmountOverflow)?;
        let population_offset = population
            .checked_sub(market.min_population)
            .ok_or(EditError::MarketAmountOverflow)?;
        let population_range = market
            .max_population
            .checked_sub(market.min_population)
            .ok_or(EditError::MarketAmountOverflow)?;
        let scaled_amount = amount_range
            .checked_mul(population_offset)
            .and_then(|amount| amount.checked_div(population_range))
            .ok_or(EditError::MarketAmountOverflow)?;
        market
            .min_amount
            .checked_add(scaled_amount)
            .ok_or(EditError::MarketAmountOverflow)
    }
}

fn checked_market_region_population(region: &Region) -> Result<i32, EditError> {
    region
        .population
        .checked_add(region.town.as_ref().map_or(0, |town| town.population))
        .ok_or(EditError::MarketAmountOverflow)
}

fn region_population(region: &Region) -> i32 {
    region.population + region.town.as_ref().map_or(0, |town| town.population)
}

fn is_coastal(
    game: &Game,
    region: &Region,
    terrains: &[crate::defs::TerrainDef],
    defs: &crate::defs::GameDefs,
) -> bool {
    let Some(terrain) = region
        .terrain_type
        .and_then(|terrain| terrains.get(terrain))
    else {
        return false;
    };
    if region.terrain_type == Some(R_LAKE) {
        if defs.lakeside_is_coastal != 0 {
            return true;
        }
    } else if terrain.similar_type == R_OCEAN as i32 {
        return true;
    }
    region.neighbors.iter().flatten().any(|neighbor_id| {
        let Some(neighbor_region) = game.region(*neighbor_id) else {
            return false;
        };
        neighbor_region
            .terrain_type
            .and_then(|neighbor_terrain| terrains.get(neighbor_terrain))
            .is_some_and(|neighbor| {
                neighbor.similar_type == R_OCEAN as i32
                    && (defs.lakeside_is_coastal != 0
                        || neighbor_region.terrain_type != Some(R_LAKE))
            })
    })
}

fn refresh_region_edit_state(
    loaded: &mut LoadedGame,
    region_id: RegionId,
) -> Result<(), EditError> {
    let ruleset = loaded.ruleset;
    let tables = ruleset.tables();
    let defs = ruleset.game_defs();
    let mut region = loaded
        .game
        .region(region_id)
        .expect("region was validated by edit operation")
        .clone();
    region.wages = calculate_wages(&loaded.game, region_id, &region, defs);
    if region.base_population != 0 {
        region.max_wages = region.wages;
        let population = checked_market_region_population(&region)?;
        region.wealth = ((population * (region.wages - 10 * defs.maintenance_cost)) / 50).max(0);
        let rural_population = if population < 3000 {
            let wage_factor = (120 / (61 - population / 50)) as f32;
            population + ((wage_factor * population as f32 + 3000.0) / (wage_factor + 1.0)) as i32
        } else {
            population
        };
        let wage_income =
            (rural_population * (region.wages - 10 * defs.maintenance_cost) / 50).max(0);
        let entertainment_population = if population < 3000 {
            let factor = (population / 10 + 300) / 6;
            population * factor / 100
        } else {
            population
        };
        let entertainment_income =
            (entertainment_population * (region.wages - 10 * defs.maintenance_cost + 10) / 50)
                .max(0);
        let wage_productivity = region.wages;
        if defs.work_fraction == 0 || defs.entertain_fraction == 0 || defs.base_man_cost == 0 {
            return Err(EditError::InvalidEconomyParameters);
        }
        set_production(
            &mut region,
            ItemId::Silver as i32,
            -1,
            wage_income / defs.work_fraction,
            wage_productivity,
        );
        let entertainment_skill = tables
            .skills
            .skills
            .iter()
            .position(|skill| skill.abbreviation == Some("ENTE"))
            .map_or(-1, |skill| skill as i32);
        set_production(
            &mut region,
            ItemId::Silver as i32,
            entertainment_skill,
            entertainment_income / defs.entertain_fraction,
            defs.entertain_income * 10,
        );
    }
    let population = checked_market_region_population(&region)?;
    if defs.variable_economy != 0 {
        for market in &mut region.markets {
            let Some(item) = usize::try_from(market.item)
                .ok()
                .and_then(|index| tables.item_families.items.get(index))
            else {
                continue;
            };
            if market.amount == -1 {
                continue;
            }
            if item.item_type & ItemTypeFlags::MAN != 0 {
                let denominator = (defs.base_man_cost * 10) as f32;
                let ratio = item.base_price as f32 / denominator;
                market.price = (region.wages as f32 * 4.0 * ratio) as i32;
                market.amount = if item.item_type & ItemTypeFlags::LEADER != 0 {
                    population / 125
                } else {
                    population / 25
                };
            } else {
                let mut target_price = market.price;
                if market.amount != 0 {
                    let fluctuation = market.base_price * market.activity / market.amount;
                    target_price = match market.market_type {
                        MarketType::Buy => (2 * market.base_price + fluctuation) / 2,
                        MarketType::Sell => (3 * market.base_price - fluctuation) / 2,
                    };
                }
                market.price += (target_price - market.price) / 5;
                market.amount = market_amount(market, population)?;
            }
        }
    }
    region.markets.retain(|market| {
        usize::try_from(market.item)
            .ok()
            .and_then(|index| tables.item_families.items.get(index))
            .is_none_or(|item| item.item_type & ItemTypeFlags::MAN == 0)
    });
    if let Some(race) = usize::try_from(region.race)
        .ok()
        .filter(|race| *race < tables.item_families.items.len())
    {
        let item = &tables.item_families.items[race];
        let ratio = item.base_price as f32 / (defs.base_man_cost * 10) as f32;
        let price = (region.wages as f32 * 4.0 * ratio) as i32;
        region.markets.push(Market {
            market_type: MarketType::Buy,
            item: race as i32,
            price,
            amount: population / 25,
            min_population: 0,
            max_population: 10000,
            min_amount: 0,
            max_amount: 2000,
            base_price: price,
            ..Market::default()
        });
        if defs.leaders_exist != 0 {
            let leaders = &tables.item_families.items[ItemId::Leaders as usize];
            let ratio = leaders.base_price as f32 / (defs.base_man_cost * 10) as f32;
            let price = (region.wages as f32 * 4.0 * ratio) as i32;
            region.markets.push(Market {
                market_type: MarketType::Buy,
                item: ItemId::Leaders as i32,
                price,
                amount: population / 125,
                min_population: 0,
                max_population: 10000,
                min_amount: 0,
                max_amount: 400,
                base_price: price,
                ..Market::default()
            });
        }
    }
    *loaded
        .game
        .region_mut(region_id)
        .expect("region exists throughout edit state refresh") = region;
    Ok(())
}

fn set_production(region: &mut Region, item_type: i32, skill: i32, amount: i32, productivity: i32) {
    let production = region
        .products
        .iter_mut()
        .find(|production| production.item_type == item_type && production.skill == skill);
    let production = if let Some(production) = production {
        production
    } else {
        region.products.push(Production::default());
        region.products.last_mut().expect("production was appended")
    };
    production.item_type = item_type;
    production.skill = skill;
    production.amount = amount;
    production.base_amount = amount;
    production.productivity = productivity;
}

fn calculate_wages(
    game: &Game,
    region_id: RegionId,
    region: &Region,
    defs: &crate::defs::GameDefs,
) -> i32 {
    if region_population(region) == 0 {
        return 0;
    }
    let development = region.development
        + road_development(game, region_id, region)
        + (region.earthlore + region.clearskies) * 12;
    let development = development
        + region
            .town
            .as_ref()
            .map_or(0, |town| town_type(town, defs).pow(2) + 1);
    let mut level = 1;
    let mut last = 0;
    let mut wages = 0;
    while development >= level {
        wages += 1;
        last = level;
        level += wages + 1;
    }
    wages *= 10;
    if development > last {
        wages += 10 * (development - last) / (level - last);
    }
    wages
}

fn road_development(game: &Game, region_id: RegionId, region: &Region) -> i32 {
    let roads = RoadGraph { game };
    let road_count = (0..6)
        .filter(|direction| roads.has_exit_road(region_id, *direction))
        .count() as i32;
    if road_count == 0 {
        return 0;
    }
    let mut visited = vec![region_id];
    let mut bonus = 0;
    for direction in 0..6 {
        if !roads.has_exit_road(region_id, direction) {
            continue;
        }
        let Some(neighbor) = game
            .region(region_id)
            .and_then(|region| region.neighbors[direction])
        else {
            continue;
        };
        if roads.has_connecting_road(region_id, direction) {
            bonus = roads.trace_connected_road(
                neighbor,
                direction,
                bonus,
                15,
                region.development,
                &mut visited,
            );
        }
    }
    if region.town.is_none() {
        bonus /= 2;
    }
    let mut next_bonus = 5;
    let mut level_off = 1;
    let mut plateau = 4;
    let mut total = 0;
    while bonus > 0 && total < 46 {
        bonus -= 1;
        if level_off >= plateau && next_bonus > 1 {
            next_bonus -= 1;
            level_off = 1;
            plateau -= 1;
        }
        level_off += 1;
        total += next_bonus;
    }
    total
}

struct RoadGraph<'a> {
    game: &'a Game,
}

impl RoadGraph<'_> {
    fn has_exit_road(&self, region_id: RegionId, direction: usize) -> bool {
        const ROAD_TYPES: [usize; 6] = [O_ROADN, O_ROADNE, O_ROADSE, O_ROADS, O_ROADSW, O_ROADNW];
        self.game.region(region_id).is_some_and(|region| {
            region.object_ids().iter().any(|object_id| {
                self.game.object(*object_id).is_some_and(|object| {
                    object.object_type == ROAD_TYPES[direction] as i32 && object.incomplete < 1
                })
            })
        })
    }

    fn real_complement(&self, region_id: RegionId, direction: usize) -> usize {
        let neighbor = self
            .game
            .region(region_id)
            .and_then(|region| region.neighbors[direction]);
        if let Some(direction) = neighbor.and_then(|neighbor| {
            self.game.region(neighbor).and_then(|region| {
                region
                    .neighbors
                    .iter()
                    .position(|neighbor| *neighbor == Some(region_id))
            })
        }) {
            return direction;
        }
        [3, 4, 5, 0, 2, 1][direction]
    }

    fn has_connecting_road(&self, region_id: RegionId, direction: usize) -> bool {
        self.game
            .region(region_id)
            .and_then(|region| region.neighbors[direction])
            .is_some_and(|neighbor| {
                self.has_exit_road(neighbor, self.real_complement(region_id, direction))
            })
    }

    fn trace_connected_road(
        &self,
        region_id: RegionId,
        incoming_direction: usize,
        mut bonus: i32,
        range: i32,
        development: i32,
        visited: &mut Vec<RegionId>,
    ) -> i32 {
        if visited.contains(&region_id) {
            return bonus;
        }
        visited.push(region_id);
        let Some(region) = self.game.region(region_id) else {
            return bonus;
        };
        if region.town.is_some() {
            bonus += 1;
        }
        if region.development > development + 9 {
            bonus += 1;
        }
        if region.development * 2 > development * 5 {
            bonus += 1;
        }
        if range > 0 {
            for direction in 0..6 {
                if !self.has_exit_road(region_id, direction) {
                    continue;
                }
                let Some(next_region) = region.neighbors[direction] else {
                    continue;
                };
                if incoming_direction == self.real_complement(next_region, direction) {
                    continue;
                }
                if self.has_connecting_road(region_id, direction) {
                    bonus = self.trace_connected_road(
                        next_region,
                        direction,
                        bonus,
                        range - 1,
                        development + 2,
                        visited,
                    );
                }
            }
        }
        bonus
    }
}

fn town_type(town: &TownInfo, defs: &crate::defs::GameDefs) -> i32 {
    let prestige = town.population * (town.development + 220) / 270;
    if prestige < defs.city_pop / 4 {
        0
    } else if prestige < defs.city_pop * 4 / 5 {
        1
    } else {
        2
    }
}

/// C++: `edit.cpp` `ag`; assigns a unique gate number using the active numbering rules.
pub fn add_region_gate(loaded: &mut LoadedGame, region_id: RegionId) -> Result<(), EditError> {
    if loaded.ruleset.game_defs().gates_exist == 0 {
        return Err(EditError::GatesDisabled);
    }
    let region = loaded
        .game
        .region(region_id)
        .ok_or(EditError::Model(ModelError::MissingRegion(region_id)))?;
    if region.gate > 0 {
        return Err(EditError::GateAlreadyExists(region_id));
    }
    let new_gate_count = loaded
        .game
        .number_of_gates
        .checked_add(1)
        .ok_or(EditError::GateStateInconsistent)?;
    let gate = if loaded.ruleset.game_defs().disperse_gate_numbers != 0 {
        let mut digit_count = 0;
        let mut count = new_gate_count;
        while count > 0 {
            count /= 10;
            digit_count += 1;
        }
        let mut range = 10;
        while digit_count > 0 {
            range *= 10;
            digit_count -= 1;
        }
        let mut used = vec![false; range as usize];
        for id in loaded.game.region_ids() {
            let existing = loaded.game.region(*id).expect("listed region exists").gate;
            if existing > 0 {
                let index =
                    usize::try_from(existing - 1).map_err(|_| EditError::GateStateInconsistent)?;
                let slot = used
                    .get_mut(index)
                    .ok_or(EditError::GateStateInconsistent)?;
                *slot = true;
            }
        }
        let mut index = loaded.rng.get_random(range) as usize;
        while used[index] {
            index = loaded.rng.get_random(range) as usize;
        }
        index as i32 + 1
    } else {
        let gate = loaded.rng.get_random(new_gate_count) + 1;
        if gate != new_gate_count {
            let region_ids = loaded.game.region_ids().to_vec();
            for id in region_ids {
                let region = loaded.game.region_mut(id).expect("listed region exists");
                if region.gate == gate {
                    region.gate = new_gate_count;
                }
            }
        }
        gate
    };
    let gate_month = loaded.rng.get_random(12);
    let region = loaded
        .game
        .region_mut(region_id)
        .expect("region was checked before gate generation");
    region.gate = gate;
    region.gate_month = gate_month;
    loaded.game.number_of_gates = new_gate_count;
    Ok(())
}

/// C++: `edit.cpp` `dg`; removes a gate and compacts numbered gates when required.
pub fn remove_region_gate(loaded: &mut LoadedGame, region_id: RegionId) -> Result<(), EditError> {
    let gate = loaded
        .game
        .region(region_id)
        .ok_or(EditError::Model(ModelError::MissingRegion(region_id)))?
        .gate;
    if gate <= 0 {
        return Err(EditError::GateNotFound(region_id));
    }
    if loaded.game.number_of_gates <= 0 {
        return Err(EditError::GateStateInconsistent);
    }
    if loaded.ruleset.game_defs().disperse_gate_numbers == 0 {
        let last_gate = loaded.game.number_of_gates;
        let Some(last_region) = loaded.game.region_ids().iter().copied().find(|id| {
            loaded
                .game
                .region(*id)
                .is_some_and(|region| region.gate == last_gate)
        }) else {
            return Err(EditError::GateStateInconsistent);
        };
        if let Some(region) = loaded.game.region_mut(last_region) {
            region.gate = gate;
        }
    }
    let region = loaded
        .game
        .region_mut(region_id)
        .expect("region was checked before gate removal");
    region.gate = 0;
    loaded.game.number_of_gates -= 1;
    Ok(())
}

/// C++: `edit.cpp` `town`; regenerates the town while preserving an existing town name.
pub fn add_or_regenerate_town(
    loaded: &mut LoadedGame,
    region_id: RegionId,
) -> Result<(), EditError> {
    transactional_edit(loaded, |staged| {
        add_or_regenerate_town_inner(staged, region_id)
    })
}

fn add_or_regenerate_town_inner(
    loaded: &mut LoadedGame,
    region_id: RegionId,
) -> Result<(), EditError> {
    if loaded.ruleset.game_defs().towns_exist == 0 {
        return Err(EditError::TownsDisabled);
    }
    let mut region = loaded
        .game
        .region(region_id)
        .ok_or(EditError::Model(ModelError::MissingRegion(region_id)))?
        .clone();
    let name = region
        .town
        .as_ref()
        .and_then(|town| town.name.clone())
        .unwrap_or_else(|| "name".to_owned());
    if region.town.take().is_some() {
        region.markets.clear();
    }
    if region.race < 0 {
        region.race = 9;
    }
    let level = determine_town_size(loaded);
    let mut town = TownInfo {
        name: Some(name),
        ..TownInfo::default()
    };
    set_town_type(loaded, &mut region, &mut town, level);
    region.town = Some(town);
    *loaded
        .game
        .region_mut(region_id)
        .expect("region was checked before town generation") = region;
    setup_city_market(loaded, region_id)?;
    remove_town_lairs(loaded, region_id)?;
    refresh_region_edit_state(loaded, region_id)?;
    Ok(())
}

/// C++: `edit.cpp` `tn`; renames the existing town.
pub fn rename_town(
    loaded: &mut LoadedGame,
    region_id: RegionId,
    name: &str,
) -> Result<(), EditError> {
    if name.trim().is_empty() {
        return Err(EditError::EmptyName);
    }
    let town = loaded
        .game
        .region_mut(region_id)
        .ok_or(EditError::Model(ModelError::MissingRegion(region_id)))?
        .town
        .as_mut()
        .ok_or(EditError::TownNotFound(region_id))?;
    town.name = Some(name.trim().to_owned());
    Ok(())
}

/// C++: `edit.cpp` `deltown`; removes the town and clears its city markets.
pub fn remove_town(loaded: &mut LoadedGame, region_id: RegionId) -> Result<(), EditError> {
    transactional_edit(loaded, |staged| {
        let region = staged
            .game
            .region_mut(region_id)
            .ok_or(EditError::Model(ModelError::MissingRegion(region_id)))?;
        if region.town.take().is_some() {
            region.markets.clear();
            refresh_region_edit_state(staged, region_id)?;
        }
        Ok(())
    })
}

/// C++: `economy.cpp` `ARegion::SetupCityMarket`; replaces all markets in a town region.
pub fn regenerate_markets(loaded: &mut LoadedGame, region_id: RegionId) -> Result<(), EditError> {
    transactional_edit(loaded, |staged| regenerate_markets_inner(staged, region_id))
}

fn regenerate_markets_inner(loaded: &mut LoadedGame, region_id: RegionId) -> Result<(), EditError> {
    let region = loaded
        .game
        .region(region_id)
        .ok_or(EditError::Model(ModelError::MissingRegion(region_id)))?;
    if region.town.is_none() {
        return Err(EditError::TownNotFound(region_id));
    }
    loaded
        .game
        .region_mut(region_id)
        .expect("region was checked before market regeneration")
        .markets
        .clear();
    setup_city_market(loaded, region_id)?;
    refresh_region_edit_state(loaded, region_id)?;
    Ok(())
}

fn transactional_edit(
    loaded: &mut LoadedGame,
    operation: impl FnOnce(&mut LoadedGame) -> Result<(), EditError>,
) -> Result<(), EditError> {
    let mut staged = loaded.clone();
    operation(&mut staged)?;
    *loaded = staged;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::adjust_region_habitat;
    use crate::defs::{
        terrains::{R_DESERT, R_PLAIN},
        ItemId,
    };
    use crate::ruleset::Ruleset;

    #[test]
    fn region_habitat_uses_race_home_and_native_terrain_independently() {
        let tables = <dyn Ruleset>::from_name("standard")
            .expect("standard ruleset")
            .tables();
        let plains = tables.terrains[R_PLAIN];
        let nomad = ItemId::Nomad as i32;
        let plainsman = ItemId::Plainsman as i32;

        assert!(plains.races.contains(&nomad));
        let nomad_abbreviation = tables.item_families.items[ItemId::Nomad.index()]
            .abbreviation
            .as_deref();
        let nomad_home = tables
            .item_families
            .races
            .iter()
            .find(|race| race.abbreviation.as_deref() == nomad_abbreviation)
            .expect("Nomad has a race definition")
            .terrain;
        assert_eq!(nomad_home, R_DESERT as i32);
        assert_eq!(
            adjust_region_habitat(1000, nomad, &plains, false, &tables),
            1000
        );

        let plainsman_abbreviation = tables.item_families.items[ItemId::Plainsman.index()]
            .abbreviation
            .as_deref();
        let plainsman_home = tables
            .item_families
            .races
            .iter()
            .find(|race| race.abbreviation.as_deref() == plainsman_abbreviation)
            .expect("Plainsman has a race definition")
            .terrain;
        assert_eq!(plainsman_home, plains.similar_type);
        assert!(plains.races.contains(&plainsman));
        assert_eq!(
            adjust_region_habitat(1000, plainsman, &plains, false, &tables),
            1125
        );

        let mut nonnative_plains = plains;
        nonnative_plains.races = [-1; 4];
        nonnative_plains.coastal_races = [-1; 3];
        assert_eq!(
            adjust_region_habitat(1000, plainsman, &nonnative_plains, false, &tables),
            900
        );
    }
}

fn determine_town_size(loaded: &mut LoadedGame) -> i32 {
    let defs = loaded.ruleset.game_defs();
    if loaded.rng.get_random(300) < defs.town_development {
        2
    } else if loaded.rng.get_random(220) < defs.town_development + 10 {
        1
    } else {
        0
    }
}

fn set_town_type(loaded: &mut LoadedGame, region: &mut Region, town: &mut TownInfo, level: i32) {
    town.habitat = town_habitat(
        region,
        &loaded.game,
        &loaded.ruleset.tables(),
        loaded.ruleset.game_defs(),
    );
    town.population = town.habitat * 2 / 3;
    town.development = town_development(
        region,
        &loaded.ruleset.tables().terrains,
        loaded.ruleset.game_defs(),
    );
    let city_pop = loaded.ruleset.game_defs().city_pop;
    let population_addition = loaded
        .rng
        .get_random((level - 1) * (level - 1) * city_pop / 12)
        + level * level * city_pop / 12;
    town.habitat += population_addition;
    town.population = town.habitat * 2 / 3;
    region.development += level * 6 + 2;
    town.development = town_development(
        region,
        &loaded.ruleset.tables().terrains,
        loaded.ruleset.game_defs(),
    );
    while town_type(town, loaded.ruleset.game_defs()) != level {
        if level > town_type(town, loaded.ruleset.game_defs()) {
            region.development += loaded
                .rng
                .get_random(loaded.ruleset.game_defs().town_development / 10 + 5);
            let mut population_addition =
                loaded.rng.get_random(city_pop / 3) + loaded.rng.get_random(city_pop / 3);
            while town.population + population_addition > city_pop {
                population_addition /= 2;
            }
            town.habitat += population_addition;
            town.population = town.habitat * 2 / 3;
            town.development = town_development(
                region,
                &loaded.ruleset.tables().terrains,
                loaded.ruleset.game_defs(),
            );
        } else {
            region.development -= loaded
                .rng
                .get_random(20 - loaded.ruleset.game_defs().town_development / 10);
            let mut population_decrease =
                loaded.rng.get_random(city_pop / 3) + loaded.rng.get_random(city_pop / 3);
            while town.population < population_decrease || town.habitat < population_decrease {
                population_decrease /= 2;
            }
            town.habitat -= population_decrease;
            town.population = town.habitat * 2 / 3;
            town.development = town_development(
                region,
                &loaded.ruleset.tables().terrains,
                loaded.ruleset.game_defs(),
            );
        }
    }
    region.max_development = region.development;
}

fn town_habitat(
    region: &Region,
    game: &Game,
    tables: &crate::defs::RulesetTables,
    defs: &crate::defs::GameDefs,
) -> i32 {
    let mut fort = 0;
    let mut farm = 0;
    let mut inn = 0;
    let mut temple = 0;
    let mut caravan = 0;
    for object_id in region.object_ids() {
        let Some(object) = game.object(*object_id).and_then(|object| {
            usize::try_from(object.object_type)
                .ok()
                .and_then(|object_type| tables.objects.get(object_type))
                .map(|definition| (object, definition))
        }) else {
            continue;
        };
        if object.1.protection > fort {
            fort = object.1.protection;
        }
        if usize::try_from(object.1.production_aided)
            .ok()
            .and_then(|item| tables.item_families.items.get(item))
            .is_some_and(|item| item.flags & ItemTypeFlags::FOOD != 0)
        {
            farm += 1;
        }
        if object.1.production_aided == ItemId::Silver as i32 {
            inn += 1;
        }
        if object.1.production_aided == ItemId::Herbs as i32 {
            temple += 1;
        }
        if object.1.flags & object_flags::TRANSPORT != 0
            && usize::try_from(object.1.production_aided)
                .ok()
                .and_then(|item| tables.item_families.items.get(item))
                .is_some_and(|item| item.item_type & ItemTypeFlags::MOUNT != 0)
        {
            caravan += 1;
        }
    }
    let mut habitat = 2;
    let mut step = 0;
    for _ in 0..5 {
        if fort > step {
            habitat += 1;
        }
        step *= 5;
        if step == 0 {
            step = 10;
        }
    }
    let mut build = farm + inn + temple + caravan;
    build = build.min(2) + 1;
    habitat = (build * build + 1) * habitat * habitat + region.habitat / 4 + 50;
    habitat
        + town_development(region, &tables.terrains, defs)
            * (region.habitat + 800 + habitat + defs.city_pop / 2)
            / 100
}

fn town_development(
    region: &Region,
    terrains: &[crate::defs::TerrainDef],
    defs: &crate::defs::GameDefs,
) -> i32 {
    let base_development = region
        .terrain_type
        .and_then(|terrain| terrains.get(terrain))
        .map_or(1, |terrain| {
            let mut level = 1;
            let mut basedev = 1;
            let mut previous = 0;
            while level <= terrain.wages {
                previous += 1;
                basedev += 1;
                if previous > level {
                    level += 1;
                    previous = 0;
                }
            }
            (defs.maintenance_cost + basedev) / 2
        });
    (region.development - base_development).clamp(0, 100)
}

fn remove_town_lairs(loaded: &mut LoadedGame, region_id: RegionId) -> Result<(), EditError> {
    let Some(region) = loaded.game.region(region_id) else {
        return Err(EditError::Model(ModelError::MissingRegion(region_id)));
    };
    let object_ids = region.object_ids().to_vec();
    let tables = loaded.ruleset.tables();
    for object_id in object_ids {
        let Some(object) = loaded.game.object(object_id) else {
            continue;
        };
        if object.object_type == O_DUMMY as i32 {
            continue;
        }
        let Some(definition) = usize::try_from(object.object_type)
            .ok()
            .and_then(|object_type| tables.objects.get(object_type))
        else {
            continue;
        };
        if definition.monster != -1 && definition.flags & object_flags::CANENTER == 0 {
            loaded.game.empty_object_units(object_id)?;
            loaded.game.detach_region_object(region_id, object_id)?;
        }
    }
    Ok(())
}

fn setup_city_market(loaded: &mut LoadedGame, region_id: RegionId) -> Result<(), EditError> {
    let region = loaded
        .game
        .region(region_id)
        .ok_or(EditError::Model(ModelError::MissingRegion(region_id)))?
        .clone();
    let tables = loaded.ruleset.tables();
    let defs = loaded.ruleset.game_defs();
    let terrain_id = region
        .terrain_type
        .ok_or(EditError::InvalidTerrain(usize::MAX))?;
    let terrain = tables
        .terrains
        .get(terrain_id)
        .ok_or(EditError::InvalidTerrain(terrain_id))?;
    let race_item = usize::try_from(region.race)
        .ok()
        .and_then(|race| tables.item_families.items.get(race));
    let race_abbreviation = race_item.and_then(|item| item.abbreviation.as_deref());
    let locals = race_abbreviation
        .and_then(|abbreviation| {
            tables
                .item_families
                .races
                .iter()
                .find(|race| race.abbreviation.as_deref() == Some(abbreviation))
        })
        .or_else(|| {
            tables
                .item_families
                .races
                .iter()
                .find(|race| race.abbreviation.as_deref() == Some("SELF"))
        });
    let locals = locals.ok_or(EditError::InvalidRace(ItemId::Leaders))?;
    let population = region_population(&region);
    let city_max = defs.city_pop;
    let mut supply = vec![0; tables.item_families.items.len()];
    let mut demand = vec![0; tables.item_families.items.len()];
    let mut rare = vec![0; tables.item_families.items.len()];
    let mut antiques = vec![0; tables.item_families.items.len()];
    let mut trade_count = 0;
    let mut markets = Vec::new();

    for (index, item) in tables.item_families.items.iter().enumerate() {
        if item.flags & (ItemFlags::DISABLED | ItemFlags::NOMARKET) != 0
            || item.item_type & ItemTypeFlags::SHIP != 0
        {
            continue;
        }
        if item.item_type & ItemTypeFlags::TRADE != 0 {
            trade_count += 1;
        }
        if index == ItemId::Silver as usize
            || item.item_type & (ItemTypeFlags::MAN | ItemTypeFlags::LEADER) != 0
        {
            continue;
        }
        let can_produce_here = if item.production_inputs[0].item == -1 {
            terrain
                .products
                .iter()
                .any(|product| product.item == index as i32)
        } else {
            item.production_inputs.iter().all(|input| {
                terrain
                    .products
                    .iter()
                    .any(|product| product.item == input.item)
            })
        };
        let can_produce = can_produce_here
            && item.production_skill.as_ref().is_some_and(|skill| {
                locals
                    .skills
                    .iter()
                    .flatten()
                    .any(|race_skill| race_skill == skill)
            });
        let useful = can_use_item(item, locals, &tables.item_families);
        if item.item_type & ItemTypeFlags::NORMAL != 0 {
            if [ItemId::Grain, ItemId::Livestock, ItemId::Fish]
                .iter()
                .any(|food| *food as usize == index)
            {
                let mut amount = defs.city_market_normal_amt;
                let price = if defs.random_economy != 0 {
                    amount += loaded.rng.get_random(amount);
                    (item.base_price * (100 + loaded.rng.get_random(50))) / 100
                } else {
                    item.base_price
                };
                let mut cap = city_max * 3 / 4 - 5000;
                if cap < 0 {
                    cap = city_max / 2;
                }
                markets.push(make_market(
                    MarketType::Sell,
                    MarketSetup {
                        item: index,
                        price,
                        amount,
                        populations: (population, population + cap),
                        amounts: (amount, amount * 2),
                    },
                ));
            } else if index == ItemId::Food as usize {
                let mut amount = defs.city_market_normal_amt;
                let price = if defs.random_economy != 0 {
                    amount += loaded.rng.get_random(amount);
                    (item.base_price * (120 + loaded.rng.get_random(80))) / 100
                } else {
                    item.base_price
                };
                let mut cap = city_max * 3 / 4 - 5000;
                if cap < 0 {
                    cap = city_max / 2;
                }
                markets.push(make_market(
                    MarketType::Buy,
                    MarketSetup {
                        item: index,
                        price,
                        amount,
                        populations: (population, population + 2 * cap),
                        amounts: (amount, amount * 5),
                    },
                ));
            } else if item.production_inputs[0].item == -1 {
                if can_produce {
                    supply[index] = 4;
                }
                if !can_produce_here {
                    if item.item_type & ItemTypeFlags::MOUNT != 0 {
                        if item.production_skill.as_ref().is_some_and(|skill| {
                            locals
                                .skills
                                .iter()
                                .flatten()
                                .any(|race_skill| race_skill == skill)
                        }) {
                            demand[index] = 4;
                        }
                    } else if useful {
                        demand[index] = 4;
                    }
                }
            } else if useful {
                if can_produce {
                    supply[index] = 2;
                }
                if !can_produce_here {
                    demand[index] = 2;
                }
            }
        } else if defs.city_market_advanced_amt != 0
            && item.item_type & ItemTypeFlags::ADVANCED != 0
        {
            if useful {
                rare[index] = 4;
            }
            if item.hitch_item > 0 {
                rare[index] = 2;
            }
        } else if defs.city_market_magic_amt != 0 && item.item_type & ItemTypeFlags::MAGIC != 0 {
            antiques[index] = if useful { 4 } else { 1 };
            if item.hitch_item > 0 {
                antiques[index] = 2;
            }
        }
    }

    if defs.city_market_advanced_amt != 0 && loaded.rng.get_random(4) == 1 {
        let total: i32 = rare.iter().sum();
        let selected = choose_weighted(&mut loaded.rng, &rare, total);
        if let Some(index) = selected {
            let item = &tables.item_families.items[index];
            let mut amount = defs.city_market_advanced_amt;
            let price = if defs.random_economy != 0 {
                amount += loaded.rng.get_random(amount);
                (item.base_price * (100 + loaded.rng.get_random(50))) / 100
            } else {
                item.base_price
            };
            let mut cap = city_max * 3 / 4 - 5000;
            if cap < city_max / 2 {
                cap = city_max / 2;
            }
            let offset = city_max / 8;
            if cap + offset < city_max {
                markets.push(make_market(
                    MarketType::Sell,
                    MarketSetup {
                        item: index,
                        price,
                        amount: amount / 6,
                        populations: (population + cap + offset, population + city_max),
                        amounts: (0, amount),
                    },
                ));
            }
        }
    }
    if defs.city_market_magic_amt != 0 && loaded.rng.get_random(8) == 1 {
        let total: i32 = antiques.iter().sum();
        let selected = choose_weighted(&mut loaded.rng, &antiques, total);
        if let Some(index) = selected {
            let item = &tables.item_families.items[index];
            let mut amount = defs.city_market_magic_amt;
            let price = if defs.random_economy != 0 {
                amount += loaded.rng.get_random(amount);
                (item.base_price * (100 + loaded.rng.get_random(50))) / 100
            } else {
                item.base_price
            };
            let cap = (city_max * 3 / 4 - 5000).max(city_max / 2);
            let _offset = city_max / 20 + (city_max / 5) * 2;
            markets.push(make_market(
                MarketType::Sell,
                MarketSetup {
                    item: index,
                    price,
                    amount: amount / 6,
                    populations: (population + cap, population + city_max),
                    amounts: (0, amount),
                },
            ));
        }
    }
    let context = CityMarketContext {
        items: &tables.item_families.items,
        defs,
        city_max,
        population,
    };
    choose_normal_markets(loaded, context, &mut demand, 4, false, &mut markets);
    choose_normal_markets(loaded, context, &mut supply, 2, true, &mut markets);
    if trade_count > 0 {
        let _initial_buy1 = loaded.rng.get_random(trade_count);
        let mut buy2 = loaded.rng.get_random(trade_count);
        let mut sell1 = loaded.rng.get_random(trade_count);
        let mut sell2 = loaded.rng.get_random(trade_count);
        let mut buy1 = loaded.rng.get_random(trade_count);
        while buy1 == buy2 {
            buy2 = loaded.rng.get_random(trade_count);
        }
        while sell1 == buy1 || sell1 == buy2 {
            sell1 = loaded.rng.get_random(trade_count);
        }
        while sell2 == sell1 || sell2 == buy2 || sell2 == buy1 {
            sell2 = loaded.rng.get_random(trade_count);
        }
        let mut trade_buy = 0;
        let mut trade_sell = 0;
        for (index, item) in tables.item_families.items.iter().enumerate() {
            if item.flags & (ItemFlags::DISABLED | ItemFlags::NOMARKET) != 0 {
                continue;
            }
            if item.item_type & ItemTypeFlags::TRADE == 0 {
                continue;
            }
            let add_buy = buy1 == 0 || buy2 == 0;
            buy1 -= 1;
            buy2 -= 1;
            let add_sell = sell1 == 0 || sell2 == 0;
            sell1 -= 1;
            sell2 -= 1;
            if add_buy {
                let mut amount = defs.city_market_trade_amt;
                let price = if defs.random_economy != 0 {
                    amount += loaded.rng.get_random(amount);
                    if defs.more_profitable_trade_goods != 0 {
                        (item.base_price * (250 + loaded.rng.get_random(100))) / 100
                    } else {
                        (item.base_price * (150 + loaded.rng.get_random(50))) / 100
                    }
                } else {
                    item.base_price
                };
                let cap = city_max / 2;
                trade_sell += 1;
                let offset =
                    -(city_max / 20) + trade_sell * (trade_sell * trade_sell * city_max / 40);
                if cap + offset < city_max {
                    markets.push(make_market(
                        MarketType::Sell,
                        MarketSetup {
                            item: index,
                            price,
                            amount: amount / 5,
                            populations: (cap + population + offset, city_max + population),
                            amounts: (0, amount),
                        },
                    ));
                }
            }
            if add_sell {
                let mut amount = defs.city_market_trade_amt;
                let price = if defs.random_economy != 0 {
                    amount += loaded.rng.get_random(amount);
                    if defs.more_profitable_trade_goods != 0 {
                        (item.base_price * (100 + loaded.rng.get_random(90))) / 100
                    } else {
                        (item.base_price * (100 + loaded.rng.get_random(50))) / 100
                    }
                } else {
                    item.base_price
                };
                let cap = city_max / 2;
                let offset = trade_buy * (city_max / 6);
                trade_buy += 1;
                if cap + offset < city_max {
                    markets.push(make_market(
                        MarketType::Buy,
                        MarketSetup {
                            item: index,
                            price,
                            amount: amount / 6,
                            populations: (cap + population + offset, city_max + population),
                            amounts: (0, amount),
                        },
                    ));
                }
            }
        }
    }
    loaded
        .game
        .region_mut(region_id)
        .expect("region was checked before city-market generation")
        .markets = markets;
    Ok(())
}

fn can_use_item(
    item: &crate::defs::ItemDef,
    race: &crate::defs::ManDef,
    tables: &crate::defs::Tables,
) -> bool {
    if item.flags & ItemFlags::DISABLED != 0 {
        return false;
    }
    if item.item_type & (ItemTypeFlags::MOUNT | ItemTypeFlags::WEAPON | ItemTypeFlags::TOOL) != 0 {
        return true;
    }
    if item.item_type & ItemTypeFlags::ARMOR == 0 {
        return false;
    }
    let armor = item.abbreviation.as_deref().and_then(|abbreviation| {
        tables
            .armor
            .iter()
            .find(|armor| armor.abbreviation.as_deref() == Some(abbreviation))
    });
    let Some(armor) = armor else {
        return false;
    };
    if armor.saves[3] == 0 {
        return false;
    }
    let protection = armor.from_value / armor.saves[3];
    if protection > 4 {
        !race.skills.iter().flatten().any(|skill| skill == "COMB")
    } else if protection > 3 {
        true
    } else {
        !race.skills.iter().flatten().any(|skill| {
            ["SAIL", "HUNT", "STEA", "LBOW"]
                .iter()
                .any(|forbidden| skill == forbidden)
        })
    }
}

struct MarketSetup {
    item: usize,
    price: i32,
    amount: i32,
    populations: (i32, i32),
    amounts: (i32, i32),
}

fn make_market(market_type: MarketType, setup: MarketSetup) -> Market {
    Market {
        market_type,
        item: setup.item as i32,
        price: setup.price,
        amount: setup.amount,
        min_population: setup.populations.0,
        max_population: setup.populations.1,
        min_amount: setup.amounts.0,
        max_amount: setup.amounts.1,
        base_price: setup.price,
        activity: 0,
    }
}

fn choose_weighted(rng: &mut crate::rng::Rng, weights: &[i32], total: i32) -> Option<usize> {
    let choice = rng.get_random(total);
    let mut sum = 0;
    for (index, weight) in weights.iter().enumerate() {
        sum += weight;
        if choice < sum {
            return Some(index);
        }
    }
    None
}

#[derive(Clone, Copy)]
struct CityMarketContext<'a> {
    items: &'a [crate::defs::ItemDef],
    defs: &'a crate::defs::GameDefs,
    city_max: i32,
    population: i32,
}

fn choose_normal_markets(
    loaded: &mut LoadedGame,
    context: CityMarketContext<'_>,
    weights: &mut [i32],
    count: i32,
    supply: bool,
    markets: &mut Vec<Market>,
) {
    let mut remaining = count;
    let mut sum = 1;
    while remaining > 0 && sum > 0 {
        let total: i32 = weights.iter().sum();
        let choice = loaded.rng.get_random(total);
        sum = 0;
        let mut selected = None;
        for (index, weight) in weights.iter().enumerate() {
            sum += weight;
            if choice < sum {
                selected = Some(index);
                break;
            }
        }
        let Some(index) = selected else {
            continue;
        };
        let item = &context.items[index];
        let mut amount = context.defs.city_market_normal_amt;
        amount = weights[index] * amount / 4;
        let price = if context.defs.random_economy != 0 {
            amount += loaded.rng.get_random(amount);
            if supply {
                (item.base_price * (150 + loaded.rng.get_random(50))) / 100
            } else {
                (item.base_price * (100 + loaded.rng.get_random(50))) / 100
            }
        } else {
            item.base_price
        };
        let cap = context.city_max / 4;
        let offset = if supply {
            let offset = (3 - remaining) * context.city_max * 3 / 40;
            if weights[index] < 4 {
                offset + context.city_max / 20
            } else {
                offset
            }
        } else {
            -(context.city_max / 20) + ((5 - remaining) * context.city_max * 3 / 40)
        };
        if supply {
            markets.push(make_market(
                MarketType::Buy,
                MarketSetup {
                    item: index,
                    price,
                    amount: 0,
                    populations: (
                        context.population + cap + offset,
                        context.population + context.city_max,
                    ),
                    amounts: (0, amount),
                },
            ));
        } else {
            markets.push(make_market(
                MarketType::Sell,
                MarketSetup {
                    item: index,
                    price,
                    amount: amount / 6,
                    populations: (
                        context.population + cap + offset,
                        context.population + context.city_max,
                    ),
                    amounts: (0, amount),
                },
            ));
        }
        weights[index] = 0;
        remaining -= 1;
    }
}
