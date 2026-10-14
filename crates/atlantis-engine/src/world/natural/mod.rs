mod map;
mod names;
mod rivers;
mod settlements;

use std::collections::BTreeSet;

use super::{classic::create_region_array, WorldGenerationError};
use crate::{
    model::{Game, RegionLevelType},
    rng::Rng,
    ruleset::{Ruleset, WorldNameStyle},
};

#[derive(Debug)]
struct NaturalWorldStatistics {
    biome_counts: [usize; 8],
    river_count: usize,
    river_region_count: usize,
    settlement_count: usize,
}

/// C++: `aregion.cpp` `ARegionList::CreateNaturalSurfaceLevel`.
pub fn create_natural_surface_level(
    game: &mut Game,
    ruleset: &dyn Ruleset,
    rng: &mut Rng,
    natural_seed: i32,
    width: i32,
    height: i32,
) -> Result<usize, WorldGenerationError> {
    let (level, statistics) = create_natural_surface_level_with_statistics(
        game,
        ruleset,
        rng,
        natural_seed,
        width,
        height,
    )?;
    debug_assert!(statistics.biome_counts.iter().sum::<usize>() > 0);
    debug_assert!(statistics.river_count <= statistics.river_region_count);
    debug_assert!(statistics.settlement_count <= statistics.biome_counts.iter().sum());
    Ok(level)
}

fn create_natural_surface_level_with_statistics(
    game: &mut Game,
    ruleset: &dyn Ruleset,
    rng: &mut Rng,
    natural_seed: i32,
    width: i32,
    height: i32,
) -> Result<(usize, NaturalWorldStatistics), WorldGenerationError> {
    if ruleset.world_name_style() != WorldNameStyle::NewOrigins {
        return Err(WorldGenerationError::UnsupportedNaturalRuleset);
    }
    if width < 8 || height < 8 || width % 8 != 0 || height % 8 != 0 {
        return Err(WorldGenerationError::InvalidNaturalSurfaceDimensions { width, height });
    }
    let map_width = width
        .checked_mul(2)
        .ok_or(WorldGenerationError::NaturalMapDimensionsOverflow { width, height })?;
    let map_height = height
        .checked_mul(2)
        .ok_or(WorldGenerationError::NaturalMapDimensionsOverflow { width, height })?;
    let seed_span = width
        .checked_mul(height)
        .ok_or(WorldGenerationError::NaturalMapDimensionsOverflow { width, height })?;

    let level = create_region_array(game, RegionLevelType::Surface, "surface", width, height)?;
    let array = game.region_arrays[level].clone();
    let natural_map = map::NaturalMap::generate(map_width, map_height, natural_seed)?;
    let mut biome_counts = [0; 8];
    for y in 0..height {
        for x in 0..width {
            let Some(region_id) = array.get_region(x, y) else {
                continue;
            };
            let biome = natural_map
                .get(x * 2, y * 2)
                .expect("surface region center is inside the climate map")
                .biome;
            let statistics_index = biome
                .statistics_index()
                .ok_or(WorldGenerationError::MissingNaturalBiome { x, y })?;
            let terrain = biome
                .terrain_id()
                .ok_or(WorldGenerationError::MissingNaturalBiome { x, y })?;
            biome_counts[statistics_index] += 1;
            game.region_mut(region_id)
                .expect("generated surface region exists")
                .terrain_type = Some(terrain);
        }
    }
    let mut river_generation = rivers::generate_rivers(
        game,
        &array,
        &natural_map,
        width.min(height) / 4,
        rng,
        natural_seed,
    );
    let mut river_names = BTreeSet::new();
    river_names.extend(river_generation.river_regions.values().copied());
    let river_count = river_names.len();
    let river_region_count = river_generation.river_regions.len();
    rivers::cleanup_isolated_places(
        game,
        &array,
        &mut river_generation.water_bodies,
        &mut river_generation.river_regions,
        rng,
    );
    rivers::place_volcanoes(game, &array, rng);
    let tables = ruleset.tables();
    let definitions = ruleset.game_defs();
    super::classic::grow_races(
        game,
        &array,
        definitions,
        &tables.terrains,
        &tables.item_families,
        rng,
    );
    let mut names = names::NaturalNameGenerator::new(seed_span);
    names.name_regions(game, &array, &river_generation, &tables, rng);
    settlements::generate(game, &array, definitions, &tables, &mut names, rng)?;
    let mut settlement_count = 0;
    for x in 0..array.x {
        for y in 0..array.y {
            let Some(region_id) = array.get_region(x, y) else {
                continue;
            };
            if game
                .region(region_id)
                .is_some_and(|region| region.town.is_some())
            {
                settlement_count += 1;
            }
        }
    }
    settlements::add_historical_buildings(game, &array, definitions, &tables, &mut names, rng);
    Ok((
        level,
        NaturalWorldStatistics {
            biome_counts,
            river_count,
            river_region_count,
            settlement_count,
        },
    ))
}

