use std::{
    cmp::Reverse,
    collections::{BTreeMap, BTreeSet, BinaryHeap},
};

use crate::{
    defs::{
        objects::{
            O_CASTLE, O_FORT, O_INN, O_ROADN, O_ROADNE, O_ROADNW, O_ROADS, O_ROADSE, O_ROADSW,
            O_TOWER,
        },
        terrains::{
            terrain_flags, R_DESERT, R_FOREST, R_JUNGLE, R_MOUNTAIN, R_OCEAN, R_PLAIN, R_SWAMP,
            R_TUNDRA, R_VOLCANO,
        },
        GameDefs, RulesetTables,
    },
    economy::manual_setup,
    model::{Game, Object, RegionArray, RegionId},
    rng::Rng,
    world::{
        classic::{region_has_ocean_neighbor, region_is_coastal},
        settlements::{town_type_value, TownType},
        WorldGenerationError,
    },
};

use super::names::NaturalNameGenerator;

const SETTLEMENT_CANDIDATES_PER_POINT: i32 = 16;
const ROAD_OBJECTS: [usize; 6] = [O_ROADN, O_ROADNE, O_ROADSE, O_ROADS, O_ROADSW, O_ROADNW];

/// C++: `mapgen.cpp` `economy`; settlements use its Poisson-distributed candidate sites.
pub(super) fn generate(
    game: &mut Game,
    array: &RegionArray,
    defs: &GameDefs,
    tables: &RulesetTables,
    names: &mut NaturalNameGenerator,
    rng: &mut Rng,
) -> Result<(), WorldGenerationError> {
    let mut town_size = rng.get_random(3);
    let initial_min_distance = town_size + rng.make_roll(2, 2);
    let mut visited = BTreeSet::new();

    get_points(
        array.x,
        array.y,
        initial_min_distance,
        SETTLEMENT_CANDIDATES_PER_POINT,
        rng,
        |(x, y), current_min_distance, rng| {
            let Some(region_id) = array.get_region(x, y) else {
                return Ok(current_min_distance);
            };
            let region = game.region(region_id).expect("surface region exists");
            let terrain_id = region
                .terrain_type
                .expect("natural surface region has a terrain");
            let terrain = tables
                .terrains
                .get(terrain_id)
                .expect("natural surface terrain has a definition");
            if terrain_id == R_OCEAN
                || terrain_id == R_VOLCANO
                || terrain.flags & terrain_flags::BARREN != 0
            {
                return Ok(current_min_distance);
            }

            let port = region_has_ocean_neighbor(game, region_id, &tables.terrains);
            let town_name = names.next_town_name(terrain_id, port, tables, rng);
            let settlement_size =
                TownType::try_from(town_size).expect("random settlement size is a valid town type");
            let is_coastal = region_is_coastal(
                game,
                region_id,
                &tables.terrains,
                defs.lakeside_is_coastal != 0,
            );
            manual_setup(
                game,
                region_id,
                terrain_id,
                terrain.population + 1,
                1.0,
                false,
                Some((settlement_size, town_name)),
                defs,
                tables,
                is_coastal,
                rng,
            )?;
            visited.insert(region_id);

            town_size = rng.get_random(3);
            Ok(town_size + rng.make_roll(2, 2))
        },
    )?;

    for x in 0..array.x {
        for y in 0..array.y {
            let Some(region_id) = array.get_region(x, y) else {
                continue;
            };
            if visited.contains(&region_id) {
                continue;
            }
            let terrain_id = game
                .region(region_id)
                .and_then(|region| region.terrain_type)
                .expect("natural surface region has a terrain");
            let terrain = tables
                .terrains
                .get(terrain_id)
                .expect("natural surface terrain has a definition");
            let add_lair = rng.get_random(100) < terrain.lair_chance;
            let is_coastal = region_is_coastal(
                game,
                region_id,
                &tables.terrains,
                defs.lakeside_is_coastal != 0,
            );
            manual_setup(
                game,
                region_id,
                terrain_id,
                terrain.population + 1,
                1.0,
                add_lair,
                None,
                defs,
                tables,
                is_coastal,
                rng,
            )?;
        }
    }
    Ok(())
}

