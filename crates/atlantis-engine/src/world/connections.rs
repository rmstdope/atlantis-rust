use std::collections::{BTreeMap, VecDeque};

use crate::{
    defs::{
        objects::{O_GATEWAY, O_SHAFT},
        terrains::{TerrainDef, TerrainId, R_OCEAN, R_PLAIN, R_TUNDRA},
    },
    economy::SetupError,
    model::{Game, Object, Region, RegionArray, RegionId, RegionLevelType},
    rng::Rng,
    ruleset::Ruleset,
};

use super::{
    classic::{planar_neighbors, region_has_ocean_neighbor, region_is_coastal},
    names::NameGenerator,
    settlements::make_starting_city,
    WorldGenerationError,
};

/// C++: `ARegionList::MakeShaftLinks` and `ARegionList::MakeShaft`.
pub fn make_shaft_links(
    game: &mut Game,
    source_array: usize,
    target_array: usize,
    odds: i32,
    ruleset: &dyn Ruleset,
    rng: &mut Rng,
) -> Result<(), WorldGenerationError> {
    let source = game
        .region_arrays
        .get(source_array)
        .cloned()
        .ok_or(WorldGenerationError::MissingRegionArray(source_array))?;
    let target = game
        .region_arrays
        .get(target_array)
        .cloned()
        .ok_or(WorldGenerationError::MissingRegionArray(target_array))?;
    let terrain_defs = ruleset.tables().terrains;

    for x in 0..source.x {
        for y in 0..source.y {
            let Some(source_region) = source.get_region(x, y) else {
                continue;
            };
            if rng.get_random(odds) != 0 {
                continue;
            }
            if is_ocean_like(game, source_region, &terrain_defs)? {
                continue;
            }

            let target_x = game.region(source_region).expect("region exists").xloc
                * (target.x / source.x)
                + rng.get_random(target.x / source.x);
            let target_y = game.region(source_region).expect("region exists").yloc
                * (target.y / source.y)
                + rng.get_random(target.y / source.y);
            let target_y = target_y + (target_x + target_y) % 2;
            let Some(target_region) = target.get_region(target_x, target_y) else {
                continue;
            };
            if is_ocean_like(game, target_region, &terrain_defs)? {
                continue;
            }
            make_shaft_pair(
                game,
                source_region,
                target_region,
                ruleset.target_shaft_uses_source_region(),
                false,
                ruleset,
            )?;
        }
    }
    Ok(())
}

/// C++: `ARegionList::SetACNeighbors`.
pub fn set_nexus_neighbors(
    game: &mut Game,
    ruleset: &dyn Ruleset,
    rng: &mut Rng,
) -> Result<(), WorldGenerationError> {
    let nexus_index = find_level_array(game, RegionLevelType::Nexus)?;
    let surface_index = find_level_array(game, RegionLevelType::Surface)?;
    let nexus = game.region_arrays[nexus_index].clone();
    let surface = game.region_arrays[surface_index].clone();
    let definitions = ruleset.game_defs();
    let tables = ruleset.tables();
    let mut names = NameGenerator::new(ruleset.world_name_style());

    for x in 0..nexus.x {
        for y in 0..nexus.y {
            let Some(nexus_region) = nexus.get_region(x, y) else {
                continue;
            };
            if definitions.start_cities_exist != 0 {
                for direction in 0..6 {
                    if game
                        .region(nexus_region)
                        .expect("Nexus region exists")
                        .neighbors[direction]
                        .is_some()
                    {
                        continue;
                    }
                    let Some(starting_city) = get_starting_city(
                        game,
                        nexus_region,
                        direction,
                        &surface,
                        ruleset,
                        &tables.terrains,
                        rng,
                    )?
                    else {
                        continue;
                    };
                    game.region_mut(nexus_region)
                        .expect("Nexus region exists")
                        .neighbors[direction] = Some(starting_city);
                    let terrain_id = game
                        .region(starting_city)
                        .expect("starting city region exists")
                        .terrain_type
                        .ok_or(SetupError::MissingTerrain(starting_city))?;
                    let port = region_has_ocean_neighbor(game, starting_city, &tables.terrains);
                    let mut town_name = |rng: &mut Rng, _region: &Region| {
                        names.next_town_name(terrain_id, &tables.terrains, port, rng)
                    };
                    make_starting_city(
                        game,
                        starting_city,
                        definitions,
                        &tables,
                        rng,
                        &mut town_name,
                    )?;
                    if definitions.gates_exist != 0 {
                        game.number_of_gates = game.number_of_gates.wrapping_add(1);
                    }
                }
            } else {
                for terrain_id in R_PLAIN..=R_TUNDRA {
                    let Some(target_region) = first_region_of_terrain(game, &surface, terrain_id)?
                    else {
                        continue;
                    };
                    let num = next_building_number(game, nexus_region);
                    let mut gateway =
                        Object::new(definitions.phased_move_offset % definitions.max_speed);
                    gateway.name = Some(format!(
                        "Gateway to {} [{num}]",
                        terrain_defs_get(&tables.terrains, terrain_id)?.name
                    ));
                    gateway.num = num;
                    gateway.object_type = O_GATEWAY as i32;
                    gateway.inner = game.region(target_region).expect("target exists").num;
                    add_connection_object(game, nexus_region, nexus_region, gateway)?;
                }
            }
        }
    }
    Ok(())
}

/// C++: `ARegionList::InitSetupGates`.
pub fn init_setup_gates(
    game: &mut Game,
    array_index: usize,
    ruleset: &dyn Ruleset,
    rng: &mut Rng,
) -> Result<(), WorldGenerationError> {
    let array = game
        .region_arrays
        .get(array_index)
        .cloned()
        .ok_or(WorldGenerationError::MissingRegionArray(array_index))?;
    let definitions = ruleset.game_defs();
    if definitions.gates_exist == 0 {
        return Ok(());
    }
    let terrain_defs = ruleset.tables().terrains;

    for block_x in 0..array.x / 8 {
        for block_y in 0..array.y / 16 {
            for _ in 0..5 {
                let x = block_x * 8 + rng.get_random(8);
                let y = block_y * 16 + rng.get_random(8) * 2 + x % 2;
                let Some(region_id) = array.get_region(x, y) else {
                    continue;
                };
                let terrain_id = game
                    .region(region_id)
                    .expect("generated region exists")
                    .terrain_type
                    .ok_or(SetupError::MissingTerrain(region_id))?;
                let terrain = terrain_defs_get(&terrain_defs, terrain_id)?;
                if terrain.similar_type == R_OCEAN as i32
                    || game.region(region_id).expect("region exists").gate == -1
                {
                    continue;
                }
                game.number_of_gates = game.number_of_gates.wrapping_add(1);
                game.region_mut(region_id).expect("region exists").gate = -1;
                break;
            }
        }
    }
    Ok(())
}

