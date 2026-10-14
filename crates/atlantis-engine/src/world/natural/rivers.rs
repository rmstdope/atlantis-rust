use std::{
    cmp::Reverse,
    collections::{BTreeMap, BTreeSet, BinaryHeap, VecDeque},
};

use crate::{
    defs::terrains::{R_MOUNTAIN, R_OCEAN, R_SWAMP, R_VOLCANO},
    model::{Game, RegionArray, RegionId},
    rng::Rng,
};

use super::map::NaturalMap;

#[derive(Clone, Debug)]
pub(super) struct WaterBody {
    pub(super) name: usize,
    pub(super) regions: Vec<RegionId>,
    members: BTreeSet<RegionId>,
    connections: BTreeSet<usize>,
}

impl WaterBody {
    fn new(name: usize) -> Self {
        Self {
            name,
            regions: Vec::new(),
            members: BTreeSet::new(),
            connections: BTreeSet::new(),
        }
    }

    fn includes(&self, region_id: RegionId) -> bool {
        self.members.contains(&region_id)
    }

    fn add(&mut self, region_id: RegionId) {
        if self.members.insert(region_id) {
            self.regions.push(region_id);
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct RiverGeneration {
    pub(super) water_bodies: Vec<WaterBody>,
    pub(super) river_regions: BTreeMap<RegionId, usize>,
}

type PathDistances = BTreeMap<RegionId, (i64, Option<RegionId>)>;
type PathPredecessors = BTreeMap<RegionId, RegionId>;

pub(super) fn discover_water_bodies(game: &Game, array: &RegionArray) -> Vec<WaterBody> {
    let mut water_bodies = Vec::new();
    let mut visited = BTreeSet::new();
    for x in 0..array.x {
        for y in 0..array.y {
            let Some(start) = array.get_region(x, y) else {
                continue;
            };
            if !is_ocean(game, start) || visited.contains(&start) {
                continue;
            }
            let name = water_bodies.len();
            let mut body = WaterBody::new(name);
            let mut pending = VecDeque::from([start]);
            visited.insert(start);
            while let Some(region_id) = pending.pop_front() {
                body.add(region_id);
                for neighbor_id in game
                    .region(region_id)
                    .expect("surface region exists")
                    .neighbors
                    .iter()
                    .flatten()
                    .copied()
                {
                    if is_ocean(game, neighbor_id) && visited.insert(neighbor_id) {
                        pending.push_back(neighbor_id);
                    }
                }
            }
            water_bodies.push(body);
        }
    }
    water_bodies
}

/// C++: `aregion.cpp` `makeRivers`; candidate shuffling uses its separate seed stream.
pub(super) fn generate_rivers(
    game: &mut Game,
    array: &RegionArray,
    map: &NaturalMap,
    max_river_reach: i32,
    rng: &mut Rng,
    shuffle_seed: i32,
) -> RiverGeneration {
    let mut water_bodies = discover_water_bodies(game, array);
    let inner_water = find_inner_water(game, array);
    let distances = water_body_distances(game, &water_bodies, &inner_water);
    let mut river_regions = BTreeMap::new();
    let mut shuffle_rng = Rng::new(shuffle_seed ^ 0x5249_5645);
    let mut river_name = 0;

    for source_index in 0..water_bodies.len() {
        let candidates: Vec<_> = (0..water_bodies.len())
            .filter(|target_index| *target_index != source_index)
            .filter_map(|target_index| {
                let distance = distances[source_index][target_index];
                (distance <= max_river_reach).then_some((target_index, distance))
            })
            .collect();
        let connection_count = rng.make_roll(3, 4).min(candidates.len() as i32) as usize;
        let mut candidates = candidates;
        shuffle(&mut candidates, &mut shuffle_rng);

        for (target_index, _) in candidates.into_iter().take(connection_count) {
            if water_bodies[source_index]
                .connections
                .contains(&water_bodies[target_index].name)
            {
                continue;
            }
            let Some((start, end, cost)) = best_river_endpoints(
                game,
                map,
                &water_bodies[source_index],
                &water_bodies[target_index],
                &inner_water,
                &river_regions,
            ) else {
                continue;
            };

            let blocked = blocked_river_regions(
                game,
                &water_bodies[source_index],
                &water_bodies[target_index],
                &inner_water,
                &river_regions,
            );
            let goals = BTreeSet::from([end]);
            let Some(full_path) = shortest_path(game, array, start, &goals, &blocked, |_, next| {
                river_step_cost(game, map, next, &river_regions)
            }) else {
                continue;
            };
            if full_path.len() < 2 || cost <= 0 {
                continue;
            }

            let source_name = water_bodies[source_index].name;
            let target_name = water_bodies[target_index].name;
            water_bodies[source_index].connections.insert(target_name);
            water_bodies[target_index].connections.insert(source_name);
            let path = &full_path[1..];
            let segment_length = 4.min(path.len() as i32 - 1);
            let mut first = true;
            let mut counter = 0;
            for &region_id in path {
                if river_regions.contains_key(&region_id) {
                    river_name += 1;
                    counter = 1;
                    first = false;
                    continue;
                }
                river_regions.insert(region_id, river_name);
                let terrain = if first {
                    if path.len() == 1 {
                        R_SWAMP
                    } else {
                        R_OCEAN
                    }
                } else if counter % segment_length as usize == 0 {
                    R_SWAMP
                } else {
                    R_OCEAN
                };
                game.region_mut(region_id)
                    .expect("river path region exists")
                    .terrain_type = Some(terrain);
                first = false;
                counter += 1;
            }
            river_name += 1;
        }
    }

    RiverGeneration {
        water_bodies,
        river_regions,
    }
}

/// C++: `aregion.cpp` `cleanupIsolatedPlaces`.
pub(super) fn cleanup_isolated_places(
    game: &mut Game,
    array: &RegionArray,
    water_bodies: &mut [WaterBody],
    river_regions: &mut BTreeMap<RegionId, usize>,
    rng: &mut Rng,
) {
    for x in 0..array.x {
        for y in 0..array.y {
            let Some(region_id) = array.get_region(x, y) else {
                continue;
            };
            if is_ocean(game, region_id) {
                continue;
            }
            let neighbors = game
                .region(region_id)
                .expect("surface region exists")
                .neighbors;
            let mut body_neighbors = Vec::new();
            let mut river_neighbors = Vec::new();
            let mut clear = true;
            for neighbor_id in neighbors.into_iter().flatten() {
                if !is_ocean(game, neighbor_id) {
                    clear = false;
                    break;
                }
                if let Some(body) = water_bodies
                    .iter()
                    .position(|body| body.includes(neighbor_id))
                {
                    body_neighbors.push(body);
                }
                if let Some(name) = river_regions.get(&neighbor_id) {
                    river_neighbors.push(*name);
                }
            }
            if !clear {
                continue;
            }

            game.region_mut(region_id)
                .expect("surface region exists")
                .terrain_type = Some(R_OCEAN);
            if !body_neighbors.is_empty() {
                let index = rng.get_random(body_neighbors.len() as i32) as usize;
                water_bodies[body_neighbors[index]].add(region_id);
            } else if !river_neighbors.is_empty() {
                let index = rng.get_random(river_neighbors.len() as i32) as usize;
                river_regions.insert(region_id, river_neighbors[index]);
            }
        }
    }
}

/// C++: `aregion.cpp` `placeVolcanoes`.
pub(super) fn place_volcanoes(game: &mut Game, array: &RegionArray, rng: &mut Rng) {
    for x in 0..array.x {
        for y in 0..array.y {
            let Some(region_id) = array.get_region(x, y) else {
                continue;
            };
            if terrain(game, region_id) != Some(R_MOUNTAIN) {
                continue;
            }
            let mountain_distance = rng.make_roll(1, 3) + 1;
            let mountains = count_neighbors(game, region_id, R_MOUNTAIN, mountain_distance);
            let volcanoes = count_neighbors(game, region_id, R_VOLCANO, 2);
            if volcanoes == 0 && mountains >= rng.make_roll(1, 6) + 2 {
                game.region_mut(region_id)
                    .expect("surface region exists")
                    .terrain_type = Some(R_VOLCANO);
            }
        }
    }
}

fn find_inner_water(game: &Game, array: &RegionArray) -> BTreeSet<RegionId> {
    let mut inner_water = BTreeSet::new();
    for x in 0..array.x {
        for y in 0..array.y {
            let Some(region_id) = array.get_region(x, y) else {
                continue;
            };
            if terrain(game, region_id) == Some(R_OCEAN)
                && game
                    .region(region_id)
                    .expect("surface region exists")
                    .neighbors
                    .iter()
                    .flatten()
                    .all(|neighbor_id| terrain(game, *neighbor_id) == Some(R_OCEAN))
            {
                inner_water.insert(region_id);
            }
        }
    }
    inner_water
}

fn water_body_distances(
    game: &Game,
    water_bodies: &[WaterBody],
    inner_water: &BTreeSet<RegionId>,
) -> Vec<Vec<i32>> {
    let mut body_by_region = BTreeMap::new();
    for (index, body) in water_bodies.iter().enumerate() {
        for region_id in &body.regions {
            body_by_region.insert(*region_id, index);
        }
    }
    let mut distances = vec![vec![i32::MAX; water_bodies.len()]; water_bodies.len()];
    for (source_index, source) in water_bodies.iter().enumerate() {
        let mut blocked = inner_water.clone();
        blocked.extend(source.regions.iter().copied());
        for &start in &source.regions {
            if inner_water.contains(&start) {
                continue;
            }
            let (found, _) = dijkstra_all(game, start, &blocked, |_, _| 1);
            for (region_id, (distance, _)) in found {
                let Some(&target_index) = body_by_region.get(&region_id) else {
                    continue;
                };
                if target_index == source_index {
                    continue;
                }
                let new_distance = (distance + 1) as i32;
                distances[source_index][target_index] =
                    distances[source_index][target_index].min(new_distance);
            }
        }
    }
    distances
}

fn best_river_endpoints(
    game: &Game,
    map: &NaturalMap,
    source: &WaterBody,
    target: &WaterBody,
    inner_water: &BTreeSet<RegionId>,
    river_regions: &BTreeMap<RegionId, usize>,
) -> Option<(RegionId, RegionId, i64)> {
    let blocked = blocked_river_regions(game, source, target, inner_water, river_regions);
    let mut best = None;
    for &start in &source.regions {
        if inner_water.contains(&start) {
            continue;
        }
        let (distances, _) = dijkstra_all(game, start, &blocked, |_, next| {
            river_step_cost(game, map, next, river_regions)
        });
        for &end in &target.regions {
            let Some(&(cost, _)) = distances.get(&end) else {
                continue;
            };
            if cost > 0 && best.is_none_or(|(_, _, best_cost)| cost < best_cost) {
                best = Some((start, end, cost));
            }
        }
    }
    best
}

fn blocked_river_regions(
    game: &Game,
    source: &WaterBody,
    target: &WaterBody,
    inner_water: &BTreeSet<RegionId>,
    river_regions: &BTreeMap<RegionId, usize>,
) -> BTreeSet<RegionId> {
    let mut blocked = inner_water.clone();
    blocked.extend(source.regions.iter().copied());
    for region_id in game.region_ids() {
        if terrain(game, *region_id) == Some(R_OCEAN)
            && !target.includes(*region_id)
            && !river_regions.contains_key(region_id)
        {
            blocked.insert(*region_id);
        }
    }
    blocked
}

fn river_step_cost(
    game: &Game,
    map: &NaturalMap,
    next: RegionId,
    river_regions: &BTreeMap<RegionId, usize>,
) -> i64 {
    let region = game.region(next).expect("path region exists");
    let elevation = map
        .get(region.xloc * 2, region.yloc * 2)
        .expect("region center is inside climate map")
        .elevation
        .max(1);
    let mut cost = i64::from(elevation);
    if river_regions.contains_key(&next) {
        cost /= 10;
    } else if game
        .region(next)
        .expect("path region exists")
        .neighbors
        .iter()
        .flatten()
        .any(|neighbor_id| terrain(game, *neighbor_id) == Some(R_OCEAN))
    {
        cost *= 10;
    }
    cost
}

fn shortest_path<F>(
    game: &Game,
    _array: &RegionArray,
    start: RegionId,
    goals: &BTreeSet<RegionId>,
    blocked: &BTreeSet<RegionId>,
    step_cost: F,
) -> Option<Vec<RegionId>>
where
    F: FnMut(RegionId, RegionId) -> i64,
{
    let (distances, previous) = dijkstra_all(game, start, blocked, step_cost);
    let goal = goals
        .iter()
        .filter_map(|region_id| {
            distances
                .get(region_id)
                .map(|(distance, _)| (*distance, *region_id))
        })
        .min()
        .map(|(_, region_id)| region_id)?;
    Some(rebuild_path(&previous, start, goal))
}

fn dijkstra_all<F>(
    game: &Game,
    start: RegionId,
    blocked: &BTreeSet<RegionId>,
    mut step_cost: F,
) -> (PathDistances, PathPredecessors)
where
    F: FnMut(RegionId, RegionId) -> i64,
{
    let mut distances = BTreeMap::from([(start, (0_i64, None))]);
    let mut previous = BTreeMap::new();
    let mut queue = BinaryHeap::from([Reverse((0_i64, 0_u64, start))]);
    let mut sequence = 1_u64;
    while let Some(Reverse((distance, _, current))) = queue.pop() {
        if distances
            .get(&current)
            .is_none_or(|(known, _)| distance != *known)
        {
            continue;
        }
        let neighbors = game.region(current).expect("path region exists").neighbors;
        for next in neighbors.into_iter().flatten() {
            if blocked.contains(&next) {
                continue;
            }
            let new_distance = distance + step_cost(current, next).max(0);
            if distances
                .get(&next)
                .is_some_and(|(known, _)| new_distance >= *known)
            {
                continue;
            }
            distances.insert(next, (new_distance, Some(current)));
            previous.insert(next, current);
            queue.push(Reverse((new_distance, sequence, next)));
            sequence = sequence.wrapping_add(1);
        }
    }
    (distances, previous)
}

fn rebuild_path(
    previous: &BTreeMap<RegionId, RegionId>,
    start: RegionId,
    mut goal: RegionId,
) -> Vec<RegionId> {
    let mut path = vec![goal];
    while goal != start {
        goal = previous[&goal];
        path.push(goal);
    }
    path.reverse();
    path
}

fn count_neighbors(game: &Game, start: RegionId, terrain_id: usize, max_distance: i32) -> i32 {
    let mut pending = VecDeque::from([(start, 0)]);
    let mut visited = BTreeSet::from([start]);
    let mut count = 0;
    while let Some((region_id, distance)) = pending.pop_front() {
        if distance >= max_distance {
            continue;
        }
        if terrain(game, region_id) == Some(terrain_id) {
            count += 1;
        }
        for neighbor_id in game
            .region(region_id)
            .expect("surface region exists")
            .neighbors
            .iter()
            .flatten()
            .copied()
        {
            if !visited.insert(neighbor_id) {
                continue;
            }
            pending.push_back((neighbor_id, distance + 1));
        }
    }
    count
}

fn is_ocean(game: &Game, region_id: RegionId) -> bool {
    terrain(game, region_id) == Some(R_OCEAN)
}

fn terrain(game: &Game, region_id: RegionId) -> Option<usize> {
    game.region(region_id)
        .and_then(|region| region.terrain_type)
}

fn shuffle<T>(values: &mut [T], rng: &mut Rng) {
    for index in (1..values.len()).rev() {
        let other = rng.get_random((index + 1) as i32) as usize;
        values.swap(index, other);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::{
        defs::terrains::{R_MOUNTAIN, R_OCEAN, R_PLAIN, R_VOLCANO},
        model::{Game, RegionLevelType},
        rng::Rng,
    };

    fn surface() -> (Game, crate::model::RegionArray) {
        let mut game = Game::default();
        let index = crate::world::classic::create_region_array(
            &mut game,
            RegionLevelType::Surface,
            "surface",
            8,
            8,
        )
        .expect("surface grid succeeds");
        let array = game.region_arrays[index].clone();
        (game, array)
    }

    #[test]
    fn rivers_connect_water_bodies_without_crossing_unrelated_ocean() {
        let (mut game, array) = surface();
        for region_id in game.region_ids().to_vec() {
            game.region_mut(region_id).unwrap().terrain_type = Some(R_PLAIN);
        }
        let water_cells = [(0, 0), (4, 0), (0, 4)];
        let water_ids: Vec<_> = water_cells
            .into_iter()
            .map(|(x, y)| array.get_region(x, y).expect("water cell exists"))
            .collect();
        for region_id in &water_ids {
            game.region_mut(*region_id).unwrap().terrain_type = Some(R_OCEAN);
        }
        let water_bodies = super::discover_water_bodies(&game, &array);
        assert_eq!(water_bodies.len(), 3);
        assert!(water_bodies.iter().all(|body| body.regions.len() == 1));

        let start = array.get_region(0, 0).expect("source water exists");
        let goal = array.get_region(4, 0).expect("target water exists");
        let unrelated = array.get_region(0, 4).expect("unrelated water exists");
        let path = super::shortest_path(
            &game,
            &array,
            start,
            &BTreeSet::from([goal]),
            &BTreeSet::from([unrelated]),
            |_, _| 1,
        )
        .expect("a land path connects the water bodies");

        assert_eq!(path.first(), Some(&start));
        assert_eq!(path.last(), Some(&goal));
        assert!(!path.contains(&unrelated));
    }

    #[test]
    fn volcanoes_only_replace_dense_mountain_regions() {
        let (mut game, array) = surface();
        for region_id in game.region_ids().to_vec() {
            game.region_mut(region_id).unwrap().terrain_type = Some(R_MOUNTAIN);
        }
        let mut rng = Rng::new(5);

        super::place_volcanoes(&mut game, &array, &mut rng);

        let volcanoes: Vec<_> = game
            .region_ids()
            .iter()
            .copied()
            .filter(|id| game.region(*id).unwrap().terrain_type == Some(R_VOLCANO))
            .collect();
        assert!(
            !volcanoes.is_empty(),
            "dense mountain fields should form volcanoes"
        );
        for volcano in volcanoes {
            for neighbor in game.region(volcano).unwrap().neighbors.iter().flatten() {
                assert_ne!(
                    game.region(*neighbor).unwrap().terrain_type,
                    Some(R_VOLCANO),
                    "volcanoes must not be adjacent"
                );
            }
        }
    }
}