/// C++: `aregion.cpp` `AddHistoricalBuildings`.
pub(super) fn add_historical_buildings(
    game: &mut Game,
    array: &RegionArray,
    defs: &GameDefs,
    tables: &RulesetTables,
    names: &mut NaturalNameGenerator,
    rng: &mut Rng,
) {
    let mut cities = Vec::new();
    for x in 0..array.x {
        for y in 0..array.y {
            let Some(region_id) = array.get_region(x, y) else {
                continue;
            };
            let (terrain_id, population, town) = {
                let region = game.region(region_id).expect("surface region exists");
                if let Some(town) = &region.town {
                    if town_type_value(town, defs.city_pop) == TownType::City as i32 {
                        cities.push(region_id);
                    }
                }
                (
                    region
                        .terrain_type
                        .expect("natural surface region has a terrain"),
                    region.population,
                    region.town.clone(),
                )
            };
            let terrain = tables
                .terrains
                .get(terrain_id)
                .expect("natural surface terrain has a definition");
            if terrain_id == R_OCEAN
                || terrain_id == R_VOLCANO
                || terrain.flags & terrain_flags::BARREN != 0
            {
                continue;
            }

            if let Some(town) = town {
                if town.population > 8_000 {
                    if rng.make_roll(3, 6) >= 12 {
                        let damage = f64::from(rng.make_roll(2, 6) - 1) / 12.0;
                        let name = names.next_object_name(terrain_id, O_CASTLE, tables, rng);
                        add_ancient_structure(game, region_id, name, O_CASTLE, damage, tables);
                    }
                } else if town.population > 4_000 {
                    if rng.make_roll(3, 6) >= 14 {
                        let damage = f64::from(rng.make_roll(2, 6) - 1) / 12.0;
                        let name = names.next_object_name(terrain_id, O_FORT, tables, rng);
                        add_ancient_structure(game, region_id, name, O_FORT, damage, tables);
                    }
                } else if town.population > 2_000 && rng.make_roll(3, 6) >= 16 {
                    let damage = f64::from(rng.make_roll(2, 6) - 1) / 12.0;
                    let name = names.next_object_name(terrain_id, O_TOWER, tables, rng);
                    add_ancient_structure(game, region_id, name, O_TOWER, damage, tables);
                }

                let level = town_type_value(&town, defs.city_pop);
                let roll = rng.make_roll(level + 1, 6);
                let count = roll / 6;
                let damage = 6 - roll % 6;
                for index in 0..count {
                    let damage_points = if index == count - 1 {
                        if damage >= 3 {
                            break;
                        }
                        f64::from(damage) / 6.0
                    } else {
                        0.0
                    };
                    let name = names.next_object_name(terrain_id, O_INN, tables, rng);
                    add_ancient_structure(game, region_id, name, O_INN, damage_points, tables);
                }
            } else if population > 2_000 {
                if rng.make_roll(3, 6) >= 16 {
                    let damage = f64::from(rng.make_roll(2, 6) - 1) / 12.0;
                    let name = names.next_object_name(terrain_id, O_FORT, tables, rng);
                    add_ancient_structure(game, region_id, name, O_FORT, damage, tables);
                }
            } else if population > 1_000 && rng.make_roll(3, 6) >= 17 {
                let damage = f64::from(rng.make_roll(2, 6) - 1) / 12.0;
                let name = names.next_object_name(terrain_id, O_TOWER, tables, rng);
                add_ancient_structure(game, region_id, name, O_TOWER, damage, tables);
            }
        }
    }

    connect_cities(game, array, &cities, tables, rng);
}

/// C++: `mapgen.cpp` `getPoints`; preserves candidate attempt counts and minimum-distance updates.
fn get_points<F>(
    width: i32,
    height: i32,
    initial_min_distance: i32,
    new_point_count: i32,
    rng: &mut Rng,
    mut on_point: F,
) -> Result<Vec<(i32, i32)>, WorldGenerationError>
where
    F: FnMut((i32, i32), i32, &mut Rng) -> Result<i32, WorldGenerationError>,
{
    let first = (rng.get_random(width), rng.get_random(height));
    let mut points = vec![first];
    let mut processing = vec![first];
    let mut min_distance = initial_min_distance;
    let mut cell_size = ((f64::from(min_distance) / 2_f64.sqrt()).ceil() as i32).max(1);

    while !processing.is_empty() {
        let point_index = rng.get_random(processing.len() as i32) as usize;
        let next = processing.remove(point_index);
        let mut count = 0;
        while count < new_point_count {
            let radius = rng.get_random(min_distance) + min_distance + 1;
            let angle = f64::from(rng.get_random(360)) / 360.0 * std::f64::consts::TAU;
            let mut x = (f64::from(next.0) + f64::from(radius) * angle.cos()).ceil() as i32;
            let mut y = (f64::from(next.1) + f64::from(radius) * angle.sin()).ceil() as i32;
            if x > width {
                x %= width;
            }
            if x < 0 {
                x += width;
            }
            if y > height {
                y %= height;
            }
            if y < 0 {
                y += height;
            }
            if (x + y) % 2 != 0 {
                continue;
            }
            count += 1;

            let candidate = (x, y);
            let mut valid = true;
            for &existing in &points {
                if cylinder_distance(candidate, existing, width) < min_distance
                    || (candidate.0 / cell_size == existing.0 / cell_size
                        && candidate.1 / cell_size == existing.1 / cell_size)
                {
                    valid = false;
                    break;
                }
            }
            if !valid {
                continue;
            }

            min_distance = on_point(candidate, min_distance, rng)?;
            cell_size = ((f64::from(min_distance) / 2_f64.sqrt()).ceil() as i32).max(1);
            points.push(candidate);
            processing.push(candidate);
        }
    }
    Ok(points)
}