#[cfg(test)]
mod tests {
    use crate::{
        defs::{
            objects::{O_CASTLE, O_FORT, O_INN, O_TOWER},
            terrains::R_PLAIN,
        },
        model::{Game, Region, RegionArray, RegionLevelType, TownInfo},
        rng::Rng,
        ruleset::Ruleset,
    };

    #[test]
    fn natural_surface_uses_the_existing_hex_topology() {
        let ruleset = <dyn Ruleset>::from_name("neworigins").expect("NewOrigins ruleset");
        let mut game = Game::default();
        let mut rng = Rng::new(5);

        let level = super::create_natural_surface_level(&mut game, ruleset, &mut rng, 5, 8, 8)
            .expect("natural surface generation succeeds");
        let array = &game.region_arrays[level];

        assert_eq!(array.level_type, RegionLevelType::Surface);
        assert_eq!(array.x, 8);
        assert_eq!(array.y, 8);
        assert_eq!(game.region_ids().len(), 32);
        for y in 0..array.y {
            for x in 0..array.x {
                let Some(region_id) = array.get_region(x, y) else {
                    continue;
                };
                let region = game.region(region_id).expect("surface region exists");
                for neighbor_id in region.neighbors.iter().flatten() {
                    let neighbor = game.region(*neighbor_id).expect("neighbor exists");
                    assert!(
                        neighbor.neighbors.contains(&Some(region_id)),
                        "neighbor link from {region_id:?} is not reciprocal"
                    );
                }
            }
        }
    }

    #[test]
    fn natural_surface_names_every_region_and_sets_up_settlements() {
        let ruleset = <dyn Ruleset>::from_name("neworigins").expect("NewOrigins ruleset");
        let mut game = Game::default();
        let mut rng = Rng::new(7);

        super::create_natural_surface_level(&mut game, ruleset, &mut rng, 7, 16, 16)
            .expect("natural surface generation succeeds");

        let mut settlements = 0;
        for region_id in game.region_ids() {
            let region = game.region(*region_id).expect("generated region exists");
            assert!(!region.name.is_empty(), "region {region_id:?} has a name");
            assert_ne!(region.name, "Region", "region {region_id:?} was named");
            assert!(
                !region.object_ids().is_empty(),
                "region {region_id:?} received ManualSetup's dummy object"
            );
            if let Some(town) = &region.town {
                settlements += 1;
                assert!(
                    town.name.as_deref().is_some_and(|name| !name.is_empty()),
                    "settlement in region {region_id:?} has a name"
                );
            }
        }
        assert!(settlements > 0, "natural generation creates settlements");
    }

    #[test]
    fn historical_buildings_follow_settlement_state() {
        let ruleset = <dyn Ruleset>::from_name("neworigins").expect("NewOrigins ruleset");
        let tables = ruleset.tables();
        let mut game = Game::default();
        let mut region = Region::default();
        region.terrain_type = Some(R_PLAIN);
        region.population = 9_000;
        region.town = Some(TownInfo {
            name: Some("Reference City".to_owned()),
            population: 9_000,
            activity: 0,
            habitat: 12_000,
            development: 100,
        });
        let city = game.add_region(region);
        let mut array =
            RegionArray::new(2, 2, RegionLevelType::Surface).expect("valid region array");
        array
            .set_region(0, 0, city)
            .expect("city is in the region array");
        let mut names = super::names::NaturalNameGenerator::new(64);
        let mut rng = Rng::new(1);

        super::settlements::add_historical_buildings(
            &mut game,
            &array,
            ruleset.game_defs(),
            &tables,
            &mut names,
            &mut rng,
        );

        let region = game.region(city).expect("city exists");
        assert!(
            region.object_ids().iter().any(|object_id| {
                game.object(*object_id).is_some_and(|object| {
                    [O_CASTLE, O_FORT, O_INN, O_TOWER].contains(&(object.object_type as usize))
                })
            }),
            "a city receives an eligible historical structure"
        );
        assert!(region.building_seq > 1, "historical buildings are numbered");
    }

