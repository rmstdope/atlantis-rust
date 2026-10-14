use std::fs;
use std::path::Path;

use atlantis_engine::defs::objects::{O_FARM, O_ROADN, O_TIMBERYARD};
use atlantis_engine::defs::terrains::R_PLAIN;
use atlantis_engine::economy::{manual_setup, setup_prods, setup_region};
use atlantis_engine::model::{Game, Object, Region};
use atlantis_engine::rng::Rng;
use atlantis_engine::ruleset::Ruleset;
use atlantis_engine::world::TownType;
use serde_json::{json, Value};

fn setup_snapshot(ruleset_name: &str) -> Value {
    let ruleset = <dyn Ruleset>::from_name(ruleset_name).expect("supported ruleset");
    let tables = ruleset.tables();
    let mut region = Region::default();
    region.terrain_type = Some(R_PLAIN);
    region.race = -1;

    let mut game = Game::default();
    let region_id = game.add_region(region);
    let mut rng = Rng::new(424242);
    setup_region(
        &mut game,
        region_id,
        ruleset.game_defs(),
        &tables,
        false,
        1.0,
        &mut rng,
        &mut |_, _| "Unused town".to_owned(),
    )
    .expect("valid plain region");

    let region = game.region(region_id).expect("region exists");
    let mut manual_game = Game::default();
    let mut manual_region = Region::default();
    manual_region.terrain_type = Some(R_PLAIN);
    manual_region.race = region.race;
    let manual_region_id = manual_game.add_region(manual_region);
    let mut market_rng = Rng::new(424242);
    manual_setup(
        &mut manual_game,
        manual_region_id,
        R_PLAIN,
        1000,
        1.0,
        false,
        Some((TownType::City, "Reference City".to_owned())),
        ruleset.game_defs(),
        &tables,
        false,
        &mut market_rng,
    )
    .expect("valid manual setup probe");
    let manual_region = manual_game
        .region(manual_region_id)
        .expect("manual region exists");

    let mut coastal_game = Game::default();
    let mut coastal_region = Region::default();
    coastal_region.terrain_type = Some(R_PLAIN);
    coastal_region.race = -1;
    let coastal_region_id = coastal_game.add_region(coastal_region);
    let mut coastal_rng = Rng::new(0);
    manual_setup(
        &mut coastal_game,
        coastal_region_id,
        R_PLAIN,
        1000,
        1.0,
        false,
        None,
        ruleset.game_defs(),
        &tables,
        true,
        &mut coastal_rng,
    )
    .expect("valid coastal manual setup probe");
    let coastal_region = coastal_game
        .region(coastal_region_id)
        .expect("coastal region exists");

    let mut town_habitat_cases = Vec::new();
    for object_type in [O_FARM, O_TIMBERYARD, O_ROADN] {
        let mut case_game = Game::default();
        let mut case_region = Region::default();
        case_region.terrain_type = Some(R_PLAIN);
        case_region.race = region.race;
        let case_region_id = case_game.add_region(case_region);
        let mut object = Object::default();
        object.object_type = object_type as i32;
        case_game
            .add_object(case_region_id, object)
            .expect("add town habitat probe object");
        let mut case_rng = Rng::new(424242);
        manual_setup(
            &mut case_game,
            case_region_id,
            R_PLAIN,
            1000,
            1.0,
            false,
            Some((TownType::City, "Reference City".to_owned())),
            ruleset.game_defs(),
            &tables,
            false,
            &mut case_rng,
        )
        .expect("valid town habitat probe");
        let case_region = case_game
            .region(case_region_id)
            .expect("town habitat probe region exists");
        let town = case_region.town.as_ref().expect("town was set");
        town_habitat_cases.push(json!({
            "object_type": object_type,
            "region_habitat": case_region.habitat,
            "development": case_region.development,
            "max_development": case_region.max_development,
            "town": {
                "population": town.population,
                "habitat": town.habitat,
                "development": town.development,
            },
        }));
    }

    json!({
        "seed": 424242,
        "terrain": region.terrain_type.expect("terrain set") as i32,
        "is_coastal": false,
        "race": region.race,
        "population": region.population,
        "base_population": region.base_population,
        "wages": region.wages,
        "max_wages": region.max_wages,
        "wealth": region.wealth,
        "habitat": region.habitat,
        "development": region.development,
        "max_development": region.max_development,
        "building_seq": region.building_seq,
        "town": region.town.as_ref().map(|town| json!({
            "name": town.name,
            "population": town.population,
            "habitat": town.habitat,
            "development": town.development,
            "activity": town.activity,
        })),
        "products": region.products.iter().map(|product| json!({
            "item_type": product.item_type,
            "base_amount": product.base_amount,
            "amount": product.amount,
            "skill": product.skill,
            "productivity": product.productivity,
            "activity": product.activity,
        })).collect::<Vec<_>>(),
        "markets": region.markets.iter().map(|market| json!({
            "market_type": market.market_type as i32,
            "item": market.item,
            "price": market.price,
            "amount": market.amount,
            "min_population": market.min_population,
            "max_population": market.max_population,
            "min_amount": market.min_amount,
            "max_amount": market.max_amount,
            "base_price": market.base_price,
            "activity": market.activity,
        })).collect::<Vec<_>>(),
        "objects": region.object_ids().iter().map(|object_id| {
            let object = game.object(*object_id).expect("region object exists");
            json!({
                "object_type": object.object_type,
                "num": object.num,
                "name": object.name,
            })
        }).collect::<Vec<_>>(),
        "manual_setup": {
            "race": manual_region.race,
            "population": manual_region.population,
            "base_population": manual_region.base_population,
            "wages": manual_region.wages,
            "max_wages": manual_region.max_wages,
            "wealth": manual_region.wealth,
            "habitat": manual_region.habitat,
            "development": manual_region.development,
            "max_development": manual_region.max_development,
            "town": manual_region.town.as_ref().map(|town| json!({
                "name": town.name,
                "population": town.population,
                "habitat": town.habitat,
                "development": town.development,
                "activity": town.activity,
            })),
            "products": manual_region.products.iter().map(|product| json!({
                "item_type": product.item_type,
                "base_amount": product.base_amount,
                "amount": product.amount,
                "skill": product.skill,
                "productivity": product.productivity,
                "activity": product.activity,
            })).collect::<Vec<_>>(),
            "markets": manual_region.markets.iter().map(|market| json!({
                "market_type": market.market_type as i32,
                "item": market.item,
                "price": market.price,
                "amount": market.amount,
                "min_population": market.min_population,
                "max_population": market.max_population,
                "min_amount": market.min_amount,
                "max_amount": market.max_amount,
                "base_price": market.base_price,
                "activity": market.activity,
            })).collect::<Vec<_>>(),
            "objects": manual_region.object_ids().iter().map(|object_id| {
                let object = manual_game.object(*object_id).expect("manual region object exists");
                json!({
                    "object_type": object.object_type,
                    "num": object.num,
                    "name": object.name,
                })
            }).collect::<Vec<_>>(),
        },
        "coastal_setup": {
            "race": coastal_region.race,
            "population": coastal_region.population,
            "base_population": coastal_region.base_population,
            "wages": coastal_region.wages,
            "max_wages": coastal_region.max_wages,
            "wealth": coastal_region.wealth,
            "habitat": coastal_region.habitat,
            "development": coastal_region.development,
            "max_development": coastal_region.max_development,
            "products": coastal_region.products.iter().map(|product| json!({
                "item_type": product.item_type,
                "base_amount": product.base_amount,
                "amount": product.amount,
                "skill": product.skill,
                "productivity": product.productivity,
                "activity": product.activity,
            })).collect::<Vec<_>>(),
            "markets": coastal_region.markets.iter().map(|market| json!({
                "market_type": market.market_type as i32,
                "item": market.item,
                "price": market.price,
                "amount": market.amount,
                "min_population": market.min_population,
                "max_population": market.max_population,
                "min_amount": market.min_amount,
                "max_amount": market.max_amount,
                "base_price": market.base_price,
                "activity": market.activity,
            })).collect::<Vec<_>>(),
        },
        "town_habitat_cases": town_habitat_cases,
    })
}

#[test]
fn reference_region_setup_fixtures_match() {
    let fixture_dir =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/economy/region-setup");
    for ruleset in ["standard", "neworigins"] {
        let path = fixture_dir.join(format!("{ruleset}.jsonl"));
        let expected: Value = serde_json::from_slice(
            &fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display())),
        )
        .unwrap_or_else(|error| panic!("parse {}: {error}", path.display()));
        assert_eq!(
            setup_snapshot(ruleset),
            expected,
            "setup differs for {ruleset}"
        );
    }
}

#[test]
fn coastal_fallback_seed_selects_the_reference_out_of_bounds_slot() {
    let ruleset = <dyn Ruleset>::from_name("standard").expect("supported ruleset");
    let tables = ruleset.tables();
    let terrain = &tables.terrains[R_PLAIN];
    let defs = *ruleset.game_defs();
    let mut region = Region::default();
    let mut rng = Rng::new(0);
    setup_prods(&mut region, terrain, &defs, &tables, true, 1.0, &mut rng);
    let choice = rng.get_random((terrain.races.len() + terrain.coastal_races.len()) as i32);

    assert_eq!(choice, terrain.races.len() as i32);
}