fn cylinder_distance(first: (i32, i32), second: (i32, i32), width: i32) -> i32 {
    let distance = |x: i32| {
        let dx = (first.0 - x).abs();
        let dy = (first.1 - second.1).abs();
        dx + ((dy - dx) / 2).max(0)
    };
    distance(second.0)
        .min(distance(second.0 + width))
        .min(distance(second.0 - width))
}

fn connect_cities(
    game: &mut Game,
    array: &RegionArray,
    cities: &[RegionId],
    tables: &RulesetTables,
    rng: &mut Rng,
) {
    let mut connected = BTreeSet::new();
    for &start in cities {
        if connected.contains(&start) {
            continue;
        }
        for &end in cities {
            if start == end || connected.contains(&end) {
                continue;
            }
            let Some(path) = city_path(game, array, start, end) else {
                continue;
            };
            if path.len() <= 1 || path.len() - 1 > 8 {
                continue;
            }
            connected.insert(start);
            connected.insert(end);
            let town_name = game
                .region(end)
                .and_then(|region| region.town.as_ref())
                .and_then(|town| town.name.as_deref())
                .unwrap_or("Unknown");
            let road_name = format!("Road to {town_name}");

            for edge in path.windows(2).rev() {
                let current = edge[0];
                let next = edge[1];
                let direction = game
                    .region(current)
                    .expect("road path region exists")
                    .neighbors
                    .iter()
                    .position(|neighbor| *neighbor == Some(next));
                let Some(direction) = direction else {
                    continue;
                };
                if rng.get_random(3) == 0 {
                    continue;
                }
                let opposite = (direction + 3) % 6;
                let current_road = ROAD_OBJECTS[direction];
                if !has_object_type(game, current, current_road) {
                    let damage = f64::from(rng.make_roll(2, 6) - 6) / 6.0;
                    add_ancient_structure(
                        game,
                        current,
                        road_name.clone(),
                        current_road,
                        damage,
                        tables,
                    );
                }
                let next_road = ROAD_OBJECTS[opposite];
                if !has_object_type(game, next, next_road) {
                    let damage = f64::from(rng.make_roll(2, 6) - 6) / 6.0;
                    add_ancient_structure(game, next, road_name.clone(), next_road, damage, tables);
                }
            }
        }
    }
}

fn city_path(
    game: &Game,
    array: &RegionArray,
    start: RegionId,
    target: RegionId,
) -> Option<Vec<RegionId>> {
    let mut distances = BTreeMap::from([(start, 0_i32)]);
    let mut previous = BTreeMap::new();
    let mut queue = BinaryHeap::from([Reverse((0_i32, 0_u64, start))]);
    let mut sequence = 1_u64;

    while let Some(Reverse((distance, _, current))) = queue.pop() {
        if distances.get(&current).copied() != Some(distance) {
            continue;
        }
        if current == target {
            break;
        }
        let neighbors = game.region(current)?.neighbors;
        for next in neighbors.into_iter().flatten() {
            if !array_contains(game, array, next) {
                continue;
            }
            let terrain_id = game.region(next)?.terrain_type?;
            if terrain_id == R_OCEAN || terrain_id == R_VOLCANO {
                continue;
            }
            let new_distance = distance + road_terrain_cost(terrain_id);
            if distances
                .get(&next)
                .is_some_and(|known_distance| new_distance >= *known_distance)
            {
                continue;
            }
            distances.insert(next, new_distance);
            previous.insert(next, current);
            queue.push(Reverse((new_distance, sequence, next)));
            sequence = sequence.wrapping_add(1);
        }
    }

    distances.get(&target)?;
    let mut path = vec![target];
    let mut current = target;
    while current != start {
        current = *previous.get(&current)?;
        path.push(current);
    }
    path.reverse();
    Some(path)
}

