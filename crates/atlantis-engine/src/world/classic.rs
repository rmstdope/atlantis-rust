use crate::ruleset::Ruleset;
use crate::{
    defs::terrains::{terrain_flags, TerrainDef, TerrainId, R_LAKE, R_NEXUS, R_NUM, R_OCEAN},
    defs::{ItemFlags, Tables},
    economy::setup_region,
    model::{Game, Region, RegionArray, RegionId, RegionLevelType},
    rng::Rng,
};

use super::{names::NameGenerator, WorldGenerationError};

/// C++: `standard/map.cpp` `MakeRegions` and `aregion.cpp` `NeighSetup`.
pub fn create_region_array(
    game: &mut Game,
    level_type: RegionLevelType,
    name: &str,
    width: i32,
    height: i32,
) -> Result<usize, WorldGenerationError> {
    let mut array = RegionArray::new(width, height, level_type)?;
    array.name = name.to_owned();

    let level = i32::try_from(game.region_arrays.len())
        .map_err(|_| WorldGenerationError::RegionNumberOverflow)?;
    let first_region_number = i32::try_from(game.region_ids().len())
        .map_err(|_| WorldGenerationError::RegionNumberOverflow)?;
    let cell_count = usize::try_from(
        width
            .checked_mul(height)
            .ok_or(WorldGenerationError::RegionNumberOverflow)?
            / 2,
    )
    .map_err(|_| WorldGenerationError::RegionNumberOverflow)?;
    if cell_count > 0
        && first_region_number
            .checked_add(
                i32::try_from(cell_count - 1)
                    .map_err(|_| WorldGenerationError::RegionNumberOverflow)?,
            )
            .is_none()
    {
        return Err(WorldGenerationError::RegionNumberOverflow);
    }

    let mut local_region_number = 0_usize;
    for y in 0..height {
        for x in 0..width {
            if (x + y) % 2 != 0 {
                continue;
            }

            let mut region = Region::default();
            region.num = first_region_number
                .checked_add(
                    i32::try_from(local_region_number)
                        .map_err(|_| WorldGenerationError::RegionNumberOverflow)?,
                )
                .ok_or(WorldGenerationError::RegionNumberOverflow)?;
            region.xloc = x;
            region.yloc = y;
            region.zloc = level;
            region.race = -1;
            region.wages = -1;
            let region_id = game.add_region(region);
            array
                .set_region(x, y, region_id)
                .expect("generated coordinates are valid cells");
            local_region_number += 1;
        }
    }

    let array_index = game.add_region_array(array);
    let connections = {
        let array = &game.region_arrays[array_index];
        let mut connections = Vec::with_capacity(cell_count);
        for y in 0..height {
            for x in 0..width {
                if let Some(region_id) = array.get_region(x, y) {
                    connections.push((region_id, planar_neighbors(array, x, y)));
                }
            }
        }
        connections
    };
    for (region_id, neighbors) in connections {
        game.region_mut(region_id)
            .expect("newly inserted region exists")
            .neighbors = neighbors;
    }

    Ok(array_index)
}

/// C++: `standard/map.cpp` `CreateNexusLevel`.
pub fn create_nexus_level(
    game: &mut Game,
    ruleset: &dyn Ruleset,
    rng: &mut Rng,
    width: i32,
    height: i32,
) -> Result<usize, WorldGenerationError> {
    let level = create_region_array(game, RegionLevelType::Nexus, "nexus", width, height)?;
    let array = game.region_arrays[level].clone();
    let name = format!("{} Nexus", ruleset.game_defs().world_name);

    for y in 0..height {
        for x in 0..width {
            let Some(region_id) = array.get_region(x, y) else {
                continue;
            };
            let region = game
                .region_mut(region_id)
                .expect("nexus region exists in the game");
            region.name.clone_from(&name);
            region.terrain_type = Some(R_NEXUS);
        }
    }

    let tables = ruleset.tables();
    let mut names = NameGenerator::new(ruleset.world_name_style());
    final_setup(
        game,
        &array,
        ruleset.game_defs(),
        ruleset,
        &tables,
        &mut names,
        rng,
    )?;

    Ok(level)
}

pub fn create_classic_surface_level(
    game: &mut Game,
    ruleset: &dyn Ruleset,
    rng: &mut Rng,
    width: i32,
    height: i32,
) -> Result<usize, WorldGenerationError> {
    let level = create_region_array(game, RegionLevelType::Surface, "surface", width, height)?;
    let array = game.region_arrays[level].clone();
    let definitions = ruleset.game_defs();
    let tables = ruleset.tables();
    let mut names = NameGenerator::new(ruleset.world_name_style());

    make_land(game, &array, definitions, rng);
    clean_up_water(game, &array, definitions, rng);
    setup_anchors(
        game,
        &array,
        definitions,
        ruleset,
        &tables.terrains,
        &mut names,
        rng,
    );
    grow_terrain(
        game,
        &array,
        definitions,
        ruleset,
        &tables.terrains,
        &mut names,
        rng,
    );
    assign_types(
        game,
        &array,
        definitions,
        ruleset,
        &tables.terrains,
        &mut names,
        rng,
    );
    sever_land_bridges(game, &array, definitions, &tables.terrains, rng);
    if definitions.lakes != 0 {
        remove_coastal_lakes(game, &array, definitions, &tables.terrains);
    }
    if definitions.grow_races != 0 {
        grow_races(
            game,
            &array,
            definitions,
            &tables.terrains,
            &tables.item_families,
            rng,
        );
    }
    final_setup(game, &array, definitions, ruleset, &tables, &mut names, rng)?;

    Ok(level)
}

fn terrain_type(game: &Game, region_id: RegionId) -> Option<TerrainId> {
    game.region(region_id)
        .expect("generated region exists")
        .terrain_type
}

fn set_terrain_type(game: &mut Game, region_id: RegionId, terrain: TerrainId) {
    game.region_mut(region_id)
        .expect("generated region exists")
        .terrain_type = Some(terrain);
}

fn neighbor(game: &Game, region_id: RegionId, direction: usize) -> Option<RegionId> {
    game.region(region_id)
        .expect("generated region exists")
        .neighbors[direction]
}