/// C++: NewOrigins `ARegionList::FixUnconnectedRegions`.
pub fn fix_unconnected_regions(
    game: &mut Game,
    ruleset: &dyn Ruleset,
    rng: &mut Rng,
) -> Result<(), WorldGenerationError> {
    if !ruleset.fixes_unconnected_regions() {
        return Ok(());
    }

    let region_ids = game.region_ids().to_vec();
    let arrays = game.region_arrays.clone();
    let terrain_defs = ruleset.tables().terrains;
    let region_by_number = region_ids
        .iter()
        .copied()
        .map(|region_id| {
            (
                game.region(region_id).expect("region exists").num,
                region_id,
            )
        })
        .collect::<BTreeMap<_, _>>();

    for region_id in &region_ids {
        let region = game.region_mut(*region_id).expect("region exists");
        region.distance = -1;
        region.next = None;
    }

    let mut head = None;
    let mut tail = None;
    for region_id in &region_ids {
        let region = game.region(*region_id).expect("region exists");
        if region.zloc == RegionLevelType::Nexus as i32 || region.gate == -1 {
            game.region_mut(*region_id).expect("region exists").distance = 0;
            game.region_mut(*region_id).expect("region exists").next = head;
            head = Some(*region_id);
            tail.get_or_insert(*region_id);
        }
    }
    if let (Some(head), Some(tail)) = (head, tail) {
        flood_connected(game, head, tail, &region_by_number);
    }

    let mut attempts = 0;
    loop {
        let mut unconnected_count = 0;
        let mut remaining = 0;
        for region_id in &region_ids {
            let distance = game.region(*region_id).expect("region exists").distance;
            if distance == -1 {
                unconnected_count += 1;
            }
            if distance > remaining {
                remaining = distance;
            }
        }
        if unconnected_count == 0 || attempts >= 1000 {
            break;
        }

        let choice = rng.get_random(unconnected_count);
        let mut choice = choice;
        let region_id = region_ids
            .iter()
            .copied()
            .find(|region_id| {
                if game.region(*region_id).expect("region exists").distance != -1 {
                    return false;
                }
                if choice == 0 {
                    true
                } else {
                    choice -= 1;
                    false
                }
            })
            .expect("unconnected region count matches region list");
        let region = game.region(region_id).expect("region exists");
        let level_index = usize::try_from(region.zloc)
            .map_err(|_| WorldGenerationError::MissingRegionArray(usize::MAX))?;
        let array = arrays
            .get(level_index)
            .ok_or(WorldGenerationError::MissingRegionArray(level_index))?;
        let old_neighbors = region.neighbors;
        let new_neighbors = planar_neighbors(array, region.xloc, region.yloc);
        let offset = rng.get_random(6) as usize;
        let direction = (0..6).find(|index| {
            new_neighbors[(index + offset) % 6].is_some_and(|neighbor| {
                game.region(neighbor).expect("region exists").distance != -1
            })
        });

        let mut reachable = None;
        if let Some(direction) = direction {
            let direction = (direction + offset) % 6;
            let neighbor_id = new_neighbors[direction].expect("selected neighbor exists");
            let mut repaired_neighbors = old_neighbors;
            repaired_neighbors[direction] = Some(neighbor_id);
            game.region_mut(region_id).expect("region exists").neighbors = repaired_neighbors;

            let neighbor = game.region(neighbor_id).expect("region exists");
            let neighbor_array_index = usize::try_from(neighbor.zloc)
                .map_err(|_| WorldGenerationError::MissingRegionArray(usize::MAX))?;
            let neighbor_array = arrays.get(neighbor_array_index).ok_or(
                WorldGenerationError::MissingRegionArray(neighbor_array_index),
            )?;
            let new_neighbor_neighbors =
                planar_neighbors(neighbor_array, neighbor.xloc, neighbor.yloc);
            let old_neighbor_neighbors = neighbor.neighbors;
            let mut repaired_neighbor_neighbors = old_neighbor_neighbors;
            for index in 0..6 {
                if new_neighbor_neighbors[index] == Some(region_id) {
                    repaired_neighbor_neighbors[index] = Some(region_id);
                }
            }
            game.region_mut(neighbor_id)
                .expect("region exists")
                .neighbors = repaired_neighbor_neighbors;
            reachable = Some(neighbor_id);
        } else if !is_ocean_like(game, region_id, &terrain_defs)? {
            let level = game.region(region_id).expect("region exists").zloc;
            if level > RegionLevelType::Surface as i32 {
                reachable =
                    find_upper_shaft_target(game, region_id, &arrays, ruleset, &terrain_defs)?;
                if let Some(upper_region) = reachable {
                    make_shaft_pair(game, region_id, upper_region, false, true, ruleset)?;
                }
            }
            if reachable.is_none()
                && ruleset.game_defs().gates_exist != 0
                && rng.get_random(10) == 0
            {
                let region = game.region_mut(region_id).expect("region exists");
                region.gate = -1;
                region.distance = 0;
                reachable = Some(region_id);
            }
        }

        if let Some(reachable) = reachable {
            game.region_mut(reachable)
                .expect("reachable region exists")
                .next = None;
            flood_connected(game, reachable, reachable, &region_by_number);
            attempts = 0;
        }
        attempts += 1;
    }
    Ok(())
}

/// C++: `ARegionList::FinalSetupGates`.
pub fn final_setup_gates(
    game: &mut Game,
    ruleset: &dyn Ruleset,
    rng: &mut Rng,
) -> Result<(), WorldGenerationError> {
    if ruleset.game_defs().gates_exist == 0 {
        return Ok(());
    }

    let mut range = game.number_of_gates;
    if ruleset.game_defs().disperse_gate_numbers != 0 {
        let mut digits = 0;
        let mut count = range;
        while count > 0 {
            count /= 10;
            digits += 1;
        }
        range = 10;
        for _ in 0..digits {
            range = range.wrapping_mul(10);
        }
    }
    let has_gates = game
        .region_ids()
        .iter()
        .any(|region_id| game.region(*region_id).expect("region exists").gate == -1);
    if !has_gates {
        return Ok(());
    }
    if range <= 0 {
        return Err(WorldGenerationError::InvalidGateCount);
    }

    let mut used = vec![false; range as usize];
    for region_id in game.region_ids().to_vec() {
        if game.region(region_id).expect("region exists").gate != -1 {
            continue;
        }
        let mut index = rng.get_random(range);
        while used[index as usize] {
            if ruleset.game_defs().disperse_gate_numbers != 0 {
                index = rng.get_random(range);
            } else {
                index = (index + 1) % range;
            }
        }
        used[index as usize] = true;
        let region = game.region_mut(region_id).expect("region exists");
        region.gate = index + 1;
        region.gate_month = rng.get_random(12);
    }
    Ok(())
}

