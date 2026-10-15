use std::fmt;

use crate::defs::{skills::lookup_skill, ItemFlags, ItemId};
use crate::model::{
    Attitude, Faction, FactionId, FactionItemShow, FactionSkillShow, Game, ModelError, ObjectId,
    RegionId, RegionLevelType, UnitType,
};
use crate::players::GAME_STATUS_NEW;
use crate::rng::Rng;
use crate::ruleset::Ruleset;
use crate::save::LoadedGame;
use crate::world::WorldGenerationError;

pub use crate::world::SurfaceGenerator;

const REVEAL_FACTION: i32 = 2;
const ATTITUDE_UNFRIENDLY: i32 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NewGameOptions {
    pub seed: i32,
    pub width: i32,
    pub height: i32,
    pub nexus_size: i32,
    pub generator: SurfaceGenerator,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NewGameError {
    InvalidSurfaceDimensions { width: i32, height: i32 },
    InvalidNexusSize(i32),
    WorldGeneration(WorldGenerationError),
    MissingStartRegion,
    MissingNexusArray,
    MissingDummy(RegionId),
    ObjectHasNoRegion(ObjectId),
    MissingFaction(&'static str),
    MissingTerrain(i32),
    MissingObjectDefinition(i32),
    MissingMonster(i32),
    MissingMonsterName(i32),
    MissingSkill(String),
    MissingVirtualMonsterFaction,
    Model(ModelError),
}

impl fmt::Display for NewGameError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSurfaceDimensions { width, height } => write!(
                formatter,
                "surface dimensions must be at least 8x8 and multiples of 8 (got {width}x{height})"
            ),
            Self::InvalidNexusSize(size) => write!(
                formatter,
                "nexus size must be 1 or a positive even number (got {size})"
            ),
            Self::WorldGeneration(error) => error.fmt(formatter),
            Self::MissingStartRegion => formatter.write_str("new faction has no starting region"),
            Self::MissingNexusArray => formatter.write_str("multi-hex nexus array is missing"),
            Self::MissingDummy(region) => {
                write!(formatter, "region {} has no dummy object", region.index())
            }
            Self::ObjectHasNoRegion(object) => {
                write!(formatter, "object {} has no region", object.index())
            }
            Self::MissingFaction(name) => write!(formatter, "new game is missing {name} faction"),
            Self::MissingTerrain(terrain) => {
                write!(formatter, "new game references missing terrain {terrain}")
            }
            Self::MissingObjectDefinition(object) => {
                write!(
                    formatter,
                    "new game references missing object definition {object}"
                )
            }
            Self::MissingMonster(item) => {
                write!(
                    formatter,
                    "new game has no monster definition for item {item}"
                )
            }
            Self::MissingMonsterName(item) => {
                write!(formatter, "new game monster item {item} has no name")
            }
            Self::MissingSkill(abbreviation) => {
                write!(formatter, "starting skill {abbreviation:?} is missing")
            }
            Self::MissingVirtualMonsterFaction => {
                formatter.write_str("virtual monsters require faction number 2")
            }
            Self::Model(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for NewGameError {}

impl From<ModelError> for NewGameError {
    fn from(error: ModelError) -> Self {
        Self::Model(error)
    }
}

impl From<WorldGenerationError> for NewGameError {
    fn from(error: WorldGenerationError) -> Self {
        Self::WorldGeneration(error)
    }
}

/// C++: `game.cpp` `Game::NewGame`; creates the world, then runs post-world setup.
pub fn create_new_game(
    ruleset: &'static dyn Ruleset,
    options: NewGameOptions,
) -> Result<LoadedGame, NewGameError> {
    if options.width < 8 || options.height < 8 || options.width % 8 != 0 || options.height % 8 != 0
    {
        return Err(NewGameError::InvalidSurfaceDimensions {
            width: options.width,
            height: options.height,
        });
    }
    if options.nexus_size <= 0 || (options.nexus_size > 1 && options.nexus_size % 2 != 0) {
        return Err(NewGameError::InvalidNexusSize(options.nexus_size));
    }

    let mut game = Game::default();
    game.game_status = GAME_STATUS_NEW;
    game.setup_unit_nums()?;
    let mut rng = Rng::new(options.seed);
    crate::world::create_world(
        &mut game,
        ruleset,
        &mut rng,
        options.seed,
        options.width,
        options.height,
        options.nexus_size,
        options.generator,
    )?;
    setup_new_game(&mut game, ruleset, &mut rng)?;

    Ok(LoadedGame {
        game,
        ruleset,
        seed: options.seed,
        rng,
    })
}

/// C++: `game.cpp` `Game::NewGame`; post-world faction setup, with later NPC phases in `npc`.
pub fn setup_new_game(
    game: &mut Game,
    ruleset: &'static dyn Ruleset,
    rng: &mut Rng,
) -> Result<(), NewGameError> {
    crate::npc::create_npc_factions(game, ruleset);
    let tables = ruleset.tables();
    if ruleset.game_defs().city_monsters_exist != 0 {
        crate::npc::create_city_monsters(game, ruleset, &tables)?;
    }
    if ruleset.game_defs().wandering_monsters_exist != 0 {
        crate::npc::grow_wandering_monsters(game, ruleset, &tables, 50, rng)?;
    }
    if ruleset.game_defs().lair_monsters_exist != 0 {
        crate::npc::grow_lair_monsters(game, ruleset, &tables, 50, rng)?;
        crate::npc::create_virtual_monsters(game, ruleset)?;
    }
    Ok(())
}

/// C++: `game.cpp` `Game::AddFaction`; creates a player faction and its optional starting leader.
pub fn add_faction(
    game: &mut Game,
    ruleset: &'static dyn Ruleset,
    no_leader: bool,
    start_region: Option<RegionId>,
    rng: &mut Rng,
) -> Result<FactionId, NewGameError> {
    let tables = ruleset.tables();
    let turn_number = game.turn_number();
    let spawn = if no_leader {
        None
    } else {
        let region = match start_region {
            Some(region) => region,
            None => default_start_region(game, ruleset, rng)?,
        };
        let object_id = game
            .get_dummy(region)
            .ok_or(NewGameError::MissingDummy(region))?;
        Some((region, object_id))
    };
    let mut faction = Faction::new(game.faction_seq, ruleset.faction_types());
    faction.last_orders = turn_number;
    faction.start_turn = turn_number;
    faction.region = start_region;
    faction.start_location = start_region;
    faction.no_start_leader = if no_leader { 1 } else { 0 };
    let setup = ruleset.starting_faction_setup(turn_number);
    faction.unclaimed = ruleset
        .game_defs()
        .start_money
        .wrapping_add(turn_number.wrapping_mul(setup.unclaimed_per_turn));

    let faction_id = game.add_faction(faction);
    if no_leader {
        game.faction_seq = game.faction_seq.wrapping_add(1);
        return Ok(faction_id);
    }

    let (_, object_id) = spawn.expect("leader factions select a starting region");
    let default_move_points =
        ruleset.game_defs().phased_move_offset % ruleset.game_defs().max_speed;
    let unit_id = game.get_new_unit(Some(faction_id), 0, default_move_points)?;
    game.attach_unit(object_id, unit_id)?;
    let monster_faction_num = game
        .monster_faction
        .and_then(|id| game.faction(id))
        .map(|faction| faction.num);

    {
        let unit = game
            .unit_mut(unit_id)
            .expect("new unit remains in the game");
        unit.items.set_num(ItemId::Leaders, 1);
        unit.unit_type = UnitType::Mage;
        unit.reveal = REVEAL_FACTION;
        unit.flags = (unit.flags | setup.set_flags) & !setup.clear_flags;

        if ruleset.game_defs().upkeep_minimum_food > 0 {
            let food = [
                (ItemId::Food, 6),
                (ItemId::Fish, 6),
                (ItemId::Livestock, 6),
                (ItemId::Grain, 2),
            ]
            .into_iter()
            .find(|(item, _)| {
                tables
                    .item_families
                    .items
                    .get(item.index())
                    .is_some_and(|definition| definition.flags & ItemFlags::DISABLED == 0)
            });
            if let Some((item, amount)) = food {
                unit.items.set_num(item, amount);
                unit.items.set_num(ItemId::Silver, 10);
            }
        }
    }
    {
        let faction = game
            .faction_mut(faction_id)
            .expect("new faction remains in the game");
        discover_item(faction, ItemId::Leaders);
        if ruleset.game_defs().upkeep_minimum_food > 0 {
            let food = [ItemId::Food, ItemId::Fish, ItemId::Livestock, ItemId::Grain]
                .into_iter()
                .find(|item| {
                    tables
                        .item_families
                        .items
                        .get(item.index())
                        .is_some_and(|definition| definition.flags & ItemFlags::DISABLED == 0)
                });
            if let Some(item) = food {
                discover_item(faction, item);
            }
        }
    }

    let studies = setup.studies;
    for (abbreviation, days) in studies {
        let skill_id = lookup_skill(&tables.skills.skills, abbreviation)
            .ok_or_else(|| NewGameError::MissingSkill(abbreviation.to_owned()))?;
        {
            let unit = game
                .unit_mut(unit_id)
                .expect("new unit remains in the game");
            unit.skills.set_days(
                skill_id,
                unit.skills.get_days(skill_id).wrapping_add(days as u32),
            );
        }
        let level = real_skill_level(
            game.unit(unit_id).expect("new unit remains in the game"),
            skill_id,
        );
        let faction = game
            .faction_mut(faction_id)
            .expect("new faction remains in the game");
        let shown = faction.skills.get_days(skill_id) as i32;
        for next_level in shown + 1..=level {
            faction.skills.set_days(skill_id, next_level as u32);
            faction.shows.push(FactionSkillShow {
                skill_type: skill_id,
                level: next_level,
            });
        }
    }

    if let Some(monster_num) = monster_faction_num {
        game.faction_mut(faction_id)
            .expect("new faction remains in the game")
            .attitudes
            .push(Attitude {
                faction_num: monster_num,
                attitude: ATTITUDE_UNFRIENDLY,
            });
    }
    game.faction_seq = game.faction_seq.wrapping_add(1);
    Ok(faction_id)
}

fn default_start_region(
    game: &Game,
    ruleset: &'static dyn Ruleset,
    rng: &mut Rng,
) -> Result<RegionId, NewGameError> {
    if ruleset.game_defs().multi_hex_nexus == 0 {
        return game
            .region_ids()
            .first()
            .copied()
            .ok_or(NewGameError::MissingStartRegion);
    }

    let (array_index, array) = game
        .region_arrays
        .iter()
        .enumerate()
        .find(|(_, array)| array.level_type == RegionLevelType::Nexus)
        .ok_or(NewGameError::MissingNexusArray)?;
    loop {
        let x = rng.get_random(array.x);
        let y = rng.get_random(array.y);
        if let Some(region) = game.get_region_at(x, y, array_index as i32) {
            return Ok(region);
        }
    }
}

fn real_skill_level(unit: &crate::model::Unit, skill_id: usize) -> i32 {
    let men = unit.items.get_num(ItemId::Leaders);
    if men == 0 {
        return 0;
    }
    let mut days = unit.skills.get_days(skill_id) as i32 / men;
    let mut threshold = 30;
    let mut level = 0;
    while days >= threshold {
        level += 1;
        days -= threshold;
        threshold += 30;
    }
    level
}

fn discover_item(faction: &mut Faction, item: ItemId) {
    if faction.items.get_num(item) == 0 {
        faction.items.set_num(item, 2);
        faction.item_shows.push(FactionItemShow {
            item_type: item,
            full: true,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn real_skill_level_uses_increasing_thirty_day_thresholds() {
        let mut unit = crate::model::Unit::default();
        unit.items.set_num(ItemId::Leaders, 1);
        unit.skills.set_days(0, 180);

        assert_eq!(real_skill_level(&unit, 0), 3);
    }
}