fn scan_region_ids(array: &RegionArray) -> Vec<RegionId> {
    let mut region_ids = Vec::new();
    for x in 0..array.x {
        for y in 0..array.y {
            if let Some(region_id) = array.get_region(x, y) {
                region_ids.push(region_id);
            }
        }
    }
    region_ids
}

fn set_unassigned_regions_to_ocean(game: &mut Game, array: &RegionArray) {
    for x in 0..array.x {
        for y in 0..array.y {
            let Some(region_id) = array.get_region(x, y) else {
                continue;
            };
            if terrain_type(game, region_id).is_none() {
                set_terrain_type(game, region_id, R_OCEAN);
            }
        }
    }
}

/// C++: `standard/map.cpp` `MakeLand`; retains its rejection draws and polar detours.
fn make_land(game: &mut Game, array: &RegionArray, defs: &crate::defs::GameDefs, rng: &mut Rng) {
    let total = scan_region_ids(array).len() as i32;
    let mut ocean = total;
    let mut percent_ocean = defs.ocean;
    if defs.sea_limit != 0 {
        percent_ocean = percent_ocean * (100 + 2 * defs.sea_limit) / 100;
    }

    while ocean > total * percent_ocean / 100 {
        let mut size = rng.get_random(defs.continent_size);
        size *= size;
        let temp_x = rng.get_random(array.x);
        let y_offset = array.y / 40;
        let y_band = array.y / 2 - 2 * y_offset;
        let temp_y = (rng.get_random(y_band) + y_offset) * 2 + temp_x % 2;
        let Some(mut region_id) = array.get_region(temp_x, temp_y) else {
            continue;
        };

        if rng.get_random(100) < defs.archipelago {
            size = size / 5 + 1;
            let mut first = true;
            let mut tries = 0;
            for _ in 0..size {
                let mut direction = rng.get_random(6) as usize;
                let mut new_region = neighbor(game, region_id, direction);
                while new_region.is_none() {
                    direction = rng.get_random(6) as usize;
                    new_region = neighbor(game, region_id, direction);
                }
                tries += 1;

                let mut blocked = false;
                for _ in 0..2 {
                    let sea_region = new_region.expect("direction has a neighboring region");
                    new_region = neighbor(game, sea_region, direction);
                    if new_region.is_none() {
                        blocked = true;
                        break;
                    }
                }
                if blocked {
                    break;
                }
                let sea_region = new_region.expect("two-step path has a region");
                let next_direction = rng.get_random(6) as usize;
                let Some(mut island_start) = neighbor(game, sea_region, next_direction) else {
                    break;
                };
                if first {
                    island_start = region_id;
                    first = false;
                }

                if terrain_type(game, island_start).is_none() {
                    region_id = island_start;
                    tries = 0;
                    set_terrain_type(game, region_id, R_NUM);
                    ocean -= 1;
                } else {
                    if tries > 5 {
                        break;
                    }
                    continue;
                }

                let mut grow_it = rng.get_random(20);
                let mut growth = 0;
                let mut grow_ch = 2;
                while grow_it > grow_ch {
                    grow_it = rng.get_random(20);
                    tries = 0;
                    let mut new_direction = rng.get_random(6) as usize;
                    while direction == real_direction_complement(game, region_id, new_direction) {
                        new_direction = rng.get_random(6) as usize;
                    }
                    let mut new_region = neighbor(game, region_id, new_direction);
                    while new_region.is_none() && tries < 36 {
                        while direction == real_direction_complement(game, region_id, new_direction)
                        {
                            new_direction = rng.get_random(6) as usize;
                        }
                        new_region = neighbor(game, region_id, new_direction);
                        tries += 1;
                    }
                    let Some(next_region) = new_region else {
                        continue;
                    };
                    region_id = next_region;
                    tries = 0;
                    if terrain_type(game, region_id).is_none() {
                        set_terrain_type(game, region_id, R_NUM);
                        growth += 1;
                        if growth > grow_ch {
                            grow_ch = growth;
                        }
                        ocean -= 1;
                    }
                }
            }
        } else {
            if terrain_type(game, region_id).is_none() {
                set_terrain_type(game, region_id, R_NUM);
                ocean -= 1;
            }
            for _ in 0..size {
                let direction = rng.get_random(6) as usize;
                let y = game
                    .region(region_id)
                    .expect("generated region exists")
                    .yloc;
                if y < y_offset * 2 && (direction < 2 || direction == 5) && rng.get_random(4) < 3 {
                    continue;
                }
                if y > (y_band + y_offset) * 2
                    && (direction < 5 && direction > 1)
                    && rng.get_random(4) < 3
                {
                    continue;
                }
                let Some(new_region) = neighbor(game, region_id, direction) else {
                    break;
                };
                let pole_check =
                    (0..6).any(|direction| neighbor(game, new_region, direction).is_none());
                if pole_check {
                    break;
                }
                region_id = new_region;
                if terrain_type(game, region_id).is_none() {
                    set_terrain_type(game, region_id, R_NUM);
                    ocean -= 1;
                }
            }
        }
    }

    set_unassigned_regions_to_ocean(game, array);
}

fn real_direction_complement(game: &Game, region_id: RegionId, direction: usize) -> usize {
    // C++ direction slots are north, northeast, southeast, south, southwest, northwest.
    const OPPOSITE: [usize; 6] = [3, 4, 5, 0, 1, 2];
    neighbor(game, region_id, direction)
        .and_then(|neighbor_id| {
            game.region(neighbor_id)
                .expect("neighbor exists")
                .neighbors
                .iter()
                .position(|candidate| *candidate == Some(region_id))
        })
        .unwrap_or(OPPOSITE[direction])
}

/// C++: `standard/map.cpp` `CleanUpWater` and `ARegion::CheckSea`.
fn clean_up_water(
    game: &mut Game,
    array: &RegionArray,
    defs: &crate::defs::GameDefs,
    rng: &mut Rng,
) {
    for _ in 0..=defs.sea_limit {
        for x in 0..array.x {
            for y in 0..array.y {
                let Some(region_id) = array.get_region(x, y) else {
                    continue;
                };
                if terrain_type(game, region_id) != Some(R_OCEAN) {
                    continue;
                }
                let mut remaining_ocean = 0;
                for direction in 0..6 {
                    remaining_ocean +=
                        check_sea(game, region_id, direction, defs.sea_limit, remaining_ocean);
                }
                if remaining_ocean > 0 {
                    continue;
                }
                let region = game.region_mut(region_id).expect("generated region exists");
                region.wages = 0;
                region.terrain_type = Some(if rng.get_random(100) < defs.lakes {
                    R_LAKE
                } else {
                    R_NUM
                });
            }
        }
    }
}

