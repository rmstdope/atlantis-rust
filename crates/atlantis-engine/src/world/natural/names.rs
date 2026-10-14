use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::{
    defs::{
        terrains::{R_MOUNTAIN, R_VOLCANO},
        RulesetTables,
    },
    model::{Game, RegionArray, RegionId},
    rng::Rng,
    ruleset::WorldNameStyle,
};

use super::rivers::RiverGeneration;

pub(super) struct NaturalNameGenerator {
    generator: super::super::names::NameGenerator,
    seed_span: i32,
}

impl NaturalNameGenerator {
    pub(super) fn new(seed_span: i32) -> Self {
        Self {
            generator: super::super::names::NameGenerator::new(WorldNameStyle::NewOrigins),
            seed_span,
        }
    }

    /// C++: `namegen.cpp` `giveNames`.
    pub(super) fn name_regions(
        &mut self,
        game: &mut Game,
        array: &RegionArray,
        rivers: &RiverGeneration,
        tables: &RulesetTables,
        rng: &mut Rng,
    ) {
        let mut named = BTreeSet::new();
        let mut river_groups = BTreeMap::<usize, Vec<RegionId>>::new();
        for (region_id, river_name) in &rivers.river_regions {
            river_groups
                .entry(*river_name)
                .or_default()
                .push(*region_id);
        }
        for regions in river_groups.values() {
            let terrain = terrain(game, regions[0]);
            let name = self.next_region_name(terrain, &tables.terrains, rng);
            assign_name(game, regions, &name);
            named.extend(regions.iter().copied());
        }

        for water_body in &rivers.water_bodies {
            let Some(&first) = water_body.regions.first() else {
                continue;
            };
            let terrain = terrain(game, first);
            let name = self.next_region_name(terrain, &tables.terrains, rng);
            assign_name(game, &water_body.regions, &name);
            named.extend(water_body.regions.iter().copied());
        }

        for x in 0..array.x {
            for y in 0..array.y {
                let Some(start) = array.get_region(x, y) else {
                    continue;
                };
                if named.contains(&start) {
                    continue;
                }
                let start_terrain = terrain(game, start);
                let mut component = Vec::new();
                let mut visited = BTreeSet::from([start]);
                let mut pending = VecDeque::from([start]);
                while let Some(region_id) = pending.pop_front() {
                    component.push(region_id);
                    for neighbor_id in game
                        .region(region_id)
                        .expect("surface region exists")
                        .neighbors
                        .iter()
                        .flatten()
                        .copied()
                    {
                        if named.contains(&neighbor_id)
                            || !same_name_area(start_terrain, terrain(game, neighbor_id))
                            || !visited.insert(neighbor_id)
                        {
                            continue;
                        }
                        pending.push_back(neighbor_id);
                    }
                }
                let name = self.next_region_name(start_terrain, &tables.terrains, rng);
                assign_name(game, &component, &name);
                named.extend(component);
            }
        }
    }

    pub(super) fn next_town_name(
        &mut self,
        terrain_id: usize,
        port: bool,
        tables: &RulesetTables,
        rng: &mut Rng,
    ) -> String {
        let seed = rng.make_roll(1, self.seed_span);
        let mut name_rng = Rng::new(seed);
        self.generator
            .next_town_name(terrain_id, &tables.terrains, port, &mut name_rng)
    }

    pub(super) fn next_object_name(
        &mut self,
        terrain_id: usize,
        object_type: usize,
        tables: &RulesetTables,
        rng: &mut Rng,
    ) -> String {
        let region_name = self.next_region_name(terrain_id, &tables.terrains, rng);
        let object_name = tables
            .objects
            .get(object_type)
            .expect("historical object has a definition")
            .name;
        format!("{region_name} {object_name}")
    }

    fn next_region_name(
        &mut self,
        terrain_id: usize,
        terrain_defs: &[crate::defs::TerrainDef],
        rng: &mut Rng,
    ) -> String {
        let seed = rng.get_random(self.seed_span) + 1;
        let mut name_rng = Rng::new(seed);
        let index = self
            .generator
            .next_region_name(terrain_id, terrain_defs, &mut name_rng);
        self.generator
            .resolve(index)
            .expect("generated NewOrigins name exists")
            .to_owned()
    }
}

fn assign_name(game: &mut Game, regions: &[RegionId], name: &str) {
    for &region_id in regions {
        game.region_mut(region_id)
            .expect("surface region exists")
            .name = name.to_owned();
    }
}

fn terrain(game: &Game, region_id: RegionId) -> usize {
    game.region(region_id)
        .and_then(|region| region.terrain_type)
        .expect("natural surface region has a terrain")
}

fn same_name_area(first: usize, next: usize) -> bool {
    if first == R_MOUNTAIN || first == R_VOLCANO {
        next == R_MOUNTAIN || next == R_VOLCANO
    } else {
        first == next
    }
}
