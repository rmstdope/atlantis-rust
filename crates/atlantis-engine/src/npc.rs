use crate::defs::{
    objects::object_flags, terrains::R_NEXUS, ItemFlags, ItemId, RulesetTables, TerrainDef,
};
use crate::model::{
    Faction, FactionId, Game, GuardMode, ObjectId, RegionId, Unit, UnitId, UnitType,
};
use crate::newgame::NewGameError;
use crate::rng::Rng;
use crate::ruleset::Ruleset;
use crate::text::is_equal;

const REVEAL_FACTION: i32 = 2;
const FLAG_BEHIND: i32 = 0x0001;
const FLAG_HOLDING: i32 = 0x0008;

/// C++: `game.cpp` `Game::CreateNPCFactions`; guard faction precedes the creature faction.
pub fn create_npc_factions(game: &mut Game, ruleset: &'static dyn Ruleset) {
    let defs = ruleset.game_defs();
    game.guard_faction = None;
    game.monster_faction = None;
    if defs.city_monsters_exist != 0 {
        let mut faction = Faction::new(game.faction_seq, ruleset.faction_types());
        faction.name = Some(format!("The Guardsmen ({})", faction.num));
        faction.last_orders = 0;
        for (_, value) in &mut faction.faction_types {
            *value = -1;
        }
        game.guard_faction = Some(game.add_faction(faction));
        game.faction_seq = game.faction_seq.wrapping_add(1);
    }

    if defs.lair_monsters_exist != 0 || defs.wandering_monsters_exist != 0 {
        let mut faction = Faction::new(game.faction_seq, ruleset.faction_types());
        faction.name = Some(format!("Creatures ({})", faction.num));
        faction.last_orders = 0;
        for (_, value) in &mut faction.faction_types {
            *value = -1;
        }
        game.monster_faction = Some(game.add_faction(faction));
        game.faction_seq = game.faction_seq.wrapping_add(1);
    }
}

/// C++: `npc.cpp` `Game::CreateCityMons`.
pub fn create_city_monsters(
    game: &mut Game,
    ruleset: &'static dyn Ruleset,
    tables: &RulesetTables,
) -> Result<(), NewGameError> {
    if ruleset.game_defs().city_monsters_exist == 0 {
        return Ok(());
    }
    let regions = game.region_ids().to_vec();
    for region_id in regions {
        let region = game.region(region_id).expect("ordered region exists");
        if region.terrain_type == Some(R_NEXUS)
            || is_starting_city(region, ruleset)
            || region.town.is_some()
        {
            create_city_monster(game, ruleset, tables, region_id)?;
        }
    }
    Ok(())
}