fn check_sea(
    game: &Game,
    region_id: RegionId,
    direction: usize,
    range: i32,
    remaining_ocean: i32,
) -> i32 {
    if terrain_type(game, region_id) != Some(R_OCEAN) {
        return 0;
    }
    if range < 1 {
        return 1;
    }
    let range = range - 1;
    let mut remaining_ocean = remaining_ocean;
    for offset in -1..2 {
        let next_direction = (direction as i32 + offset + 6) as usize % 6;
        let Some(next_region) = neighbor(game, region_id, next_direction) else {
            continue;
        };
        remaining_ocean += check_sea(game, next_region, direction, range, remaining_ocean);
        if remaining_ocean != 0 {
            break;
        }
    }
    remaining_ocean
}

/// C++: `standard/map.cpp` `SetupAnchors`; coordinates and random draws follow its nested loops.
fn setup_anchors(
    game: &mut Game,
    array: &RegionArray,
    defs: &crate::defs::GameDefs,
    ruleset: &dyn Ruleset,
    terrain_defs: &[TerrainDef],
    names: &mut NameGenerator,
    rng: &mut Rng,
) {
    let mut skip = 250;
    let mut factor = 2;
    if defs.terrain_granularity != 0 {
        skip = defs.terrain_granularity;
        while skip > 5 {
            factor += 1;
            skip -= 5;
            if skip < 1 {
                skip = 1;
            }
        }
        skip = 100 * ((skip + 3) * factor + 2) / (skip + factor - 2);
    }

    for x in 0..array.x / factor {
        for y in 0..array.y / (factor * 2) {
            if rng.get_random(1000) > skip {
                continue;
            }
            for _ in 0..4 {
                let temp_x = x * factor + rng.get_random(factor);
                let temp_y = y * factor * 2 + rng.get_random(factor) * 2 + temp_x % 2;
                let Some(region_id) = array.get_region(temp_x, temp_y) else {
                    continue;
                };
                if terrain_type(game, region_id) != Some(R_NUM) {
                    continue;
                }
                let region_y = game
                    .region(region_id)
                    .expect("generated region exists")
                    .yloc;
                let terrain = ruleset.get_reg_type(latitude(region_y, array.y), rng);
                set_terrain_type(game, region_id, terrain);
                game.region_mut(region_id)
                    .expect("generated region exists")
                    .population = 1;
                if !is_ocean_like(terrain, terrain_defs) {
                    let name = names.next_region_name(terrain, terrain_defs, rng);
                    game.region_mut(region_id)
                        .expect("generated region exists")
                        .wages = name;
                }
                break;
            }
        }
    }
}

fn latitude(y: i32, height: i32) -> i32 {
    let mut latitude = y * 8 / height;
    if latitude > 3 {
        latitude = 7 - latitude;
    }
    latitude.max(0)
}

fn is_ocean_like(terrain: TerrainId, terrain_defs: &[TerrainDef]) -> bool {
    terrain_defs[terrain].similar_type == R_OCEAN as i32
}

/// C++: `standard/map.cpp` `GrowTerrain`; retains the inner-loop breaks and their scan effects.
fn grow_terrain(
    game: &mut Game,
    array: &RegionArray,
    defs: &crate::defs::GameDefs,
    ruleset: &dyn Ruleset,
    terrain_defs: &[TerrainDef],
    names: &mut NameGenerator,
    rng: &mut Rng,
) {
    for iteration in 0..30 {
        for x in 0..array.x {
            for y in 0..array.y {
                let Some(region_id) = array.get_region(x, y) else {
                    continue;
                };
                game.region_mut(region_id)
                    .expect("generated region exists")
                    .population = 1;
            }
        }

        for x in 0..array.x {
            for y in 0..array.y {
                let Some(region_id) = array.get_region(x, y) else {
                    continue;
                };
                if iteration > 0 && iteration < 21 && rng.get_random(3) < 2 {
                    continue;
                }
                if terrain_type(game, region_id) != Some(R_NUM) {
                    continue;
                }
                if defs.lakes != 0 && rng.get_random(100) < defs.lakes / 10 + 1 {
                    set_terrain_type(game, region_id, R_LAKE);
                    break;
                }
                if rng.get_random(1000) < defs.odd_terrain {
                    let region_y = game
                        .region(region_id)
                        .expect("generated region exists")
                        .yloc;
                    let terrain = ruleset.get_reg_type(latitude(region_y, array.y), rng);
                    set_terrain_type(game, region_id, terrain);
                    if !is_ocean_like(terrain, terrain_defs) {
                        let name = names.next_region_name(terrain, terrain_defs, rng);
                        game.region_mut(region_id)
                            .expect("generated region exists")
                            .wages = name;
                    }
                    break;
                }

                let initial_direction = rng.get_random(6) as usize;
                for offset in 0..6 {
                    let direction = (offset + initial_direction) % 6;
                    let Some(neighbor_id) = neighbor(game, region_id, direction) else {
                        continue;
                    };
                    let neighbor_region = game.region(neighbor_id).expect("neighbor exists");
                    if neighbor_region.population < 1 {
                        continue;
                    }
                    let neighbor_terrain = neighbor_region.terrain_type;
                    if neighbor_terrain != Some(R_NUM)
                        && neighbor_terrain
                            .is_some_and(|terrain| !is_ocean_like(terrain, terrain_defs))
                    {
                        let neighbor_terrain =
                            neighbor_terrain.expect("terrain condition requires a value");
                        let neighbor_wages = neighbor_region.wages;
                        if iteration == 0 {
                            game.region_mut(neighbor_id)
                                .expect("neighbor exists")
                                .population = 0;
                        }
                        let region = game.region_mut(region_id).expect("generated region exists");
                        region.population = 0;
                        region.race = neighbor_terrain as i32;
                        region.wages = neighbor_wages;
                        break;
                    }
                }
            }
        }

        for x in 0..array.x {
            for y in 0..array.y {
                let Some(region_id) = array.get_region(x, y) else {
                    continue;
                };
                let region = game.region(region_id).expect("generated region exists");
                if region.terrain_type == Some(R_NUM) && region.race != -1 {
                    set_terrain_type(
                        game,
                        region_id,
                        usize::try_from(region.race).expect("generated terrain is nonnegative"),
                    );
                }
            }
        }
    }
}