/// Runs the connection and gate setup sequence used by both original world generators.
pub fn connect_world(
    game: &mut Game,
    ruleset: &dyn Ruleset,
    rng: &mut Rng,
) -> Result<(), WorldGenerationError> {
    let definitions = ruleset.game_defs();
    let underworld_levels = definitions.underworld_levels;
    let underdeep_levels = definitions.underdeep_levels;
    let total_underground = underworld_levels + underdeep_levels;

    if total_underground == 1 {
        make_shaft_links(game, 2, 1, 8, ruleset, rng)?;
    } else if total_underground > 0 {
        make_shaft_links(game, 2, 1, 10, ruleset, rng)?;
        for level in 3..underworld_levels + 2 {
            make_shaft_links(game, level as usize, 1, 10 * level - 10, ruleset, rng)?;
        }
        if underworld_levels > 1 {
            for level in 3..underworld_levels + 2 {
                for target in 2..level {
                    let odds = if level == target + 1 { 12 } else { 24 };
                    make_shaft_links(game, level as usize, target as usize, odds, ruleset, rng)?;
                }
            }
        }
        if underdeep_levels > 0 && underworld_levels > 0 {
            make_shaft_links(
                game,
                (underworld_levels + 2) as usize,
                (underworld_levels + 1) as usize,
                12,
                ruleset,
                rng,
            )?;
        }
        if underdeep_levels > 1 {
            for level in underworld_levels + 3..underworld_levels + underdeep_levels + 2 {
                for target in underworld_levels + 2..level {
                    let odds = if level == target + 1 { 12 } else { 25 };
                    make_shaft_links(game, level as usize, target as usize, odds, ruleset, rng)?;
                }
            }
        }
    }

    set_nexus_neighbors(game, ruleset, rng)?;
    let surface = find_level_array(game, RegionLevelType::Surface)?;
    init_setup_gates(game, surface, ruleset, rng)?;
    for level in 2..underworld_levels + 2 {
        init_setup_gates(game, level as usize, ruleset, rng)?;
    }
    fix_unconnected_regions(game, ruleset, rng)?;
    final_setup_gates(game, ruleset, rng)
}

fn find_level_array(game: &Game, level: RegionLevelType) -> Result<usize, WorldGenerationError> {
    game.region_arrays
        .iter()
        .position(|array| array.level_type == level)
        .ok_or(WorldGenerationError::MissingRegionLevel(level))
}

fn first_region_of_terrain(
    game: &Game,
    array: &RegionArray,
    terrain_id: TerrainId,
) -> Result<Option<RegionId>, WorldGenerationError> {
    for x in 0..array.x {
        for y in 0..array.y {
            let Some(region_id) = array.get_region(x, y) else {
                continue;
            };
            if game
                .region(region_id)
                .expect("generated region exists")
                .terrain_type
                == Some(terrain_id)
            {
                return Ok(Some(region_id));
            }
        }
    }
    Ok(None)
}

fn get_starting_city(
    game: &Game,
    nexus_region: RegionId,
    direction: usize,
    surface: &RegionArray,
    ruleset: &dyn Ruleset,
    terrain_defs: &[TerrainDef],
    rng: &mut Rng,
) -> Result<Option<RegionId>, WorldGenerationError> {
    let max_x = surface.x;
    let max_y = surface.y;
    if max_x <= 0 || max_y < 2 {
        return Ok(None);
    }
    let minimum_distance = max_y / 10 + 2;
    for require_starting_city in [true, false] {
        for _ in 0..10_000 {
            let x = rng.get_random(max_x);
            let y = 2 * rng.get_random(max_y / 2) + x % 2;
            let Some(region_id) = surface.get_region(x, y) else {
                continue;
            };
            if require_starting_city {
                if !can_be_starting_city(game, region_id, ruleset, terrain_defs)? {
                    continue;
                }
            } else if game.region(region_id).expect("region exists").terrain_type == Some(R_OCEAN) {
                continue;
            }

            let too_close = (0..direction).any(|neighbor_direction| {
                game.region(nexus_region)
                    .expect("Nexus region exists")
                    .neighbors[neighbor_direction]
                    .is_some_and(|other| {
                        planar_distance(game, region_id, other, surface.x, ruleset)
                            < minimum_distance
                    })
            });
            if !too_close {
                return Ok(Some(region_id));
            }
        }
        if !require_starting_city {
            break;
        }
    }
    Ok(None)
}

fn can_be_starting_city(
    game: &Game,
    region_id: RegionId,
    ruleset: &dyn Ruleset,
    terrain_defs: &[TerrainDef],
) -> Result<bool, WorldGenerationError> {
    let region = game.region(region_id).expect("region exists");
    if region.terrain_type == Some(R_OCEAN)
        || !region_is_coastal(
            game,
            region_id,
            terrain_defs,
            ruleset.game_defs().lakeside_is_coastal != 0,
        )
        || region
            .town
            .as_ref()
            .is_some_and(|town| town.population == 5000)
    {
        return Ok(false);
    }

    let mut in_list = VecDeque::from([region_id]);
    let mut in_nums = vec![region.num];
    let mut done_nums = Vec::new();
    let mut region_count = 0;
    while let Some(current_id) = in_list.pop_front() {
        let current = game.region(current_id).expect("region exists");
        for neighbor_id in current.neighbors.iter().flatten() {
            let neighbor = game.region(*neighbor_id).expect("neighbor exists");
            if neighbor.terrain_type == Some(R_OCEAN)
                || in_nums.contains(&neighbor.num)
                || done_nums.contains(&neighbor.num)
            {
                continue;
            }
            region_count += 1;
            if region_count > 20 {
                return Ok(true);
            }
            in_nums.push(neighbor.num);
            in_list.push_back(*neighbor_id);
        }
        done_nums.push(current.num);
    }
    Ok(false)
}

