use std::fs;
use std::path::Path;

use atlantis_engine::defs::terrains::R_PLAIN;
use atlantis_engine::economy::{manual_setup, setup_region};
use atlantis_engine::model::{Game, Region};
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