/// C++: `standard/map.cpp` `AssignTypes` and `RandomTerrain`.
fn assign_types(
    game: &mut Game,
    array: &RegionArray,
    defs: &crate::defs::GameDefs,
    ruleset: &dyn Ruleset,
    terrain_defs: &[TerrainDef],
    names: &mut NameGenerator,
    rng: &mut Rng,
) {
    for x in 0..array.x {
        for y in 0..array.y {
            let Some(region_id) = array.get_region(x, y) else {
                continue;
            };
            if terrain_type(game, region_id) != Some(R_NUM) {
                continue;
            }

            let mut adjacent_terrain = None;
            let mut adjacent_name = -1;
            for direction in 0..6 {
                let Some(neighbor_id) = neighbor(game, region_id, direction) else {
                    continue;
                };
                let neighbor_region = game.region(neighbor_id).expect("neighbor exists");
                let Some(neighbor_terrain) = neighbor_region.terrain_type else {
                    continue;
                };
                if neighbor_terrain != R_NUM
                    && !is_ocean_like(neighbor_terrain, terrain_defs)
                    && neighbor_region.wages > 0
                {
                    adjacent_terrain = Some(neighbor_terrain);
                    adjacent_name = neighbor_region.wages;
                }
            }

            if let Some(terrain) = adjacent_terrain.filter(|_| defs.conquest_game == 0) {
                let region = game.region_mut(region_id).expect("generated region exists");
                region.terrain_type = Some(terrain);
                region.wages = adjacent_name;
            } else {
                let region_y = game
                    .region(region_id)
                    .expect("generated region exists")
                    .yloc;
                let terrain = ruleset.get_reg_type(latitude(region_y, array.y), rng);
                set_terrain_type(game, region_id, terrain);
                let name = names.next_region_name(terrain, terrain_defs, rng);
                game.region_mut(region_id)
                    .expect("generated region exists")
                    .wages = name;
            }
        }
    }
}

/// C++: `standard/map.cpp` `SeverLandBridges`.
fn sever_land_bridges(
    game: &mut Game,
    array: &RegionArray,
    defs: &crate::defs::GameDefs,
    terrain_defs: &[TerrainDef],
    rng: &mut Rng,
) {
    for x in 0..array.x {
        for y in 0..array.y {
            let Some(region_id) = array.get_region(x, y) else {
                continue;
            };
            let Some(terrain) = terrain_type(game, region_id) else {
                continue;
            };
            if is_ocean_like(terrain, terrain_defs)
                || coastal_count(game, region_id, defs, terrain_defs) != 4
            {
                continue;
            }
            let mut chance = defs.sever_land_bridges;
            for direction in 0..6 {
                let Some(neighbor_id) = neighbor(game, region_id, direction) else {
                    continue;
                };
                let Some(neighbor_terrain) = terrain_type(game, neighbor_id) else {
                    continue;
                };
                if is_ocean_like(neighbor_terrain, terrain_defs) {
                    continue;
                }
                if coastal_count(game, neighbor_id, defs, terrain_defs) == 4 {
                    chance *= 2;
                }
            }
            if rng.get_random(100) < chance {
                game.region_mut(region_id)
                    .expect("generated region exists")
                    .wages = -2;
            }
        }
    }

    for x in 0..array.x {
        for y in 0..array.y {
            let Some(region_id) = array.get_region(x, y) else {
                continue;
            };
            if game
                .region(region_id)
                .expect("generated region exists")
                .wages
                <= -2
            {
                let region = game.region_mut(region_id).expect("generated region exists");
                region.terrain_type = Some(R_OCEAN);
                region.wages = -1;
            }
        }
    }
}

fn coastal_count(
    game: &Game,
    region_id: RegionId,
    defs: &crate::defs::GameDefs,
    terrain_defs: &[TerrainDef],
) -> i32 {
    let terrain = terrain_type(game, region_id).expect("generated region has terrain");
    if terrain == R_LAKE && defs.lakeside_is_coastal != 0 {
        return 1;
    }
    if terrain != R_LAKE && is_ocean_like(terrain, terrain_defs) {
        return 1;
    }

    let mut count = 0;
    for direction in 0..6 {
        let Some(neighbor_id) = neighbor(game, region_id, direction) else {
            continue;
        };
        let Some(neighbor_terrain) = terrain_type(game, neighbor_id) else {
            continue;
        };
        if is_ocean_like(neighbor_terrain, terrain_defs)
            && (defs.lakeside_is_coastal != 0 || neighbor_terrain != R_LAKE)
        {
            count += 1;
        }
    }
    count
}

/// C++: `standard/map.cpp` `RemoveCoastalLakes`.
fn remove_coastal_lakes(
    game: &mut Game,
    array: &RegionArray,
    defs: &crate::defs::GameDefs,
    terrain_defs: &[TerrainDef],
) {
    for _ in 0..2 {
        for x in 0..array.x {
            for y in 0..array.y {
                let Some(region_id) = array.get_region(x, y) else {
                    continue;
                };
                if terrain_type(game, region_id) != Some(R_LAKE) {
                    continue;
                }
                if coastal_count(game, region_id, defs, terrain_defs) > 0 {
                    let region = game.region_mut(region_id).expect("generated region exists");
                    region.terrain_type = Some(R_OCEAN);
                    region.wages = -1;
                } else if game
                    .region(region_id)
                    .expect("generated region exists")
                    .wages
                    <= 0
                {
                    let mut preferred_name = 0;
                    let mut preferred_count = 0;
                    let mut second_name = 0;
                    let mut second_count = 0;
                    for direction in 0..6 {
                        let Some(neighbor_id) = neighbor(game, region_id, direction) else {
                            continue;
                        };
                        let neighbor_region = game.region(neighbor_id).expect("neighbor exists");
                        let Some(neighbor_terrain) = neighbor_region.terrain_type else {
                            continue;
                        };
                        if neighbor_region.wages > 0 && neighbor_terrain == R_LAKE {
                            preferred_count = 1;
                            preferred_name = neighbor_region.wages;
                            break;
                        }
                        if !is_ocean_like(neighbor_terrain, terrain_defs)
                            && neighbor_region.wages > -1
                        {
                            if neighbor_region.wages == preferred_name {
                                preferred_count += 1;
                            } else if neighbor_region.wages == second_name {
                                second_count += 1;
                            } else if second_count == 0 {
                                second_name = neighbor_region.wages;
                                second_count += 1;
                            }
                            if second_count > preferred_count {
                                std::mem::swap(&mut preferred_name, &mut second_name);
                                std::mem::swap(&mut preferred_count, &mut second_count);
                            }
                        }
                    }
                    if preferred_count > 0 {
                        game.region_mut(region_id)
                            .expect("generated region exists")
                            .wages = preferred_name;
                    }
                }
            }
        }
    }
}