fn planar_distance(
    game: &Game,
    one_id: RegionId,
    two_id: RegionId,
    surface_width: i32,
    ruleset: &dyn Ruleset,
) -> i32 {
    let one = game.region(one_id).expect("region exists");
    let two = game.region(two_id).expect("region exists");
    let one_x = one.xloc * ruleset.level_x_scale(one.zloc);
    let one_y = one.yloc * ruleset.level_y_scale(one.zloc);
    let two_x = two.xloc * ruleset.level_x_scale(two.zloc);
    let two_y = two.yloc * ruleset.level_y_scale(two.zloc);
    let max_y = (one_y - two_y).abs();
    let mut max_x = (one_x - two_x).abs();
    max_x = max_x.min((one_x + surface_width - two_x).abs());
    max_x = max_x.min((one_x - (two_x + surface_width)).abs());
    if max_y > max_x {
        max_x = (max_x + max_y) / 2;
    }
    max_x
}

fn find_upper_shaft_target(
    game: &Game,
    region_id: RegionId,
    arrays: &[RegionArray],
    ruleset: &dyn Ruleset,
    terrain_defs: &[TerrainDef],
) -> Result<Option<RegionId>, WorldGenerationError> {
    let region = game.region(region_id).expect("region exists");
    let level = region.zloc;
    let Some(upper_level) = level.checked_sub(1) else {
        return Ok(None);
    };
    let upper_index = usize::try_from(upper_level)
        .map_err(|_| WorldGenerationError::MissingRegionArray(usize::MAX))?;
    let upper = arrays
        .get(upper_index)
        .ok_or(WorldGenerationError::MissingRegionArray(upper_index))?;
    let xscale = ruleset.level_x_scale(level) / ruleset.level_x_scale(upper_level);
    let yscale = 2 * ruleset.level_y_scale(level) / ruleset.level_y_scale(upper_level);
    let x = region.xloc * ruleset.level_x_scale(level) / ruleset.level_x_scale(upper_level);
    let y = region.yloc * ruleset.level_y_scale(level) / ruleset.level_y_scale(upper_level);
    for offset_x in 0..xscale {
        for offset_y in 0..yscale {
            let Some(candidate) = upper.get_region(x + offset_x, y + offset_y) else {
                continue;
            };
            if !is_ocean_like(game, candidate, terrain_defs)? {
                return Ok(Some(candidate));
            }
        }
    }
    Ok(None)
}

fn flood_connected(
    game: &mut Game,
    first: RegionId,
    mut tail: RegionId,
    region_by_number: &BTreeMap<i32, RegionId>,
) {
    let mut current_id = Some(first);
    while let Some(region_id) = current_id {
        let distance = game.region(region_id).expect("region exists").distance;
        let neighbors = game.region(region_id).expect("region exists").neighbors;
        for neighbor_id in neighbors.into_iter().flatten() {
            enqueue_connected(game, neighbor_id, distance, &mut tail);
        }

        let object_ids = game
            .region(region_id)
            .expect("region exists")
            .object_ids()
            .to_vec();
        for object_id in object_ids {
            let inner = game.object(object_id).expect("object exists").inner;
            if inner != -1 {
                if let Some(inner_id) = region_by_number.get(&inner) {
                    enqueue_connected(game, *inner_id, distance, &mut tail);
                }
            }
        }
        current_id = game.region(region_id).expect("region exists").next;
    }
}

fn enqueue_connected(game: &mut Game, target: RegionId, distance: i32, tail: &mut RegionId) {
    if game.region(target).expect("region exists").distance != -1 {
        return;
    }
    game.region_mut(*tail).expect("tail region exists").next = Some(target);
    let region = game.region_mut(target).expect("target region exists");
    region.distance = distance.wrapping_add(1);
    region.next = None;
    *tail = target;
}

fn make_shaft_pair(
    game: &mut Game,
    source: RegionId,
    target: RegionId,
    target_uses_source: bool,
    target_first: bool,
    ruleset: &dyn Ruleset,
) -> Result<(), WorldGenerationError> {
    let defs = ruleset.game_defs();
    let move_points = defs.phased_move_offset % defs.max_speed;
    if target_first {
        add_shaft(
            game,
            target,
            target,
            game.region(source).expect("source exists").num,
            move_points,
        )?;
        add_shaft(
            game,
            source,
            source,
            game.region(target).expect("target exists").num,
            move_points,
        )?;
    } else {
        add_shaft(
            game,
            source,
            source,
            game.region(target).expect("target exists").num,
            move_points,
        )?;
        add_shaft(
            game,
            target,
            if target_uses_source { source } else { target },
            game.region(source).expect("source exists").num,
            move_points,
        )?;
    }
    Ok(())
}

fn add_shaft(
    game: &mut Game,
    storage_region: RegionId,
    owner_region: RegionId,
    inner: i32,
    move_points: i32,
) -> Result<(), WorldGenerationError> {
    let num = next_building_number(game, storage_region);
    let mut shaft = Object::new(move_points);
    shaft.name = Some(format!("Shaft [{num}]"));
    shaft.num = num;
    shaft.object_type = O_SHAFT as i32;
    shaft.inner = inner;
    add_connection_object(game, storage_region, owner_region, shaft)
}

fn add_connection_object(
    game: &mut Game,
    storage_region: RegionId,
    owner_region: RegionId,
    object: Object,
) -> Result<(), WorldGenerationError> {
    let object_id = game
        .add_object(storage_region, object)
        .map_err(|error| WorldGenerationError::RegionSetup(SetupError::Model(error)))?;
    game.object_mut(object_id)
        .expect("new object exists")
        .region = Some(owner_region);
    Ok(())
}

fn next_building_number(game: &mut Game, region_id: RegionId) -> i32 {
    let region = game.region_mut(region_id).expect("region exists");
    let number = region.building_seq;
    region.building_seq = region.building_seq.wrapping_add(1);
    number
}

fn is_ocean_like(
    game: &Game,
    region_id: RegionId,
    terrain_defs: &[TerrainDef],
) -> Result<bool, WorldGenerationError> {
    let terrain_id = game
        .region(region_id)
        .ok_or(SetupError::MissingRegion(region_id))?
        .terrain_type
        .ok_or(SetupError::MissingTerrain(region_id))?;
    Ok(terrain_defs_get(terrain_defs, terrain_id)?.similar_type == R_OCEAN as i32)
}

fn terrain_defs_get(
    terrain_defs: &[TerrainDef],
    terrain_id: TerrainId,
) -> Result<&TerrainDef, WorldGenerationError> {
    terrain_defs
        .get(terrain_id)
        .ok_or(WorldGenerationError::RegionSetup(
            SetupError::InvalidTerrain(terrain_id as i32),
        ))
}