/// C++: `npc.cpp` `Game::GrowWMons`; preserves the original block bounds and retry draws.
pub fn grow_wandering_monsters(
    game: &mut Game,
    ruleset: &'static dyn Ruleset,
    tables: &RulesetTables,
    rate: i32,
    rng: &mut Rng,
) -> Result<(), NewGameError> {
    if ruleset.game_defs().wandering_monsters_exist == 0 {
        return Ok(());
    }
    let monster_faction = game
        .monster_faction
        .ok_or(NewGameError::MissingFaction("creature"))?;
    let dimensions = game
        .region_arrays
        .iter()
        .map(|array| (array.x, array.y))
        .collect::<Vec<_>>();

    for (level, (width, height)) in dimensions.into_iter().enumerate() {
        for x_section in (0..width).step_by(8) {
            for y_section in (0..height).step_by(16) {
                let mut monsters = 0;
                let mut wanted = 0;

                for x in 0..8 {
                    if x + x_section > width {
                        break;
                    }
                    for y in (0..16).step_by(2) {
                        if y + y_section > height {
                            break;
                        }
                        let Some(region_id) =
                            game.get_region_at(x + x_section, y + y_section + x % 2, level as i32)
                        else {
                            continue;
                        };
                        if game
                            .region(region_id)
                            .expect("region array points to region")
                            .zloc
                            != level as i32
                            || is_guarded(game, region_id)
                        {
                            continue;
                        }
                        let region = game
                            .region(region_id)
                            .expect("region array points to region");
                        let terrain = terrain_for(region.terrain_type, tables)?;
                        monsters += wandering_monster_count(game, region_id);
                        if has_available_wandering_monster(terrain, tables) {
                            wanted += terrain.wandering_monster_frequency;
                        }
                    }
                }

                wanted /= 10;
                wanted -= monsters;
                wanted = (wanted * rate + rng.get_random(100)) / 100;

                if wanted > 0 {
                    let mut placed = 0;
                    for _ in 0..1000 {
                        if placed >= wanted {
                            break;
                        }
                        let x = rng.get_random(8);
                        let mut y = rng.get_random(16);
                        if y % 2 == 1 {
                            y -= 1;
                        }
                        let Some(region_id) =
                            game.get_region_at(x + x_section, y + y_section + x % 2, level as i32)
                        else {
                            continue;
                        };
                        if game
                            .region(region_id)
                            .expect("region array points to region")
                            .zloc
                            == level as i32
                            && !is_guarded(game, region_id)
                            && make_wandering_monster(
                                game,
                                ruleset,
                                tables,
                                monster_faction,
                                region_id,
                                rng,
                            )?
                        {
                            placed += 1;
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

/// C++: `npc.cpp` `Game::GrowLMons`; chance is drawn only for empty, growable lairs.
pub fn grow_lair_monsters(
    game: &mut Game,
    ruleset: &'static dyn Ruleset,
    tables: &RulesetTables,
    rate: i32,
    rng: &mut Rng,
) -> Result<(), NewGameError> {
    if ruleset.game_defs().lair_monsters_exist == 0 {
        return Ok(());
    }
    let monster_faction = game
        .monster_faction
        .ok_or(NewGameError::MissingFaction("creature"))?;
    for region_id in game.region_ids().to_vec() {
        if is_guarded(game, region_id) {
            continue;
        }
        let object_ids = game
            .region(region_id)
            .expect("ordered region exists")
            .object_ids()
            .to_vec();
        for object_id in object_ids {
            let object = game.object(object_id).expect("ordered object exists");
            if !object.unit_ids().is_empty() {
                continue;
            }
            let object_type = usize::try_from(object.object_type)
                .map_err(|_| NewGameError::MissingObjectDefinition(object.object_type))?;
            let definition = tables
                .objects
                .get(object_type)
                .ok_or(NewGameError::MissingObjectDefinition(object.object_type))?;
            if definition.monster != -1
                && definition.flags & object_flags::NOMONSTERGROWTH == 0
                && rng.get_random(100) < rate
            {
                make_lair_monster(game, ruleset, tables, monster_faction, object_id, rng)?;
            }
        }
    }
    Ok(())
}

/// C++: ruleset `monsters.cpp` `Game::CreateVMons`.
pub fn create_virtual_monsters(
    game: &mut Game,
    ruleset: &'static dyn Ruleset,
) -> Result<(), NewGameError> {
    if ruleset.game_defs().lair_monsters_exist == 0 {
        return Ok(());
    }
    let Some(virtual_monster) = ruleset.virtual_monster() else {
        return Ok(());
    };
    let monster_faction = game
        .faction_ids()
        .iter()
        .copied()
        .find(|id| game.faction(*id).is_some_and(|faction| faction.num == 2));

    for region_id in game.region_ids().to_vec() {
        let object_ids = game
            .region(region_id)
            .expect("ordered region exists")
            .object_ids()
            .to_vec();
        for object_id in object_ids {
            if game
                .object(object_id)
                .expect("ordered object exists")
                .object_type
                != virtual_monster.object_type
            {
                continue;
            }
            let monster_faction =
                monster_faction.ok_or(NewGameError::MissingVirtualMonsterFaction)?;
            make_monster_unit(
                game,
                ruleset,
                monster_faction,
                object_id,
                virtual_monster.name,
                virtual_monster.item,
                virtual_monster.count,
            )?;
        }
    }
    Ok(())
}

/// C++: `game.cpp` `Game::CreateCityMon`.
fn create_city_monster(
    game: &mut Game,
    ruleset: &'static dyn Ruleset,
    tables: &RulesetTables,
    region_id: RegionId,
) -> Result<(), NewGameError> {
    let faction = game
        .guard_faction
        .ok_or(NewGameError::MissingFaction("guard"))?;
    let defs = ruleset.game_defs();
    let region = game.region(region_id).expect("ordered region exists");
    let nexus = region.terrain_type == Some(R_NEXUS);
    let starting_city = is_starting_city(region, ruleset);
    let advanced_city = nexus || starting_city;
    let invulnerable = defs.safe_start_cities != 0 && starting_city || nexus;
    let skill_level = if advanced_city {
        3
    } else {
        town_type(region, defs.city_pop) + 1
    };
    let count = if advanced_city {
        defs.amt_start_city_guards
    } else {
        defs.city_guard.wrapping_mul(skill_level)
    };
    let object_id = game
        .get_dummy(region_id)
        .ok_or(NewGameError::MissingDummy(region_id))?;

    let guard = new_unit(game, faction, ruleset)?;
    {
        let unit = game.unit_mut(guard).expect("new unit remains in the game");
        unit.name = Some(format!("City Guard ({})", unit.num));
        unit.unit_type = UnitType::Guard;
        unit.guard = GuardMode::Guard;
        unit.reveal = REVEAL_FACTION;
        unit.items.set_num(ItemId::Leaders, count);
        unit.items.set_num(ItemId::Sword, count);
        if invulnerable {
            unit.items.set_num(ItemId::Amuletofi, count);
        }
        unit.items
            .set_num(ItemId::Silver, count.wrapping_mul(defs.guard_money));
        if advanced_city && defs.start_city_guards_plate != 0 && defs.leaders_exist != 0 {
            unit.items.set_num(ItemId::PlateArmor, count);
        }
        set_unit_skill(unit, tables, "COMB", skill_level, count)?;
        if advanced_city {
            set_unit_skill(unit, tables, "OBSE", 10, count)?;
            if defs.start_city_tactics != 0 {
                set_unit_skill(unit, tables, "TACT", defs.start_city_tactics, count)?;
            }
        } else {
            set_unit_skill(unit, tables, "OBSE", skill_level, count)?;
        }
        unit.flags |= FLAG_HOLDING;
    }
    game.attach_unit(object_id, guard)?;

    if advanced_city && defs.start_city_mages != 0 {
        let mage = new_unit(game, faction, ruleset)?;
        {
            let unit = game.unit_mut(mage).expect("new unit remains in the game");
            unit.name = Some(format!("City Mage ({})", unit.num));
            unit.unit_type = UnitType::GuardMage;
            unit.reveal = REVEAL_FACTION;
            unit.items.set_num(ItemId::Leaders, 1);
            if invulnerable {
                unit.items.set_num(ItemId::Amuletofi, 1);
            }
            unit.items.set_num(ItemId::Silver, defs.guard_money);
            set_unit_skill(unit, tables, "FORC", defs.start_city_mages, 1)?;
            let fire = set_unit_skill(unit, tables, "FIRE", defs.start_city_mages, 1)?;
            if defs.start_city_tactics != 0 {
                set_unit_skill(unit, tables, "TACT", defs.start_city_tactics, 1)?;
            }
            unit.combat = fire as i32;
            unit.flags |= FLAG_BEHIND | FLAG_HOLDING;
        }
        game.attach_unit(object_id, mage)?;
    }
    Ok(())
}

/// C++: `npc.cpp` `Game::MakeWMon`.
fn make_wandering_monster(
    game: &mut Game,
    ruleset: &'static dyn Ruleset,
    tables: &RulesetTables,
    faction: FactionId,
    region_id: RegionId,
    rng: &mut Rng,
) -> Result<bool, NewGameError> {
    let region = game
        .region(region_id)
        .expect("region array points to region");
    let terrain = terrain_for(region.terrain_type, tables)?;
    if terrain.wandering_monster_frequency == 0 {
        return Ok(false);
    }
    let mut item = terrain.small_monster;
    if rng.get_random(2) != 0 && terrain.humanoid != -1 {
        item = terrain.humanoid;
    }
    if terrain.big_monster != -1 && rng.get_random(8) == 0 {
        item = terrain.big_monster;
    }
    let Some(item_id) = available_monster(item, tables) else {
        return Ok(false);
    };
    let definition = monster_for_item(item_id, tables)?;
    let name = definition
        .name
        .as_deref()
        .ok_or(NewGameError::MissingMonsterName(item_id.index() as i32))?;
    let count = (definition.number + rng.get_random(definition.number) + 1) / 2;
    let object_id = game
        .get_dummy(region_id)
        .ok_or(NewGameError::MissingDummy(region_id))?;
    make_monster_unit(game, ruleset, faction, object_id, name, item_id, count)?;
    Ok(true)
}

/// C++: `npc.cpp` `Game::MakeLMon`.
fn make_lair_monster(
    game: &mut Game,
    ruleset: &'static dyn Ruleset,
    tables: &RulesetTables,
    faction: FactionId,
    object_id: ObjectId,
    rng: &mut Rng,
) -> Result<(), NewGameError> {
    let object = game.object(object_id).expect("ordered object exists");
    let object_type = usize::try_from(object.object_type)
        .map_err(|_| NewGameError::MissingObjectDefinition(object.object_type))?;
    let object_definition = tables
        .objects
        .get(object_type)
        .ok_or(NewGameError::MissingObjectDefinition(object.object_type))?;
    if object_definition.flags & object_flags::NOMONSTERGROWTH != 0 {
        return Ok(());
    }
    let region_id = object
        .region
        .ok_or(NewGameError::ObjectHasNoRegion(object_id))?;
    let mut item = item_from_index(object_definition.monster);
    if item == Some(ItemId::Trent) || item == Some(ItemId::Centaur) {
        let terrain = terrain_for(
            game.region(region_id)
                .expect("object region exists")
                .terrain_type,
            tables,
        )?;
        item = item_from_index(if item == Some(ItemId::Trent) {
            terrain.big_monster
        } else {
            terrain.humanoid
        });
    }
    let Some(item_id) = item else {
        return Ok(());
    };
    if !item_is_enabled(item_id, tables) {
        return Ok(());
    }

    let initial_definition = monster_for_item(item_id, tables)?;
    let unit_id = new_unit(game, faction, ruleset)?;
    match item_id {
        ItemId::Imp => {
            initialize_monster_unit(
                game,
                unit_id,
                "Demons",
                ItemId::Imp,
                rng.get_random(initial_definition.number + 1),
            );
            set_extra_monster(
                game,
                unit_id,
                tables,
                ItemId::Demon,
                rng.get_random(monster_number(tables, ItemId::Demon)? + 1),
            )?;
            set_extra_monster(
                game,
                unit_id,
                tables,
                ItemId::Devil,
                rng.get_random(monster_number(tables, ItemId::Devil)? + 1),
            )?;
        }
        ItemId::Skeleton => {
            initialize_monster_unit(
                game,
                unit_id,
                "Undead",
                ItemId::Skeleton,
                rng.get_random(initial_definition.number + 1),
            );
            set_extra_monster(
                game,
                unit_id,
                tables,
                ItemId::Undead,
                rng.get_random(monster_number(tables, ItemId::Undead)? + 1),
            )?;
            set_extra_monster(
                game,
                unit_id,
                tables,
                ItemId::Lich,
                rng.get_random(monster_number(tables, ItemId::Lich)? + 1),
            )?;
        }
        ItemId::Magicians => {
            initialize_monster_unit(
                game,
                unit_id,
                "Evil Mages",
                ItemId::Magicians,
                (initial_definition.number + rng.get_random(initial_definition.number) + 1) / 2,
            );
            set_extra_monster(
                game,
                unit_id,
                tables,
                ItemId::Sorcerers,
                rng.get_random(monster_number(tables, ItemId::Sorcerers)? + 1),
            )?;
            set_guard_and_flags(game, unit_id, GuardMode::None, FLAG_BEHIND);
            game.attach_unit(object_id, unit_id)?;

            let warriors = ItemId::Warriors;
            let warriors_count = {
                let number = monster_number(tables, warriors)?;
                (number + rng.get_random(number) + 1) / 2
            };
            let warrior_unit = new_unit(game, faction, ruleset)?;
            initialize_monster_unit(
                game,
                warrior_unit,
                monster_name(tables, warriors)?,
                warriors,
                warriors_count,
            );
            set_guard_and_flags(game, warrior_unit, GuardMode::None, 0);
            game.attach_unit(object_id, warrior_unit)?;
            return Ok(());
        }
        ItemId::Darkmage => {
            initialize_monster_unit(
                game,
                unit_id,
                "Dark Mages",
                ItemId::Darkmage,
                rng.get_random(initial_definition.number) + 1,
            );
            set_extra_monster(
                game,
                unit_id,
                tables,
                ItemId::Magicians,
                (monster_number(tables, ItemId::Magicians)?
                    + rng.get_random(monster_number(tables, ItemId::Magicians)?)
                    + 1)
                    / 2,
            )?;
            set_extra_monster(
                game,
                unit_id,
                tables,
                ItemId::Sorcerers,
                rng.get_random(monster_number(tables, ItemId::Sorcerers)? + 1),
            )?;
            set_extra_monster(
                game,
                unit_id,
                tables,
                ItemId::Darkmage,
                rng.get_random(monster_number(tables, ItemId::Darkmage)? + 1),
            )?;
            set_guard_and_flags(game, unit_id, GuardMode::None, FLAG_BEHIND);
            game.attach_unit(object_id, unit_id)?;

            let drow = ItemId::Drow;
            let drow_count = {
                let number = monster_number(tables, drow)?;
                (number + rng.get_random(number) + 1) / 2
            };
            let drow_unit = new_unit(game, faction, ruleset)?;
            initialize_monster_unit(
                game,
                drow_unit,
                monster_name(tables, drow)?,
                drow,
                drow_count,
            );
            set_guard_and_flags(game, drow_unit, GuardMode::None, 0);
            game.attach_unit(object_id, drow_unit)?;
            return Ok(());
        }
        ItemId::Illyrthid => {
            initialize_monster_unit(
                game,
                unit_id,
                monster_name(tables, ItemId::Illyrthid)?,
                ItemId::Illyrthid,
                (initial_definition.number + rng.get_random(initial_definition.number) + 1) / 2,
            );
            set_guard_and_flags(game, unit_id, GuardMode::None, FLAG_BEHIND);
            game.attach_unit(object_id, unit_id)?;

            let skeleton_count = rng.get_random(monster_number(tables, ItemId::Skeleton)? + 1);
            let undead_count = rng.get_random(monster_number(tables, ItemId::Undead)? + 1);
            let undead_unit = new_unit(game, faction, ruleset)?;
            initialize_monster_unit(
                game,
                undead_unit,
                "Undead",
                ItemId::Skeleton,
                skeleton_count,
            );
            set_extra_monster(game, undead_unit, tables, ItemId::Undead, undead_count)?;
            set_guard_and_flags(game, undead_unit, GuardMode::None, 0);
            game.attach_unit(object_id, undead_unit)?;
            return Ok(());
        }
        ItemId::StormGiant => {
            let mut chosen = item_id;
            let mut definition = initial_definition;
            if rng.get_random(3) < 1 {
                chosen = ItemId::CloudGiant;
                definition = monster_for_item(chosen, tables)?;
            }
            initialize_monster_unit(
                game,
                unit_id,
                monster_name(tables, chosen)?,
                chosen,
                (definition.number + rng.get_random(definition.number) + 1) / 2,
            );
        }
        _ => initialize_monster_unit(
            game,
            unit_id,
            monster_name(tables, item_id)?,
            item_id,
            (initial_definition.number + rng.get_random(initial_definition.number) + 1) / 2,
        ),
    }
    game.attach_unit(object_id, unit_id)?;
    Ok(())
}

/// C++: `unit.cpp` `Unit::MakeWMon` and `Unit::MoveUnit`.
fn make_monster_unit(
    game: &mut Game,
    ruleset: &'static dyn Ruleset,
    faction: FactionId,
    object_id: ObjectId,
    name: &str,
    item: ItemId,
    count: i32,
) -> Result<UnitId, NewGameError> {
    let unit_id = new_unit(game, faction, ruleset)?;
    initialize_monster_unit(game, unit_id, name, item, count);
    game.attach_unit(object_id, unit_id)?;
    Ok(unit_id)
}

fn initialize_monster_unit(game: &mut Game, unit_id: UnitId, name: &str, item: ItemId, count: i32) {
    let unit = game
        .unit_mut(unit_id)
        .expect("new unit remains in the game");
    unit.name = Some(format!("{name} ({})", unit.num));
    unit.unit_type = UnitType::WanderingMonster;
    unit.items.set_num(item, count);
    unit.guard = GuardMode::Avoid;
    unit.flags |= FLAG_HOLDING;
}

fn set_extra_monster(
    game: &mut Game,
    unit_id: UnitId,
    tables: &RulesetTables,
    item: ItemId,
    count: i32,
) -> Result<(), NewGameError> {
    monster_for_item(item, tables)?;
    game.unit_mut(unit_id)
        .expect("new unit remains in the game")
        .items
        .set_num(item, count);
    Ok(())
}

fn set_guard_and_flags(game: &mut Game, unit_id: UnitId, guard: GuardMode, flags: i32) {
    let unit = game
        .unit_mut(unit_id)
        .expect("new unit remains in the game");
    unit.guard = guard;
    unit.flags |= flags;
}

fn new_unit(
    game: &mut Game,
    faction: FactionId,
    ruleset: &'static dyn Ruleset,
) -> Result<UnitId, NewGameError> {
    let defs = ruleset.game_defs();
    Ok(game.get_new_unit(Some(faction), 0, defs.phased_move_offset % defs.max_speed)?)
}

/// C++: `unit.cpp` `Unit::SetSkill`.
fn set_unit_skill(
    unit: &mut Unit,
    tables: &RulesetTables,
    abbreviation: &str,
    level: i32,
    men: i32,
) -> Result<usize, NewGameError> {
    let skill_id = crate::defs::skills::lookup_skill(&tables.skills.skills, abbreviation)
        .ok_or_else(|| NewGameError::MissingSkill(abbreviation.to_owned()))?;
    unit.skills
        .set_days(skill_id, days_by_level(level).wrapping_mul(men) as u32);
    Ok(skill_id)
}

/// C++: `skills.cpp` `GetDaysByLevel`.
fn days_by_level(level: i32) -> i32 {
    let mut days: i32 = 0;
    for value in 1..=level {
        days = days.wrapping_add(value.wrapping_mul(30));
    }
    days
}

/// C++: `aregion.cpp` `ARegion::IsStartingCity`.
fn is_starting_city(region: &crate::model::Region, ruleset: &'static dyn Ruleset) -> bool {
    region
        .town
        .as_ref()
        .is_some_and(|town| town.population >= ruleset.game_defs().city_pop.wrapping_mul(120) / 100)
}

/// C++: `economy.cpp` `TownInfo::TownType`.
pub(crate) fn town_type(region: &crate::model::Region, city_population: i32) -> i32 {
    let town = region.town.as_ref().expect("town type requires a town");
    let prestige = town
        .population
        .wrapping_mul(town.development.wrapping_add(220))
        / 270;
    if prestige < city_population / 4 {
        0
    } else if prestige < city_population.wrapping_mul(4) / 5 {
        1
    } else {
        2
    }
}

fn terrain_for(
    terrain_id: Option<usize>,
    tables: &RulesetTables,
) -> Result<&TerrainDef, NewGameError> {
    terrain_id
        .and_then(|index| tables.terrains.get(index))
        .ok_or(NewGameError::MissingTerrain(
            terrain_id.map_or(-1, |index| index as i32),
        ))
}

fn is_guarded(game: &Game, region_id: RegionId) -> bool {
    game.region(region_id)
        .expect("ordered region exists")
        .object_ids()
        .iter()
        .filter_map(|object_id| game.object(*object_id))
        .flat_map(|object| object.unit_ids())
        .filter_map(|unit_id| game.unit(*unit_id))
        .any(|unit| unit.guard == GuardMode::Guard)
}

fn wandering_monster_count(game: &Game, region_id: RegionId) -> i32 {
    game.region(region_id)
        .expect("ordered region exists")
        .object_ids()
        .iter()
        .filter_map(|object_id| game.object(*object_id))
        .flat_map(|object| object.unit_ids())
        .filter_map(|unit_id| game.unit(*unit_id))
        .filter(|unit| unit.unit_type == UnitType::WanderingMonster)
        .count() as i32
}

fn has_available_wandering_monster(terrain: &TerrainDef, tables: &RulesetTables) -> bool {
    [terrain.small_monster, terrain.big_monster, terrain.humanoid]
        .into_iter()
        .any(|item| available_monster(item, tables).is_some())
}

fn available_monster(item: i32, tables: &RulesetTables) -> Option<ItemId> {
    let item = item_from_index(item)?;
    item_is_enabled(item, tables).then_some(item)
}

fn item_is_enabled(item: ItemId, tables: &RulesetTables) -> bool {
    tables
        .item_families
        .items
        .get(item.index())
        .is_some_and(|definition| definition.flags & ItemFlags::DISABLED == 0)
}

fn item_from_index(item: i32) -> Option<ItemId> {
    usize::try_from(item).ok().and_then(ItemId::from_index)
}

fn monster_for_item(
    item: ItemId,
    tables: &RulesetTables,
) -> Result<&crate::defs::MonDef, NewGameError> {
    let item_definition = tables
        .item_families
        .items
        .get(item.index())
        .ok_or(NewGameError::MissingMonster(item.index() as i32))?;
    let abbreviation = item_definition
        .abbreviation
        .as_deref()
        .ok_or(NewGameError::MissingMonster(item.index() as i32))?;
    tables
        .item_families
        .monsters
        .iter()
        .find(|monster| {
            monster
                .abbreviation
                .as_deref()
                .is_some_and(|value| is_equal(value, abbreviation))
        })
        .ok_or(NewGameError::MissingMonster(item.index() as i32))
}

fn monster_number(tables: &RulesetTables, item: ItemId) -> Result<i32, NewGameError> {
    Ok(monster_for_item(item, tables)?.number)
}

fn monster_name(tables: &RulesetTables, item: ItemId) -> Result<&str, NewGameError> {
    monster_for_item(item, tables)?
        .name
        .as_deref()
        .ok_or(NewGameError::MissingMonsterName(item.index() as i32))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defs::{
        objects::{O_BKEEP, O_MAGETOWER},
        terrains::{R_PLAIN, R_TUNDRA},
        RulesetTables,
    };
    use crate::model::{Object, Region, RegionArray, RegionLevelType, TownInfo};

    #[test]
    fn starting_city_and_town_types_match_original_integer_arithmetic() {
        let ruleset = <dyn Ruleset>::from_name("standard").expect("standard ruleset");
        let mut region = Region::default();
        region.terrain_type = Some(R_PLAIN);
        region.town = Some(TownInfo {
            population: 24_000,
            development: 50,
            ..TownInfo::default()
        });
        assert!(is_starting_city(&region, ruleset));
        assert_eq!(town_type(&region, ruleset.game_defs().city_pop), 2);

        region.town.as_mut().unwrap().population = 100;
        assert_eq!(town_type(&region, ruleset.game_defs().city_pop), 0);
    }

    #[test]
    fn empty_map_setup_preserves_guard_then_creature_faction_order() {
        let ruleset = <dyn Ruleset>::from_name("standard").expect("standard ruleset");
        let mut game = Game::default();
        create_npc_factions(&mut game, ruleset);

        assert_eq!(game.faction(game.guard_faction.unwrap()).unwrap().num, 1);
        assert_eq!(game.faction(game.monster_faction.unwrap()).unwrap().num, 2);
    }

    #[test]
    fn empty_region_array_does_not_create_wandering_monsters() {
        let ruleset = <dyn Ruleset>::from_name("standard").expect("standard ruleset");
        let mut game = Game::default();
        create_npc_factions(&mut game, ruleset);
        let array = RegionArray::new(2, 2, RegionLevelType::Surface).unwrap();
        game.add_region_array(array);

        grow_wandering_monsters(
            &mut game,
            ruleset,
            &RulesetTables::default(),
            50,
            &mut Rng::new(1),
        )
        .unwrap();
    }

    #[test]
    fn city_guard_marks_its_region_guarded_and_has_original_equipment_order() {
        let ruleset = <dyn Ruleset>::from_name("standard").expect("standard ruleset");
        let tables = ruleset.tables();
        let mut game = Game::default();
        create_npc_factions(&mut game, ruleset);
        let mut region = Region::default();
        region.terrain_type = Some(R_NEXUS);
        let region_id = game.add_region(region);
        let dummy = game
            .add_object(region_id, Object::default())
            .expect("dummy object");

        create_city_monster(&mut game, ruleset, &tables, region_id).unwrap();

        let units = game.object(dummy).unwrap().unit_ids();
        assert_eq!(units.len(), 2);
        let guard = game.unit(units[0]).unwrap();
        assert_eq!(guard.name.as_deref(), Some("City Guard (1)"));
        assert_eq!(guard.unit_type, UnitType::Guard);
        assert_eq!(guard.guard, GuardMode::Guard);
        assert_eq!(
            guard
                .items
                .entries()
                .iter()
                .map(|item| item.item_type)
                .collect::<Vec<_>>(),
            [
                ItemId::Leaders,
                ItemId::Sword,
                ItemId::Amuletofi,
                ItemId::Silver,
                ItemId::PlateArmor
            ]
        );
        assert!(is_guarded(&game, region_id));
    }

    #[test]
    fn wandering_growth_places_the_seeded_number_of_monster_units() {
        let ruleset = <dyn Ruleset>::from_name("standard").expect("standard ruleset");
        let tables = ruleset.tables();
        let mut game = Game::default();
        create_npc_factions(&mut game, ruleset);
        let mut array = RegionArray::new(8, 16, RegionLevelType::Surface).unwrap();
        let mut region_ids = Vec::new();
        for (x, y) in [(0, 0), (1, 1)] {
            let mut region = Region::default();
            region.terrain_type = Some(R_TUNDRA);
            region.zloc = 0;
            let region_id = game.add_region(region);
            array.set_region(x, y, region_id).unwrap();
            game.add_object(region_id, Object::default())
                .expect("dummy object");
            region_ids.push(region_id);
        }
        game.add_region_array(array);

        grow_wandering_monsters(&mut game, ruleset, &tables, 50, &mut Rng::new(42)).unwrap();

        let units = region_ids
            .iter()
            .flat_map(|region| game.region(*region).unwrap().object_ids())
            .flat_map(|object| game.object(*object).unwrap().unit_ids())
            .copied()
            .collect::<Vec<_>>();
        assert_eq!(units.len(), 1);
        assert!(units.iter().all(|unit| {
            game.unit(*unit)
                .is_some_and(|unit| unit.unit_type == UnitType::WanderingMonster)
        }));
    }

    #[test]
    fn lair_growth_preserves_magician_army_unit_and_item_order() {
        let ruleset = <dyn Ruleset>::from_name("standard").expect("standard ruleset");
        let tables = ruleset.tables();
        let mut game = Game::default();
        create_npc_factions(&mut game, ruleset);
        let mut region = Region::default();
        region.terrain_type = Some(R_PLAIN);
        let region_id = game.add_region(region);
        game.add_object(region_id, Object::default())
            .expect("dummy object");
        let mut tower = Object::default();
        tower.object_type = O_MAGETOWER as i32;
        let tower_id = game.add_object(region_id, tower).expect("mage tower");

        grow_lair_monsters(&mut game, ruleset, &tables, 100, &mut Rng::new(42)).unwrap();

        let units = game.object(tower_id).unwrap().unit_ids();
        assert_eq!(units.len(), 2);
        let mages = game.unit(units[0]).unwrap();
        assert_eq!(
            mages.name.as_deref(),
            Some(format!("Evil Mages ({})", mages.num).as_str())
        );
        assert_eq!(mages.items.entries()[0].item_type, ItemId::Magicians);
        if mages.items.entries().len() > 1 {
            assert_eq!(mages.items.entries()[1].item_type, ItemId::Sorcerers);
        }
        assert_eq!(mages.guard, GuardMode::None);
        assert_ne!(mages.flags & FLAG_BEHIND, 0);
        let warriors = game.unit(units[1]).unwrap();
        assert_eq!(
            warriors.name.as_deref(),
            Some(format!("Evil Warriors ({})", warriors.num).as_str())
        );
        assert_eq!(warriors.guard, GuardMode::None);
    }

    #[test]
    fn virtual_monsters_are_ruleset_specific_and_use_faction_two() {
        for (ruleset_name, expected) in [("standard", true), ("neworigins", false)] {
            let ruleset = <dyn Ruleset>::from_name(ruleset_name).expect("ruleset");
            let mut game = Game::default();
            create_npc_factions(&mut game, ruleset);
            let region_id = game.add_region(Region::default());
            let mut keep = Object::default();
            keep.object_type = O_BKEEP as i32;
            let keep_id = game.add_object(region_id, keep).expect("black keep");

            create_virtual_monsters(&mut game, ruleset).unwrap();

            let units = game.object(keep_id).unwrap().unit_ids();
            assert_eq!(units.len(), usize::from(expected));
            if expected {
                let demon = game.unit(units[0]).unwrap();
                assert_eq!(demon.faction, game.monster_faction);
                assert_eq!(
                    demon.name.as_deref(),
                    Some(format!("Elder Demons ({})", demon.num).as_str())
                );
                assert_eq!(demon.items.get_num(ItemId::Balrog), 200);
            }
        }
    }
}