fn array_contains(game: &Game, array: &RegionArray, region_id: RegionId) -> bool {
    game.region(region_id)
        .is_some_and(|region| array.get_region(region.xloc, region.yloc) == Some(region_id))
}

fn road_terrain_cost(terrain_id: usize) -> i32 {
    match terrain_id {
        R_MOUNTAIN | R_FOREST | R_JUNGLE | R_SWAMP | R_TUNDRA => 2,
        R_PLAIN | R_DESERT => 1,
        _ => 0,
    }
}

fn has_object_type(game: &Game, region_id: RegionId, object_type: usize) -> bool {
    game.region(region_id)
        .expect("road region exists")
        .object_ids()
        .iter()
        .any(|object_id| {
            game.object(*object_id)
                .is_some_and(|object| object.object_type == object_type as i32)
        })
}

fn add_ancient_structure(
    game: &mut Game,
    region_id: RegionId,
    name: String,
    object_type: usize,
    damage: f64,
    tables: &RulesetTables,
) {
    let definition = tables
        .objects
        .get(object_type)
        .expect("historical object has a definition");
    let maximum_incomplete = (definition.cost - 1).max(0);
    let incomplete = ((f64::from(definition.cost) * damage) as i32).clamp(0, maximum_incomplete);
    let region = game
        .region_mut(region_id)
        .expect("historical region exists");
    let number = region.building_seq;
    region.building_seq = region
        .building_seq
        .checked_add(1)
        .expect("region building sequence fits in i32");

    let mut object = Object::default();
    object.name = Some(format!("{name} [{number}]"));
    object.num = number;
    object.object_type = object_type as i32;
    object.incomplete = incomplete;
    object.inner = -1;
    game.add_object(region_id, object)
        .expect("historical region exists");
}

#[cfg(test)]
mod tests {
    use crate::{
        defs::terrains::R_PLAIN,
        model::{Game, RegionLevelType, TownInfo},
        rng::Rng,
        ruleset::Ruleset,
    };

    use super::{connect_cities, ROAD_OBJECTS};

    #[test]
    fn connected_cities_receive_matching_directional_roads() {
        let ruleset = <dyn Ruleset>::from_name("neworigins").expect("NewOrigins ruleset");
        let tables = ruleset.tables();
        let mut game = Game::default();
        let array_index = super::super::super::classic::create_region_array(
            &mut game,
            RegionLevelType::Surface,
            "surface",
            8,
            8,
        )
        .expect("surface array is created");
        let array = game.region_arrays[array_index].clone();
        for region_id in game.region_ids().to_vec() {
            game.region_mut(region_id)
                .expect("surface region exists")
                .terrain_type = Some(R_PLAIN);
        }

        let start = array.get_region(0, 0).expect("start region exists");
        let end = game
            .region(start)
            .expect("start region exists")
            .neighbors
            .iter()
            .flatten()
            .next()
            .copied()
            .expect("start region has a neighbor");
        for (region_id, name) in [(start, "West City"), (end, "East City")] {
            game.region_mut(region_id).expect("city region exists").town = Some(TownInfo {
                name: Some(name.to_owned()),
                population: 10_000,
                ..TownInfo::default()
            });
        }
        let direction = game
            .region(start)
            .expect("start region exists")
            .neighbors
            .iter()
            .position(|neighbor| *neighbor == Some(end))
            .expect("cities share an edge");
        let seed = (1..100)
            .find(|seed| {
                let mut rng = Rng::new(*seed);
                rng.get_random(3) != 0
            })
            .expect("a seed places this road");
        let mut rng = Rng::new(seed);

        connect_cities(&mut game, &array, &[start, end], &tables, &mut rng);

        for (region_id, object_type) in [
            (start, ROAD_OBJECTS[direction]),
            (end, ROAD_OBJECTS[(direction + 3) % ROAD_OBJECTS.len()]),
        ] {
            let road = game
                .region(region_id)
                .expect("city region exists")
                .object_ids()
                .iter()
                .find_map(|object_id| {
                    game.object(*object_id)
                        .filter(|object| object.object_type == object_type as i32)
                })
                .expect("each end of the city road has a matching road object");
            assert_eq!(road.name.as_deref(), Some("Road to East City [1]"));
        }
    }
}