#[cfg(test)]
mod tests {
    use crate::{
        defs::ItemId,
        defs::{
            objects::{O_GATEWAY, O_SHAFT},
            terrains::{R_FOREST, R_JUNGLE, R_MOUNTAIN, R_PLAIN, R_SWAMP, R_TUNDRA},
        },
        model::{Game, RegionLevelType},
        rng::Rng,
        ruleset::Ruleset,
    };

    use super::{
        connect_world, final_setup_gates, fix_unconnected_regions, init_setup_gates,
        make_shaft_links, set_nexus_neighbors,
    };
    use crate::defs::terrains::R_VOLCANO;
    use crate::world::classic::{
        create_classic_surface_level, create_nexus_level, create_region_array,
        create_underdeep_level, create_underworld_level,
    };

    #[test]
    fn shaft_links_create_reciprocal_objects_in_cpp_order() {
        for ruleset_name in ["standard", "neworigins"] {
            let ruleset = <dyn Ruleset>::from_name(ruleset_name).expect("supported ruleset");
            let mut game = Game::default();
            let surface = create_region_array(&mut game, RegionLevelType::Surface, "surface", 8, 8)
                .expect("surface array");
            let underworld =
                create_region_array(&mut game, RegionLevelType::Underworld, "underworld", 8, 8)
                    .expect("underworld array");
            let region_ids = game.region_ids().to_vec();
            for region_id in region_ids {
                game.region_mut(region_id).unwrap().terrain_type = Some(R_PLAIN);
            }

            make_shaft_links(
                &mut game,
                surface,
                underworld,
                1,
                ruleset,
                &mut Rng::new(424242),
            )
            .expect("shaft links");

            for y in 0..8 {
                for x in 0..8 {
                    let Some(source_id) = game.region_arrays[surface].get_region(x, y) else {
                        continue;
                    };
                    let target_id = game.region_arrays[underworld]
                        .get_region(x, y)
                        .expect("matching target coordinate");
                    let source = game.region(source_id).unwrap();
                    let target = game.region(target_id).unwrap();
                    assert_eq!(source.object_ids().len(), 1, "{ruleset_name} ({x},{y})");
                    assert_eq!(target.object_ids().len(), 1, "{ruleset_name} ({x},{y})");

                    let source_shaft = game.object(source.object_ids()[0]).unwrap();
                    let target_shaft = game.object(target.object_ids()[0]).unwrap();
                    assert_eq!(source_shaft.object_type, O_SHAFT as i32);
                    assert_eq!(target_shaft.object_type, O_SHAFT as i32);
                    assert_eq!(source_shaft.inner, target.num);
                    assert_eq!(target_shaft.inner, source.num);
                    assert_eq!(source_shaft.region, Some(source_id));
                    let expected_target_owner = if ruleset_name == "standard" {
                        source_id
                    } else {
                        target_id
                    };
                    assert_eq!(target_shaft.region, Some(expected_target_owner));
                }
            }
        }
    }

    #[test]
    fn scaled_shaft_links_number_shared_targets_in_source_call_order() {
        let ruleset = <dyn Ruleset>::from_name("standard").expect("standard ruleset");
        let mut game = Game::default();
        let surface = create_region_array(&mut game, RegionLevelType::Surface, "surface", 8, 8)
            .expect("surface array");
        let small_underworld =
            create_region_array(&mut game, RegionLevelType::Underworld, "small", 4, 4)
                .expect("scaled underworld array");
        let large_underworld =
            create_region_array(&mut game, RegionLevelType::Underworld, "large", 8, 8)
                .expect("full-size underworld array");
        for region_id in game.region_ids().to_vec() {
            game.region_mut(region_id).unwrap().terrain_type = Some(R_PLAIN);
        }

        let mut rng = Rng::new(424242);
        make_shaft_links(&mut game, small_underworld, surface, 1, ruleset, &mut rng)
            .expect("scaled shafts");
        make_shaft_links(&mut game, large_underworld, surface, 1, ruleset, &mut rng)
            .expect("full-size shafts");

        let small_source_array = game.region_arrays[small_underworld].clone();
        let large_source_array = game.region_arrays[large_underworld].clone();
        let surface_array = game.region_arrays[surface].clone();
        let mut small_source_nums = Vec::new();
        for y in 0..small_source_array.y {
            for x in 0..small_source_array.x {
                if let Some(region_id) = small_source_array.get_region(x, y) {
                    small_source_nums.push(game.region(region_id).unwrap().num);
                }
            }
        }
        let mut shared_targets = 0;
        for y in 0..surface_array.y {
            for x in 0..surface_array.x {
                let Some(target_region_id) = surface_array.get_region(x, y) else {
                    continue;
                };
                let target = game.region(target_region_id).unwrap();
                let objects = target
                    .object_ids()
                    .iter()
                    .map(|object_id| game.object(*object_id).unwrap())
                    .collect::<Vec<_>>();
                assert!((1..=2).contains(&objects.len()));
                assert!(objects
                    .iter()
                    .all(|object| object.object_type == O_SHAFT as i32));
                let matching_large_source = large_source_array
                    .get_region(target.xloc, target.yloc)
                    .expect("same-sized source region");
                assert_eq!(
                    objects.last().unwrap().inner,
                    game.region(matching_large_source).unwrap().num
                );
                if objects.len() == 1 {
                    assert_eq!(objects[0].num, 1);
                } else {
                    shared_targets += 1;
                    assert_eq!(
                        objects.iter().map(|object| object.num).collect::<Vec<_>>(),
                        [1, 2]
                    );
                    assert!(small_source_nums.contains(&objects[0].inner));
                }
            }
        }
        assert!(
            shared_targets > 0,
            "expected scaled shafts to share targets"
        );
    }