/// C++: `standard/map.cpp` `FinalSetup` and `aregion.cpp` `ARegion::Setup`.
fn final_setup(
    game: &mut Game,
    array: &RegionArray,
    definitions: &crate::defs::GameDefs,
    ruleset: &dyn Ruleset,
    tables: &crate::defs::RulesetTables,
    names: &mut NameGenerator,
    rng: &mut Rng,
) -> Result<(), WorldGenerationError> {
    for x in 0..array.x {
        for y in 0..array.y {
            let Some(region_id) = array.get_region(x, y) else {
                continue;
            };
            let region = game.region(region_id).expect("generated region exists");
            let terrain = region
                .terrain_type
                .expect("classic generation assigns every region a terrain");
            let wages = region.wages;
            if is_ocean_like(terrain, &tables.terrains) && terrain != R_LAKE {
                game.region_mut(region_id)
                    .expect("generated region exists")
                    .name = format!("{} Ocean", definitions.world_name);
            } else if wages == -1 {
                game.region_mut(region_id)
                    .expect("generated region exists")
                    .name = ruleset.unnamed_region_name().to_owned();
            } else if wages != -2 {
                let name = names
                    .resolve(wages)
                    .ok_or(WorldGenerationError::MissingGeneratedName(wages))?;
                game.region_mut(region_id)
                    .expect("generated region exists")
                    .name = name.to_owned();
            } else {
                game.region_mut(region_id)
                    .expect("generated region exists")
                    .wages = -1;
            }

            let is_coastal = region_is_coastal(
                game,
                region_id,
                &tables.terrains,
                definitions.lakeside_is_coastal != 0,
            );
            let has_ocean_neighbor = region_has_ocean_neighbor(game, region_id, &tables.terrains);
            let mut town_name = |rng: &mut Rng, _region: &Region| {
                names.next_town_name(terrain, &tables.terrains, has_ocean_neighbor, rng)
            };
            setup_region(
                game,
                region_id,
                definitions,
                tables,
                is_coastal,
                1.0,
                rng,
                &mut town_name,
            )?;
        }
    }
    Ok(())
}

fn region_is_coastal(
    game: &Game,
    region_id: RegionId,
    terrain_defs: &[TerrainDef],
    lakeside_is_coastal: bool,
) -> bool {
    let region = game.region(region_id).expect("generated region exists");
    let terrain = region
        .terrain_type
        .expect("classic generation assigns every region a terrain");
    if terrain == R_LAKE && lakeside_is_coastal {
        return true;
    }
    if terrain != R_LAKE && is_ocean_like(terrain, terrain_defs) {
        return true;
    }
    region.neighbors.iter().flatten().any(|neighbor_id| {
        let neighbor_terrain = game
            .region(*neighbor_id)
            .expect("neighbor exists")
            .terrain_type
            .expect("classic generation assigns every region a terrain");
        is_ocean_like(neighbor_terrain, terrain_defs)
            && (lakeside_is_coastal || neighbor_terrain != R_LAKE)
    })
}

fn region_has_ocean_neighbor(
    game: &Game,
    region_id: RegionId,
    terrain_defs: &[TerrainDef],
) -> bool {
    game.region(region_id)
        .expect("generated region exists")
        .neighbors
        .iter()
        .flatten()
        .any(|neighbor_id| {
            let neighbor_terrain = game
                .region(*neighbor_id)
                .expect("neighbor exists")
                .terrain_type
                .expect("classic generation assigns every region a terrain");
            is_ocean_like(neighbor_terrain, terrain_defs)
        })
}