    #[derive(serde::Deserialize)]
    struct ReferenceFixture {
        format: String,
        width: i32,
        height: i32,
        biomes: Vec<String>,
        worlds: Vec<ReferenceWorld>,
    }

    #[derive(serde::Deserialize)]
    struct ReferenceWorld {
        seed: i32,
        biome_counts: [usize; 8],
        river_count: usize,
        river_region_count: usize,
        settlement_count: usize,
    }

    fn assert_relative_match(name: &str, actual: u64, reference: u64, tolerance: f64) {
        let relative_error = if reference == 0 {
            if actual == 0 {
                0.0
            } else {
                f64::INFINITY
            }
        } else {
            (actual as f64 - reference as f64).abs() / reference as f64
        };
        assert!(
            relative_error <= tolerance,
            "{name} differs by {:.1}% (actual total {actual}, reference total {reference}, tolerance {:.1}%)",
            relative_error * 100.0,
            tolerance * 100.0
        );
    }

    #[test]
    fn natural_statistics_match_cpp_reference_over_50_seeds() {
        const REFERENCE: &str =
            include_str!("../../../../../fixtures/world-generation/neworigins-natural-stats.json");
        let reference: ReferenceFixture =
            serde_json::from_str(REFERENCE).expect("valid C++ natural-world statistics fixture");

        assert_eq!(
            reference.format,
            "atlantis-neworigins-natural-world-stats-v1"
        );
        assert_eq!((reference.width, reference.height), (64, 64));
        assert_eq!(
            reference.biomes,
            [
                "tundra",
                "mountains",
                "swamp",
                "forest",
                "plains",
                "jungle",
                "desert",
                "water"
            ]
            .map(str::to_owned)
        );
        assert_eq!(reference.worlds.len(), 50);

        let ruleset = <dyn Ruleset>::from_name("neworigins").expect("NewOrigins ruleset");
        let mut reference_biomes = [0_u64; 8];
        let mut actual_biomes = [0_u64; 8];
        let mut reference_rivers = 0_u64;
        let mut actual_rivers = 0_u64;
        let mut reference_river_regions = 0_u64;
        let mut actual_river_regions = 0_u64;
        let mut reference_settlements = 0_u64;
        let mut actual_settlements = 0_u64;

        for (index, expected) in reference.worlds.iter().enumerate() {
            assert_eq!(expected.seed, index as i32 + 1, "reference seed order");
            assert_eq!(
                expected.biome_counts.iter().sum::<usize>(),
                2_048,
                "reference biome count for seed {}",
                expected.seed
            );
            for (total, count) in reference_biomes.iter_mut().zip(expected.biome_counts) {
                *total += count as u64;
            }
            reference_rivers += expected.river_count as u64;
            reference_river_regions += expected.river_region_count as u64;
            reference_settlements += expected.settlement_count as u64;

            let mut game = Game::default();
            let mut rng = Rng::new(expected.seed);
            let (_, actual) = super::create_natural_surface_level_with_statistics(
                &mut game,
                ruleset,
                &mut rng,
                expected.seed,
                reference.width,
                reference.height,
            )
            .expect("natural surface generation succeeds");

            for (total, count) in actual_biomes.iter_mut().zip(actual.biome_counts) {
                *total += count as u64;
            }
            actual_rivers += actual.river_count as u64;
            actual_river_regions += actual.river_region_count as u64;
            actual_settlements += actual.settlement_count as u64;
        }

        let total_regions = (reference.worlds.len() * 2_048) as f64;
        for index in 0..reference.biomes.len() {
            let actual_share = actual_biomes[index] as f64 / total_regions;
            let reference_share = reference_biomes[index] as f64 / total_regions;
            let difference = (actual_share - reference_share).abs();
            assert!(
                difference <= 0.02,
                "{} share differs by {:.2} percentage points (actual {:.2}%, reference {:.2}%)",
                reference.biomes[index],
                difference * 100.0,
                actual_share * 100.0,
                reference_share * 100.0
            );
        }
        assert_relative_match("river count", actual_rivers, reference_rivers, 0.25);
        assert_relative_match(
            "river-region count",
            actual_river_regions,
            reference_river_regions,
            0.25,
        );
        assert_relative_match(
            "settlement count",
            actual_settlements,
            reference_settlements,
            0.15,
        );
    }
}