    #[test]
    fn nexus_connections_create_expected_gateways_for_each_terrain() {
        let ruleset = <dyn Ruleset>::from_name("neworigins").expect("neworigins ruleset");
        let mut game = Game::default();
        let nexus = create_region_array(&mut game, RegionLevelType::Nexus, "nexus", 1, 1)
            .expect("Nexus array");
        let _surface = create_region_array(&mut game, RegionLevelType::Surface, "surface", 16, 16)
            .expect("surface array");
        let terrain_ids = [R_PLAIN, R_FOREST, R_MOUNTAIN, R_SWAMP, R_JUNGLE, R_TUNDRA];
        let surface_ids = game.region_ids()[1..].to_vec();
        for (terrain, region_id) in terrain_ids.into_iter().zip(surface_ids) {
            game.region_mut(region_id).unwrap().terrain_type = Some(terrain);
        }
        set_nexus_neighbors(&mut game, ruleset, &mut Rng::new(424242)).expect("Nexus connections");

        let nexus_id = game.region_arrays[nexus].get_region(0, 0).unwrap();
        let nexus_region = game.region(nexus_id).unwrap();
        assert_eq!(nexus_region.object_ids().len(), terrain_ids.len());
        assert_eq!(nexus_region.neighbors, [None; 6]);
        for (object_id, terrain) in nexus_region.object_ids().iter().copied().zip(terrain_ids) {
            let object = game.object(object_id).unwrap();
            let target = game
                .region_ids()
                .iter()
                .copied()
                .find(|region_id| game.region(*region_id).unwrap().terrain_type == Some(terrain))
                .unwrap();
            assert_eq!(object.object_type, O_GATEWAY as i32);
            assert_eq!(object.inner, game.region(target).unwrap().num);
        }
        assert_eq!(game.number_of_gates, 0);
    }

    #[test]
    fn standard_nexus_connections_create_spaced_starting_cities() {
        let ruleset = <dyn Ruleset>::from_name("standard").expect("standard ruleset");
        let mut game = Game::default();
        let nexus = create_region_array(&mut game, RegionLevelType::Nexus, "nexus", 1, 1)
            .expect("Nexus array");
        let surface = create_region_array(&mut game, RegionLevelType::Surface, "surface", 64, 64)
            .expect("surface array");
        for region_id in game.region_ids().to_vec() {
            let region = game.region_mut(region_id).unwrap();
            region.terrain_type = Some(R_PLAIN);
            region.race = ItemId::Man.index() as i32;
        }
        for y in 0..64 {
            if let Some(region_id) = game.region_arrays[surface].get_region(0, y) {
                game.region_mut(region_id).unwrap().terrain_type =
                    Some(crate::defs::terrains::R_OCEAN);
            }
        }

        set_nexus_neighbors(&mut game, ruleset, &mut Rng::new(123456)).expect("Nexus connections");

        let nexus_id = game.region_arrays[nexus].get_region(0, 0).unwrap();
        let neighbors = game.region(nexus_id).unwrap().neighbors;
        assert!(neighbors.iter().all(Option::is_some));
        let cities = neighbors.into_iter().flatten().collect::<Vec<_>>();
        assert_eq!(cities.len(), 6);
        assert_eq!(
            cities
                .iter()
                .copied()
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            6
        );
        assert!(cities.iter().all(|region_id| {
            game.region(*region_id)
                .unwrap()
                .town
                .as_ref()
                .is_some_and(|town| town.population >= 5000)
        }));
        assert_eq!(game.number_of_gates, 6);
    }

    #[test]
    fn shaft_links_preserve_cpp_random_draw_count() {
        let ruleset = <dyn Ruleset>::from_name("standard").expect("standard ruleset");
        let mut game = Game::default();
        let source =
            create_region_array(&mut game, RegionLevelType::Underworld, "underworld", 8, 8)
                .expect("underworld array");
        let target = create_region_array(&mut game, RegionLevelType::Surface, "surface", 8, 8)
            .expect("surface array");
        for region_id in game.region_ids().to_vec() {
            game.region_mut(region_id).unwrap().terrain_type = Some(R_PLAIN);
        }

        let mut actual = Rng::new(921);
        let mut expected = actual.clone();
        make_shaft_links(&mut game, source, target, 1, ruleset, &mut actual).expect("shaft links");
        for _ in 0..32 {
            expected.get_random(1);
            expected.get_random(1);
            expected.get_random(1);
        }
        assert_eq!(actual.get_random(1000), expected.get_random(1000));
    }

    #[test]
    fn gate_setup_allows_volcanoes_in_both_rulesets() {
        for (ruleset_name, expected_gates) in [("standard", 1), ("neworigins", 1)] {
            let ruleset = <dyn Ruleset>::from_name(ruleset_name).unwrap();
            let mut game = Game::default();
            let surface =
                create_region_array(&mut game, RegionLevelType::Surface, "surface", 8, 16)
                    .expect("surface array");
            for region_id in game.region_ids().to_vec() {
                game.region_mut(region_id).unwrap().terrain_type = Some(R_VOLCANO);
            }

            init_setup_gates(&mut game, surface, ruleset, &mut Rng::new(921)).expect("gate setup");

            assert_eq!(game.number_of_gates, expected_gates, "{ruleset_name}");
            assert_eq!(
                game.region_ids()
                    .iter()
                    .filter(|id| game.region(**id).unwrap().gate == -1)
                    .count() as i32,
                expected_gates,
                "{ruleset_name}"
            );
        }
    }

    #[test]
    fn final_gate_setup_assigns_each_reserved_number_once() {
        let ruleset = <dyn Ruleset>::from_name("standard").unwrap();
        let mut game = Game::default();
        create_region_array(&mut game, RegionLevelType::Surface, "surface", 8, 8)
            .expect("surface array");
        let gated = game.region_ids()[..3].to_vec();
        for region_id in &gated {
            game.region_mut(*region_id).unwrap().gate = -1;
        }
        game.number_of_gates = gated.len() as i32;

        final_setup_gates(&mut game, ruleset, &mut Rng::new(921)).expect("final gate setup");

        let mut numbers = gated
            .iter()
            .map(|region_id| game.region(*region_id).unwrap().gate)
            .collect::<Vec<_>>();
        numbers.sort_unstable();
        assert_eq!(numbers, [1, 2, 3]);
        assert!(gated
            .iter()
            .all(|region_id| { (0..12).contains(&game.region(*region_id).unwrap().gate_month) }));
    }