/// C++: `standard/map.cpp` `GrowRaces`; this runs only when the ruleset enables race growth.
fn grow_races(
    game: &mut Game,
    array: &RegionArray,
    defs: &crate::defs::GameDefs,
    terrain_defs: &[TerrainDef],
    item_tables: &Tables,
    rng: &mut Rng,
) {
    for x in 0..array.x {
        for y in 0..array.y {
            let Some(region_id) = array.get_region(x, y) else {
                continue;
            };
            game.region_mut(region_id)
                .expect("generated region exists")
                .race = -1;
        }
    }

    for x in 0..array.x {
        for y in 0..array.y {
            let jiggle = 4 + 2 * defs.grow_races;
            if (y + (x % 2) * (jiggle / 2)) % jiggle > 1 {
                continue;
            }
            let x_offset = x + 2 - rng.get_random(3) - rng.get_random(3);
            let Some(region_id) = array.get_region(x_offset, y) else {
                continue;
            };
            let region_terrain = terrain_type(game, region_id).expect("generated terrain exists");
            if region_terrain == R_LAKE && defs.lakeside_is_coastal == 0 {
                continue;
            }
            let terrain = &terrain_defs[region_terrain];
            if terrain.flags & terrain_flags::BARREN != 0 {
                continue;
            }
            let mut wigout = 0;

            if is_ocean_like(region_terrain, terrain_defs) {
                let initial_direction = rng.get_random(6) as usize;
                let Some(mut neighbor_id) = neighbor(game, region_id, initial_direction) else {
                    continue;
                };
                let mut count = 0;
                while count < 20
                    && game
                        .region(region_id)
                        .expect("generated region exists")
                        .race
                        == -1
                {
                    count += 1;
                    let neighbor_terrain =
                        terrain_type(game, neighbor_id).expect("neighbor terrain exists");
                    if !is_ocean_like(neighbor_terrain, terrain_defs) {
                        let races = &terrain_defs[neighbor_terrain].coastal_races;
                        loop {
                            let race = races[rng.get_random(races.len() as i32) as usize];
                            game.region_mut(region_id)
                                .expect("generated region exists")
                                .race = race;
                            wigout += 1;
                            if wigout > 100 || (race != -1 && !race_is_disabled(race, item_tables))
                            {
                                break;
                            }
                        }
                    } else {
                        let direction = rng.get_random(6) as usize;
                        if initial_direction
                            == real_direction_complement(game, neighbor_id, direction)
                        {
                            continue;
                        }
                        let Some(next_neighbor) = neighbor(game, neighbor_id, direction) else {
                            continue;
                        };
                        neighbor_id = next_neighbor;
                    }
                }
            } else {
                let races = &terrain.races;
                loop {
                    let race = races[rng.get_random(races.len() as i32) as usize];
                    game.region_mut(region_id)
                        .expect("generated region exists")
                        .race = race;
                    wigout += 1;
                    if wigout > 100 || (race != -1 && !race_is_disabled(race, item_tables)) {
                        break;
                    }
                }
            }
        }
    }

    for _ in 0..25 {
        for x in 0..array.x {
            for y in 0..array.y {
                let Some(region_id) = array.get_region(x, y) else {
                    continue;
                };
                let region_race = game
                    .region(region_id)
                    .expect("generated region exists")
                    .race;
                if region_race == -1 {
                    continue;
                }
                let region_terrain =
                    terrain_type(game, region_id).expect("generated terrain exists");
                for direction in 0..6 {
                    let Some(neighbor_id) = neighbor(game, region_id, direction) else {
                        continue;
                    };
                    if game.region(neighbor_id).expect("neighbor exists").race != -1 {
                        continue;
                    }
                    let neighbor_terrain =
                        terrain_type(game, neighbor_id).expect("neighbor terrain exists");
                    let coastal = terrain_defs[region_terrain]
                        .coastal_races
                        .contains(&region_race);
                    if is_ocean_like(neighbor_terrain, terrain_defs) && !coastal {
                        continue;
                    }

                    let mut chance = rng.get_random(5);
                    if coastal {
                        if is_ocean_like(neighbor_terrain, terrain_defs) {
                            chance += 2;
                        }
                    } else {
                        let race = race_definition(region_race, item_tables);
                        if race.terrain == terrain_defs[neighbor_terrain].similar_type {
                            chance += 2;
                        }
                        for candidate in &terrain_defs[neighbor_terrain].races {
                            if *candidate == region_race {
                                chance += 1;
                            }
                        }
                    }
                    if chance > 3 {
                        game.region_mut(neighbor_id).expect("neighbor exists").race = region_race;
                    }
                }
            }
        }
    }
}

fn race_is_disabled(race: i32, item_tables: &Tables) -> bool {
    let index = usize::try_from(race).expect("race item index is nonnegative");
    item_tables
        .items
        .get(index)
        .expect("race item exists")
        .flags
        & ItemFlags::DISABLED
        != 0
}

fn race_definition(race: i32, item_tables: &Tables) -> &crate::defs::ManDef {
    let index = usize::try_from(race).expect("race item index is nonnegative");
    let item = item_tables.items.get(index).expect("race item exists");
    let abbreviation = item
        .abbreviation
        .as_deref()
        .expect("race item has an abbreviation");
    item_tables
        .find_race(abbreviation)
        .expect("race definition exists")
}

fn planar_neighbors(array: &RegionArray, x: i32, y: i32) -> [Option<RegionId>; 6] {
    let last_y = array.y - 1;
    // Keep the C++ direction order: north, northeast, southeast, south, southwest, northwest.
    let candidates = [
        (x, y - 2, y != 0 && y != 1),
        (x + 1, y - 1, y != 0),
        (x + 1, y + 1, y != last_y),
        (x, y + 2, y != last_y && y != last_y - 1),
        (x - 1, y + 1, y != last_y),
        (x - 1, y - 1, y != 0),
    ];
    candidates.map(|(neighbor_x, neighbor_y, allowed)| {
        allowed
            .then(|| array.get_region(neighbor_x, neighbor_y))
            .flatten()
    })
}

#[cfg(test)]
mod tests {
    use crate::{
        defs::terrains::{
            R_DESERT, R_FOREST, R_JUNGLE, R_MOUNTAIN, R_NEXUS, R_OCEAN, R_PLAIN, R_SWAMP, R_TUNDRA,
        },
        model::{Game, RegionLevelType},
        rng::Rng,
        ruleset::{Ruleset, WorldNameStyle},
        text,
    };

    use super::{create_classic_surface_level, create_nexus_level, create_region_array};