    #[test]
    fn neworigins_connectivity_repair_opens_one_wall_reciprocally() {
        let ruleset = <dyn Ruleset>::from_name("neworigins").unwrap();
        let mut game = Game::default();
        let nexus = create_region_array(&mut game, RegionLevelType::Nexus, "nexus", 1, 1)
            .expect("Nexus array");
        let surface = create_region_array(&mut game, RegionLevelType::Surface, "surface", 8, 8)
            .expect("surface array");
        for region_id in game.region_ids().to_vec() {
            game.region_mut(region_id).unwrap().terrain_type = Some(R_PLAIN);
        }
        let nexus_id = game.region_arrays[nexus].get_region(0, 0).unwrap();
        let target_id = game.region_arrays[surface].get_region(0, 0).unwrap();
        let isolated_id = game.region_arrays[surface].get_region(4, 4).unwrap();
        let mut gateway = crate::model::Object::default();
        gateway.object_type = O_GATEWAY as i32;
        gateway.inner = game.region(target_id).unwrap().num;
        game.add_object(nexus_id, gateway).expect("gateway");
        for region_id in game.region_ids().to_vec() {
            let region = game.region_mut(region_id).unwrap();
            if region.neighbors.contains(&Some(isolated_id)) {
                for neighbor in &mut region.neighbors {
                    if *neighbor == Some(isolated_id) {
                        *neighbor = None;
                    }
                }
            }
        }
        game.region_mut(isolated_id).unwrap().neighbors = [None; 6];

        fix_unconnected_regions(&mut game, ruleset, &mut Rng::new(921))
            .expect("connectivity repair");

        let isolated = game.region(isolated_id).unwrap();
        assert!(isolated.neighbors.iter().flatten().any(|neighbor| {
            game.region(*neighbor)
                .unwrap()
                .neighbors
                .contains(&Some(isolated_id))
        }));
        assert!(game.region_ids().iter().all(|region_id| game
            .region(*region_id)
            .unwrap()
            .distance
            >= 0));
    }

    #[test]
    fn neworigins_connectivity_repair_owns_upper_shaft_in_upper_region() {
        let ruleset = <dyn Ruleset>::from_name("neworigins").unwrap();
        let mut game = Game::default();
        let nexus = create_region_array(&mut game, RegionLevelType::Nexus, "nexus", 1, 1)
            .expect("Nexus array");
        let surface = create_region_array(&mut game, RegionLevelType::Surface, "surface", 8, 8)
            .expect("surface array");
        let underworld =
            create_region_array(&mut game, RegionLevelType::Underworld, "underworld", 8, 8)
                .expect("underworld array");
        for region_id in game.region_ids().to_vec() {
            game.region_mut(region_id).unwrap().terrain_type = Some(R_PLAIN);
        }
        let nexus_id = game.region_arrays[nexus].get_region(0, 0).unwrap();
        let surface_id = game.region_arrays[surface].get_region(0, 0).unwrap();
        let underworld_id = game.region_arrays[underworld].get_region(0, 0).unwrap();
        let mut gateway = crate::model::Object::default();
        gateway.object_type = O_GATEWAY as i32;
        gateway.inner = game.region(surface_id).unwrap().num;
        game.add_object(nexus_id, gateway).expect("gateway");
        let target_rank = game
            .region_ids()
            .iter()
            .filter(|region_id| game.region(**region_id).unwrap().zloc == 2)
            .position(|region_id| *region_id == underworld_id)
            .expect("underworld region is in the region list");
        let seed = (0..10_000)
            .find(|seed| {
                let mut rng = Rng::new(*seed);
                rng.get_random(64) as usize == target_rank
            })
            .expect("find deterministic shaft-fallback seed");

        fix_unconnected_regions(&mut game, ruleset, &mut Rng::new(seed))
            .expect("connectivity repair");

        let upper_shaft = game
            .region(surface_id)
            .unwrap()
            .object_ids()
            .first()
            .and_then(|object_id| game.object(*object_id))
            .expect("upper shaft");
        let lower_shaft = game
            .region(underworld_id)
            .unwrap()
            .object_ids()
            .first()
            .and_then(|object_id| game.object(*object_id))
            .expect("lower shaft");
        assert_eq!(upper_shaft.object_type, O_SHAFT as i32);
        assert_eq!(upper_shaft.region, Some(surface_id));
        assert_eq!(upper_shaft.inner, game.region(underworld_id).unwrap().num);
        assert_eq!(lower_shaft.object_type, O_SHAFT as i32);
        assert_eq!(lower_shaft.region, Some(underworld_id));
        assert_eq!(lower_shaft.inner, game.region(surface_id).unwrap().num);
    }

    #[test]
    fn neworigins_surface_repair_uses_gate_fallback_not_a_nexus_shaft() {
        let ruleset = <dyn Ruleset>::from_name("neworigins").unwrap();
        let mut game = Game::default();
        let nexus = create_region_array(&mut game, RegionLevelType::Nexus, "nexus", 1, 1)
            .expect("Nexus array");
        create_region_array(&mut game, RegionLevelType::Surface, "surface", 8, 8)
            .expect("surface array");
        for region_id in game.region_ids().to_vec() {
            game.region_mut(region_id).unwrap().terrain_type = Some(R_PLAIN);
        }
        let seed = (0..10_000)
            .find(|seed| {
                let mut rng = Rng::new(*seed);
                rng.get_random(64);
                rng.get_random(6);
                rng.get_random(10) == 0
            })
            .expect("find deterministic gate-fallback seed");

        fix_unconnected_regions(&mut game, ruleset, &mut Rng::new(seed))
            .expect("connectivity repair");

        let surface_regions = game
            .region_ids()
            .iter()
            .copied()
            .filter(|region_id| game.region(*region_id).unwrap().zloc == 1)
            .collect::<Vec<_>>();
        assert!(surface_regions
            .iter()
            .any(|region_id| game.region(*region_id).unwrap().gate == -1));
        assert_eq!(
            game.number_of_gates, 0,
            "NewOrigins C++ leaves its global count unchanged for repair-created gates"
        );
        assert!(game.region_ids().iter().all(|region_id| game
            .region(*region_id)
            .unwrap()
            .object_ids()
            .is_empty()));
        assert!(game.region_ids().iter().all(|region_id| game
            .region(*region_id)
            .unwrap()
            .distance
            >= 0));
        let nexus_id = game.region_arrays[nexus].get_region(0, 0).unwrap();
        assert!(game.region(nexus_id).unwrap().object_ids().is_empty());

        final_setup_gates(&mut game, ruleset, &mut Rng::new(seed))
            .expect("finalize repair-created gate");
        assert_eq!(game.number_of_gates, 0);
        assert_eq!(
            surface_regions
                .iter()
                .filter(|region_id| game.region(**region_id).unwrap().gate > 0)
                .count(),
            1
        );
    }

    #[test]
    fn classic_connections_match_cpp_reference_and_connectivity_invariants() {
        const CONNECTION_FIXTURE: &str =
            include_str!("../../../../fixtures/world-generation/classic-connections.json");
        let fixture: serde_json::Value =
            serde_json::from_str(CONNECTION_FIXTURE).expect("connection fixture is valid JSON");
        assert_eq!(
            fixture["format"].as_str(),
            Some("atlantis-classic-world-connections-v1")
        );
        assert_eq!(fixture["seed"].as_i64(), Some(1));
        assert_eq!(fixture["width"].as_i64(), Some(64));
        assert_eq!(fixture["height"].as_i64(), Some(64));

        for ruleset_name in ["standard", "neworigins"] {
            let ruleset = <dyn Ruleset>::from_name(ruleset_name).unwrap();
            let definitions = ruleset.game_defs();
            let mut game = Game::default();
            let mut rng = Rng::new(1);
            create_nexus_level(&mut game, ruleset, &mut rng, 1, 1).expect("Nexus level");
            create_classic_surface_level(&mut game, ruleset, &mut rng, 64, 64)
                .expect("classic surface");
            for level in 2..definitions.underworld_levels + 2 {
                create_underworld_level(
                    &mut game,
                    ruleset,
                    &mut rng,
                    64 / ruleset.level_x_scale(level),
                    64 / ruleset.level_y_scale(level),
                )
                .expect("underworld level");
            }
            for level in definitions.underworld_levels + 2
                ..definitions.underworld_levels + definitions.underdeep_levels + 2
            {
                create_underdeep_level(
                    &mut game,
                    ruleset,
                    &mut rng,
                    64 / ruleset.level_x_scale(level),
                    64 / ruleset.level_y_scale(level),
                )
                .expect("underdeep level");
            }
            connect_world(&mut game, ruleset, &mut rng).expect("world connections");

            let actual = connection_records(&game);
            let expected = fixture["rulesets"][ruleset_name]["records"]
                .as_array()
                .expect("ruleset connection records");
            assert_eq!(actual.len(), expected.len(), "{ruleset_name} record count");
            let reference_summary = expected[0]
                .as_str()
                .expect("reference summary record")
                .split_whitespace()
                .collect::<Vec<_>>();
            assert_eq!(reference_summary[0], "WORLDGEN_CONNECTION_SUMMARY");
            assert_eq!(
                game.region_ids().len(),
                reference_summary[1].parse::<usize>().expect("region count"),
                "{ruleset_name} region count"
            );
            assert_eq!(
                game.number_of_gates,
                reference_summary[2].parse::<i32>().expect("gate count"),
                "{ruleset_name} gate count"
            );

            if ruleset_name == "neworigins" {
                // NewOrigins world generation is structural/statistical, not a byte-level oracle.
                let unreachable = game
                    .region_ids()
                    .iter()
                    .filter(|region_id| game.region(**region_id).unwrap().distance < 0)
                    .map(|region_id| game.region(*region_id).unwrap().num)
                    .collect::<Vec<_>>();
                assert!(
                    unreachable.is_empty(),
                    "NewOrigins left regions unreachable: {:?}",
                    &unreachable[..unreachable.len().min(12)]
                );

                let mut gate_numbers = game
                    .region_ids()
                    .iter()
                    .filter_map(|region_id| {
                        let gate = game.region(*region_id).unwrap().gate;
                        (gate > 0).then_some(gate)
                    })
                    .collect::<Vec<_>>();
                gate_numbers.sort_unstable();
                assert_eq!(gate_numbers.len(), game.number_of_gates as usize);
                assert!(
                    gate_numbers.iter().all(|gate| (1..=1000).contains(gate)),
                    "NewOrigins dispersed gate number outside 1..=1000: {:?}",
                    gate_numbers
                );
                assert!(
                    gate_numbers.windows(2).all(|pair| pair[0] < pair[1]),
                    "NewOrigins gate numbers are not unique: {:?}",
                    gate_numbers
                );
                assert!(
                    game.region_ids()
                        .iter()
                        .all(|region_id| game.region(*region_id).unwrap().gate >= 0),
                    "NewOrigins left a reserved gate number unassigned"
                );
                continue;
            }

            let differences = actual
                .iter()
                .zip(expected)
                .enumerate()
                .filter_map(|(index, (actual, expected))| {
                    let expected = expected.as_str().expect("reference record is a string");
                    (actual != expected).then(|| format!("{index}: {actual} != {expected}"))
                })
                .take(12)
                .collect::<Vec<_>>();
            assert!(
                differences.is_empty(),
                "{ruleset_name} connection fixture differences:\n{}\nactual-only objects: {:?}\nreference-only objects: {:?}",
                differences.join("\n"),
                actual
                    .iter()
                    .filter(|record| record.starts_with("WORLDGEN_CONNECTION_OBJECT"))
                    .filter(|record| {
                        !expected
                            .iter()
                            .any(|expected| expected.as_str() == Some(record.as_str()))
                    })
                    .collect::<Vec<_>>(),
                expected
                    .iter()
                    .filter(|record| {
                        record
                            .as_str()
                            .is_some_and(|record| record.starts_with("WORLDGEN_CONNECTION_OBJECT"))
                    })
                    .filter(|record| {
                        !actual.iter().any(|actual| {
                            actual.as_str()
                                == record.as_str().expect("reference record is a string")
                        })
                    })
                    .collect::<Vec<_>>()
            );
        }
    }

    fn connection_records(game: &Game) -> Vec<String> {
        let mut records = vec![format!(
            "WORLDGEN_CONNECTION_SUMMARY {} {}",
            game.region_ids().len(),
            game.number_of_gates
        )];
        for region_id in game.region_ids() {
            let region = game.region(*region_id).expect("region exists");
            let neighbors = region
                .neighbors
                .iter()
                .map(|neighbor| {
                    neighbor.map_or(-1, |neighbor| {
                        game.region(neighbor).expect("neighbor exists").num
                    })
                })
                .map(|number| number.to_string())
                .collect::<Vec<_>>()
                .join(" ");
            records.push(format!(
                "WORLDGEN_CONNECTION_REGION {} {} {} {} {} {} {} {}",
                region.num,
                region.zloc,
                region.xloc,
                region.yloc,
                region.terrain_type.expect("terrain exists"),
                region.gate,
                region.gate_month,
                neighbors
            ));
            for object_id in region.object_ids() {
                let object = game.object(*object_id).expect("object exists");
                if object.object_type != O_SHAFT as i32 && object.object_type != O_GATEWAY as i32 {
                    continue;
                }
                let owner = object
                    .region
                    .and_then(|owner| game.region(owner))
                    .map_or(-1, |owner| owner.num);
                records.push(format!(
                    "WORLDGEN_CONNECTION_OBJECT {} {} {} {} {}",
                    region.num, object.object_type, object.num, object.inner, owner
                ));
            }
        }
        records
    }
}