    #[test]
    fn planar_level_has_all_hexes_and_reference_neighbor_topology() {
        let mut game = Game::default();
        let level = create_region_array(&mut game, RegionLevelType::Surface, "", 4, 4)
            .expect("surface region array");

        assert_eq!(level, 0);
        assert_eq!(game.region_arrays[level].x, 4);
        assert_eq!(game.region_arrays[level].y, 4);
        assert_eq!(game.region_ids().len(), 8);
        for y in 0..4 {
            for x in 0..4 {
                assert_eq!(
                    game.get_region_at(x, y, level as i32).is_some(),
                    (x + y) % 2 == 0
                );
            }
        }

        let origin = game.get_region_at(0, 0, level as i32).unwrap();
        let wrapped = game.region(origin).unwrap().neighbors[4].unwrap();
        assert_eq!(
            (
                game.region(wrapped).unwrap().xloc,
                game.region(wrapped).unwrap().yloc
            ),
            (3, 1)
        );
        assert_eq!(game.region(origin).unwrap().neighbors[0], None);
        assert_eq!(game.region(origin).unwrap().neighbors[1], None);
        let southeast = game.region(origin).unwrap().neighbors[2].unwrap();
        assert_eq!(
            (
                game.region(southeast).unwrap().xloc,
                game.region(southeast).unwrap().yloc
            ),
            (1, 1)
        );
        assert_eq!(game.region(origin).unwrap().neighbors[5], None);
        assert_eq!(game.region(wrapped).unwrap().neighbors[1], Some(origin));

        assert_eq!(game.region(southeast).unwrap().neighbors[5], Some(origin));
        let south = game.region(origin).unwrap().neighbors[3].unwrap();
        assert_eq!(
            (
                game.region(south).unwrap().xloc,
                game.region(south).unwrap().yloc
            ),
            (0, 2)
        );
        assert_eq!(game.region(south).unwrap().neighbors[0], Some(origin));
        let south_edge = game.get_region_at(1, 3, level as i32).unwrap();
        for direction in [2, 3, 4] {
            assert_eq!(
                game.region(south_edge).unwrap().neighbors[direction],
                None,
                "south pole boundary at direction {direction}"
            );
        }
        let opposites = [3, 4, 5, 0, 1, 2];
        for region_id in game.region_ids().iter().copied() {
            for (direction, opposite) in opposites.iter().copied().enumerate() {
                if let Some(neighbor_id) = game.region(region_id).unwrap().neighbors[direction] {
                    assert_eq!(
                        game.region(neighbor_id).unwrap().neighbors[opposite],
                        Some(region_id),
                        "nonreciprocal direction {direction}"
                    );
                }
            }
        }
    }

    #[test]
    fn nexus_level_is_named_and_uses_nexus_terrain() {
        for (ruleset_name, expected_name) in [("standard", "Unnamed"), ("neworigins", "The Void")] {
            let mut game = Game::default();
            let ruleset = <dyn Ruleset>::from_name(ruleset_name).expect("supported ruleset");
            let mut rng = Rng::new(424242);
            let level =
                create_nexus_level(&mut game, ruleset, &mut rng, 1, 1).expect("nexus level");

            assert_eq!(level, 0, "{ruleset_name}");
            assert_eq!(
                game.region_arrays[level].level_type,
                RegionLevelType::Nexus,
                "{ruleset_name}"
            );
            let region_id = game.get_region_at(0, 0, level as i32).unwrap();
            let region = game.region(region_id).unwrap();
            assert_eq!(region.terrain_type, Some(R_NEXUS), "{ruleset_name}");
            assert_eq!(region.name, expected_name, "{ruleset_name}");
            assert_eq!(region.neighbors, [None; 6], "{ruleset_name}");
            assert_eq!(region.habitat, 100, "{ruleset_name}");
            assert_eq!(region.population, 0, "{ruleset_name}");
            assert_eq!(region.object_ids().len(), 1, "{ruleset_name}");
        }
    }

    #[test]
    fn classic_surface_generation_covers_both_rulesets_and_respects_ocean_share() {
        for name in ["standard", "neworigins"] {
            let mut game = Game::default();
            let ruleset = <dyn Ruleset>::from_name(name).expect("supported ruleset");
            let mut rng = Rng::new(424242);
            create_nexus_level(&mut game, ruleset, &mut rng, 1, 1).expect("nexus level");
            let level = create_classic_surface_level(&mut game, ruleset, &mut rng, 64, 64)
                .expect("classic surface");
            let terrain_defs = ruleset.tables().terrains;

            assert_eq!(level, 1, "{name}");
            assert_eq!(game.region_arrays[level].name, "surface", "{name}");
            assert_eq!(game.region_ids().len(), 1 + 64 * 64 / 2, "{name}");
            let regions = game.region_arrays[level]
                .get_region(0, 0)
                .and_then(|id| game.region(id))
                .expect("surface origin");
            assert_eq!(regions.zloc, level as i32);

            let mut ocean_count = 0;
            let mut land_count = 0;
            for region_id in game.region_ids().iter().copied() {
                let region = game.region(region_id).expect("ordered region");
                if region.zloc != level as i32 {
                    continue;
                }
                let terrain = region.terrain_type.expect("generated terrain");
                assert!(terrain < crate::defs::R_NUM, "{name}: terrain {terrain}");
                let terrain_def = &terrain_defs[terrain];
                assert!(
                    region.habitat > 0,
                    "{name}: region habitat was not initialized"
                );
                assert!(
                    !region.object_ids().is_empty(),
                    "{name}: region has no dummy object"
                );
                if terrain_def.population == 0 {
                    assert_eq!(region.population, 0, "{name}: unpopulated terrain");
                } else {
                    assert!(region.population > 0, "{name}: populated terrain");
                }
                if terrain == crate::defs::terrains::R_OCEAN {
                    ocean_count += 1;
                } else if terrain != crate::defs::terrains::R_LAKE {
                    land_count += 1;
                }
            }

            assert!(land_count > 0, "{name}: no land terrain");
            assert!(ocean_count > 0, "{name}: no ocean terrain");
            let target_ocean_percent =
                ruleset.game_defs().ocean * (100 + 2 * ruleset.game_defs().sea_limit) / 100;
            let actual_ocean_percent = ocean_count * 100 / (64 * 64 / 2);
            assert!(
                (target_ocean_percent - 10..=target_ocean_percent + 10)
                    .contains(&actual_ocean_percent),
                "{name}: expected ocean share near {target_ocean_percent}%, got {actual_ocean_percent}%"
            );
        }
    }

    #[test]
    fn surface_terrain_rules_match_both_ruleset_tables() {
        let cases = [
            ("standard", 0, 23, R_TUNDRA),
            ("standard", 0, 24, R_MOUNTAIN),
            ("standard", 1, 15, R_PLAIN),
            ("standard", 1, 16, R_FOREST),
            ("standard", 1, 48, R_SWAMP),
            ("standard", 2, 44, R_JUNGLE),
            ("standard", 2, 52, R_DESERT),
            ("standard", 3, 36, R_JUNGLE),
            ("standard", 3, 48, R_DESERT),
            ("neworigins", 0, 31, R_TUNDRA),
            ("neworigins", 0, 32, R_MOUNTAIN),
            ("neworigins", 1, 7, R_TUNDRA),
            ("neworigins", 1, 8, R_PLAIN),
            ("neworigins", 1, 48, R_SWAMP),
            ("neworigins", 2, 44, R_JUNGLE),
            ("neworigins", 2, 52, R_DESERT),
            ("neworigins", 3, 36, R_JUNGLE),
        ];
        for (name, latitude, roll, expected) in cases {
            let ruleset = <dyn Ruleset>::from_name(name).expect("supported ruleset");
            assert_eq!(
                ruleset.surface_terrain_from_roll(latitude, roll),
                expected,
                "{name}, latitude {latitude}, roll {roll}"
            );
        }
    }

    #[test]
    fn level_scales_follow_ruleset_underworld_counts() {
        let standard = <dyn Ruleset>::from_name("standard").expect("standard ruleset");
        let neworigins = <dyn Ruleset>::from_name("neworigins").expect("neworigins ruleset");

        assert_eq!(standard.level_x_scale(1), 1);
        assert_eq!(standard.level_y_scale(2), 1);
        assert_eq!(neworigins.level_x_scale(1), 1);
        assert_eq!(neworigins.level_x_scale(2), 2);
        assert_eq!(neworigins.level_y_scale(2), 2);
        assert_eq!(standard.unnamed_region_name(), "Unnamed");
        assert_eq!(neworigins.unnamed_region_name(), "The Void");
    }

    #[test]
    fn name_generators_return_unique_resolvable_names() {
        let standard = <dyn Ruleset>::from_name("standard").expect("standard ruleset");
        let mut standard_names =
            super::super::names::NameGenerator::new(standard.world_name_style());
        let mut standard_rng = Rng::new(424242);
        let standard_terrains = standard.tables().terrains;
        let first = standard_names.next_region_name(R_PLAIN, &standard_terrains, &mut standard_rng);
        let second =
            standard_names.next_region_name(R_FOREST, &standard_terrains, &mut standard_rng);
        assert_ne!(first, second);
        assert!(standard_names.resolve(first).is_some());
        assert!(standard_names.resolve(second).is_some());
        let standard_town_names: Vec<_> = (0..32)
            .map(|_| {
                standard_names.next_town_name(R_PLAIN, &standard_terrains, false, &mut standard_rng)
            })
            .collect();
        assert_eq!(
            standard_town_names
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            standard_town_names.len()
        );

        let neworigins = <dyn Ruleset>::from_name("neworigins").expect("neworigins ruleset");
        let terrain_defs = neworigins.tables().terrains;
        let mut names = super::super::names::NameGenerator::new(WorldNameStyle::NewOrigins);
        let mut rng = Rng::new(78123);
        let mut generated: Vec<String> = Vec::new();
        for _ in 0..64 {
            let index = names.next_region_name(R_OCEAN, &terrain_defs, &mut rng);
            let name = names.resolve(index).expect("generated name index");
            assert!(name.len() <= 12);
            assert!(
                generated
                    .iter()
                    .all(|existing| !text::is_equal(existing, name)),
                "duplicate generated name: {name}"
            );
            generated.push(name.to_owned());
        }
        for _ in 0..16 {
            let name = names.next_town_name(R_PLAIN, &terrain_defs, true, &mut rng);
            assert!(name.len() <= 12);
            assert!(
                generated
                    .iter()
                    .all(|existing| !text::is_equal(existing, &name)),
                "town name duplicates a generated region name: {name}"
            );
            generated.push(name);
        }
    }

    #[test]
    fn terrain_distribution_matches_reference_50_seeds() {
        const TERRAIN_FIXTURE: &str =
            include_str!("../../../../fixtures/world-generation/classic-terrain-counts.json");
        let fixture: serde_json::Value =
            serde_json::from_str(TERRAIN_FIXTURE).expect("reference terrain fixture is valid JSON");
        assert_eq!(
            fixture["format"].as_str(),
            Some("atlantis-classic-terrain-counts-v1")
        );
        assert_eq!(fixture["width"].as_u64(), Some(64));
        assert_eq!(fixture["height"].as_u64(), Some(64));

        let fixture_seeds = fixture["seeds"]
            .as_array()
            .expect("reference fixture has seeds");
        let seeds = fixture_seeds
            .iter()
            .map(|seed| seed.as_u64().expect("fixture seed is an integer"))
            .collect::<Vec<_>>();
        assert_eq!(seeds, (1..=50).collect::<Vec<_>>());

        for ruleset_name in ["standard", "neworigins"] {
            let ruleset = <dyn Ruleset>::from_name(ruleset_name).expect("supported ruleset");
            let reference_counts = fixture["rulesets"][ruleset_name]
                .as_array()
                .expect("reference fixture has ruleset terrain counts");
            assert_eq!(reference_counts.len(), seeds.len(), "{ruleset_name}");
            let mut expected = vec![0_usize; crate::defs::R_NUM];
            let mut actual = vec![0_usize; crate::defs::R_NUM];

            for (seed_index, seed) in seeds.iter().copied().enumerate() {
                let counts = reference_counts[seed_index]
                    .as_array()
                    .expect("reference seed has a terrain-count array");
                assert_eq!(
                    counts.len(),
                    crate::defs::R_NUM,
                    "{ruleset_name}, seed {seed}"
                );
                let mut seed_total = 0;
                for (terrain, count) in counts.iter().enumerate() {
                    let count =
                        usize::try_from(count.as_u64().expect("terrain count is an integer"))
                            .expect("terrain count fits usize");
                    expected[terrain] += count;
                    seed_total += count;
                }
                assert_eq!(seed_total, 64 * 64 / 2, "{ruleset_name}, seed {seed}");

                let mut game = Game::default();
                let mut rng = Rng::new(i32::try_from(seed).expect("fixture seed fits i32"));
                create_nexus_level(&mut game, ruleset, &mut rng, 1, 1).expect("nexus level");
                let level = create_classic_surface_level(&mut game, ruleset, &mut rng, 64, 64)
                    .expect("classic surface");
                for region_id in game.region_ids().iter().copied() {
                    let region = game.region(region_id).expect("generated region exists");
                    if region.zloc != level as i32 {
                        continue;
                    }
                    let terrain = region.terrain_type.expect("surface terrain is assigned");
                    assert!(terrain < crate::defs::R_NUM, "{ruleset_name}, seed {seed}");
                    actual[terrain] += 1;
                }
            }

            assert_eq!(actual, expected, "{ruleset_name} terrain counts");
        }
    }
}
