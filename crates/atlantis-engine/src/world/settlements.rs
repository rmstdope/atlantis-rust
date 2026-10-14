use crate::defs::{
    objects::{object_flags, O_DUMMY},
    GameDefs, ItemFlags, ItemId, ItemTypeFlags, RulesetTables, TerrainDef,
};
use crate::economy::{region_wages, setup_city_market, SetupError};
use crate::model::{Game, Market, MarketType, Region, RegionId, TownInfo};
use crate::rng::Rng;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum TownType {
    Village = 0,
    Town = 1,
    City = 2,
}

impl TryFrom<i32> for TownType {
    type Error = SetupError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Village),
            1 => Ok(Self::Town),
            2 => Ok(Self::City),
            _ => Err(SetupError::InvalidTownType(value)),
        }
    }
}

/// C++: `economy.cpp` `ARegion::DetermineTownSize`.
pub(crate) fn determine_town_size(defs: &GameDefs, rng: &mut Rng) -> TownType {
    if rng.get_random(300) < defs.town_development {
        TownType::City
    } else if rng.get_random(220) < defs.town_development + 10 {
        TownType::Town
    } else {
        TownType::Village
    }
}

/// C++: `economy.cpp` `ARegion::AddTown` and `ARegion::SetTownType`.
pub(crate) fn add_town(
    game: &mut Game,
    region_id: RegionId,
    town_type: TownType,
    name: Option<String>,
    terrain: &TerrainDef,
    defs: &GameDefs,
    tables: &RulesetTables,
    rng: &mut Rng,
) -> Result<(), SetupError> {
    let habitat = town_habitat(game, region_id, terrain, defs, tables)?;
    let development = town_development(
        game.region(region_id)
            .ok_or(SetupError::MissingRegion(region_id))?,
        terrain,
        defs,
    );
    game.region_mut(region_id)
        .ok_or(SetupError::MissingRegion(region_id))?
        .town = Some(TownInfo {
        name,
        population: habitat * 2 / 3,
        activity: 0,
        habitat,
        development,
    });
    let level = town_type as i32;
    {
        let region = game
            .region_mut(region_id)
            .ok_or(SetupError::MissingRegion(region_id))?;
        let population = rng.get_random((level - 1) * (level - 1) * defs.city_pop / 12)
            + level * level * defs.city_pop / 12;
        let town = region.town.as_mut().expect("town was set above");
        town.habitat += population;
        town.population = town.habitat * 2 / 3;
        region.development += level * 6 + 2;
    }
    {
        let region = game
            .region(region_id)
            .ok_or(SetupError::MissingRegion(region_id))?;
        let development = town_development(region, terrain, defs);
        game.region_mut(region_id)
            .expect("region was checked above")
            .town
            .as_mut()
            .expect("town was set above")
            .development = development;
    }

    loop {
        let (current_type, current_population, current_habitat) = {
            let region = game
                .region(region_id)
                .ok_or(SetupError::MissingRegion(region_id))?;
            let town = region.town.as_ref().expect("town was set above");
            (
                town_type_value(town, defs.city_pop),
                town.population,
                town.habitat,
            )
        };
        if current_type == level {
            break;
        }
        if level > current_type {
            let development_increase = rng.get_random(defs.town_development / 10 + 5);
            let mut population =
                rng.get_random(defs.city_pop / 3) + rng.get_random(defs.city_pop / 3);
            game.region_mut(region_id)
                .expect("region was checked above")
                .development += development_increase;
            while current_population + population > defs.city_pop {
                population /= 2;
            }
            {
                let town = game
                    .region_mut(region_id)
                    .expect("region was checked above")
                    .town
                    .as_mut()
                    .expect("town was set above");
                town.habitat += population;
                town.population = town.habitat * 2 / 3;
            }
            let development = town_development(
                game.region(region_id)
                    .ok_or(SetupError::MissingRegion(region_id))?,
                terrain,
                defs,
            );
            game.region_mut(region_id)
                .expect("region was checked above")
                .town
                .as_mut()
                .expect("town was set above")
                .development = development;
        } else {
            let development_decrease = rng.get_random(20 - defs.town_development / 10);
            let mut population =
                rng.get_random(defs.city_pop / 3) + rng.get_random(defs.city_pop / 3);
            game.region_mut(region_id)
                .expect("region was checked above")
                .development -= development_decrease;
            while current_population < population || current_habitat < population {
                population /= 2;
            }
            {
                let town = game
                    .region_mut(region_id)
                    .expect("region was checked above")
                    .town
                    .as_mut()
                    .expect("town was set above");
                town.habitat -= population;
                town.population = town.habitat * 2 / 3;
            }
            let development = town_development(
                game.region(region_id)
                    .ok_or(SetupError::MissingRegion(region_id))?,
                terrain,
                defs,
            );
            game.region_mut(region_id)
                .expect("region was checked above")
                .town
                .as_mut()
                .expect("town was set above")
                .development = development;
        }
    }
    {
        let region = game
            .region_mut(region_id)
            .ok_or(SetupError::MissingRegion(region_id))?;
        region.max_development = region.development;
        setup_city_market(region, terrain, defs, tables, rng)?;
    }
    remove_lairs(game, region_id, tables)?;
    Ok(())
}

/// C++: `standard/world.cpp` and `neworigins/world.cpp` `ARegion::MakeStartingCity`.
pub fn make_starting_city<F>(
    game: &mut Game,
    region_id: RegionId,
    defs: &GameDefs,
    tables: &RulesetTables,
    rng: &mut Rng,
    town_name: &mut F,
) -> Result<(), SetupError>
where
    F: FnMut(&mut Rng, &Region) -> String,
{
    if defs.towns_exist == 0 {
        return Ok(());
    }
    let terrain_id = game
        .region(region_id)
        .ok_or(SetupError::MissingRegion(region_id))?
        .terrain_type
        .ok_or(SetupError::MissingTerrain(region_id))?;
    let terrain = tables
        .terrains
        .get(terrain_id)
        .ok_or(SetupError::InvalidTerrain(terrain_id as i32))?;
    {
        let region = game
            .region_mut(region_id)
            .ok_or(SetupError::MissingRegion(region_id))?;
        if defs.gates_exist != 0 {
            region.gate = -1;
        }
        region.town = None;
    }
    let name = town_name(
        rng,
        game.region(region_id)
            .ok_or(SetupError::MissingRegion(region_id))?,
    );
    add_town(
        game,
        region_id,
        TownType::City,
        Some(name),
        terrain,
        defs,
        tables,
        rng,
    )?;

    if defs.start_cities_exist == 0 {
        return Ok(());
    }

    {
        let region = game
            .region_mut(region_id)
            .ok_or(SetupError::MissingRegion(region_id))?;
        let town = region.town.as_mut().expect("starting city has a town");
        town.habitat = 125 * defs.city_pop / 100;
        while town.population < town.habitat {
            town.population += rng.get_random(200) + 200;
        }
        region.markets.clear();
    }
    {
        let development = town_development(
            game.region(region_id)
                .ok_or(SetupError::MissingRegion(region_id))?,
            terrain,
            defs,
        );
        game.region_mut(region_id)
            .expect("region was checked above")
            .town
            .as_mut()
            .expect("starting city has a town")
            .development = development;
    }

    if defs.start_cities_start_unlimited != 0 {
        let region = game
            .region_mut(region_id)
            .ok_or(SetupError::MissingRegion(region_id))?;
        for (index, item) in tables.item_families.items.iter().enumerate() {
            if item.flags & ItemFlags::DISABLED != 0
                || item.item_type & ItemTypeFlags::NORMAL == 0
                || [
                    ItemId::Silver.index(),
                    ItemId::Livestock.index(),
                    ItemId::Fish.index(),
                    ItemId::Grain.index(),
                ]
                .contains(&index)
            {
                continue;
            }
            region.markets.push(Market {
                market_type: MarketType::Buy,
                item: index as i32,
                price: item.base_price * 5 / 2,
                amount: -1,
                min_population: 5000,
                max_population: 5000,
                min_amount: -1,
                max_amount: -1,
                base_price: item.base_price * 5 / 2,
                activity: 0,
            });
        }
        add_starting_recruitment_markets(game, region_id, defs, tables)?;
    } else {
        let region = game
            .region_mut(region_id)
            .ok_or(SetupError::MissingRegion(region_id))?;
        setup_city_market(region, terrain, defs, tables, rng)?;
        add_starting_recruitment_markets(game, region_id, defs, tables)?;
    }
    Ok(())
}

fn add_starting_recruitment_markets(
    game: &mut Game,
    region_id: RegionId,
    defs: &GameDefs,
    tables: &RulesetTables,
) -> Result<(), SetupError> {
    let region = game
        .region_mut(region_id)
        .ok_or(SetupError::MissingRegion(region_id))?;
    let population = region.population + region.town.as_ref().map_or(0, |town| town.population);
    let race = tables
        .item_families
        .items
        .get(usize::try_from(region.race).unwrap_or(usize::MAX))
        .ok_or(SetupError::InvalidRaceItem(region.race))?;
    let race_ratio = race.base_price as f32 / (defs.base_man_cost as f32 * 10.0);
    let wages = region_wages(region, defs, population);
    let race_price = ((wages * 4) as f32 * race_ratio) as i32;
    let unlimited = defs.start_cities_start_unlimited != 0;
    region.markets.push(Market {
        market_type: MarketType::Buy,
        item: region.race,
        price: race_price,
        amount: if unlimited { -1 } else { population / 5 },
        min_population: if unlimited { 5000 } else { 0 },
        max_population: if unlimited { 5000 } else { 10_000 },
        min_amount: if unlimited { -1 } else { 0 },
        max_amount: if unlimited { -1 } else { 2000 },
        base_price: race_price,
        activity: 0,
    });
    if defs.leaders_exist != 0 {
        let leaders = &tables.item_families.items[ItemId::Leaders.index()];
        let ratio = leaders.base_price as f32 / (defs.base_man_cost as f32 * 10.0);
        let price = ((wages * 4) as f32 * ratio) as i32;
        region.markets.push(Market {
            market_type: MarketType::Buy,
            item: ItemId::Leaders.index() as i32,
            price,
            amount: if unlimited { -1 } else { population / 25 },
            min_population: if unlimited { 5000 } else { 0 },
            max_population: if unlimited { 5000 } else { 10_000 },
            min_amount: if unlimited { -1 } else { 0 },
            max_amount: if unlimited { -1 } else { 400 },
            base_price: price,
            activity: 0,
        });
    }
    Ok(())
}

/// C++: `economy.cpp` `ARegion::TownHabitat`.
fn town_habitat(
    game: &Game,
    region_id: RegionId,
    terrain: &TerrainDef,
    defs: &GameDefs,
    tables: &RulesetTables,
) -> Result<i32, SetupError> {
    let region = game
        .region(region_id)
        .ok_or(SetupError::MissingRegion(region_id))?;
    let mut fort = 0;
    let mut farm = false;
    let mut inn = false;
    let mut temple = false;
    let mut caravan = false;
    for object_id in region.object_ids() {
        let object = game
            .object(*object_id)
            .ok_or(SetupError::MissingObject(*object_id))?;
        let object_type = usize::try_from(object.object_type)
            .ok()
            .and_then(|index| tables.objects.get(index))
            .ok_or(SetupError::InvalidObjectType(object.object_type))?;
        fort = fort.max(object_type.protection);
        let aided = object_type.production_aided;
        let aided_item = usize::try_from(aided)
            .ok()
            .and_then(|index| tables.item_families.items.get(index));
        let aided_flags = aided_item.map_or(0, |item| item.flags);
        if aided_flags & ItemTypeFlags::FOOD != 0 {
            farm = true;
        }
        if aided == ItemId::Silver.index() as i32 {
            inn = true;
        }
        if aided == ItemId::Herbs.index() as i32 {
            temple = true;
        }
        if object_type.flags & object_flags::TRANSPORT != 0
            && aided_flags & ItemTypeFlags::MOUNT != 0
        {
            caravan = true;
        }
    }

    let mut habitat = 2;
    let mut step = 0;
    for _ in 0..5 {
        if fort > step {
            habitat += 1;
        }
        step *= 5;
        if step == 0 {
            step = 10;
        }
    }
    let build_count = farm as i32 + inn as i32 + temple as i32 + caravan as i32;
    let build = build_count.min(2) + 1;
    let habitat = (build * build + 1) * habitat * habitat + region.habitat / 4 + 50;
    Ok(habitat
        + (town_development(region, terrain, defs)
            * (region.habitat + 800 + habitat + defs.city_pop / 2))
            / 100)
}

/// C++: `economy.cpp` `ARegion::TownDevelopment`.
fn town_development(region: &Region, terrain: &TerrainDef, defs: &GameDefs) -> i32 {
    let mut level = 1;
    let mut base_development = 1;
    let mut previous = 0;
    while level <= terrain.wages {
        previous += 1;
        base_development += 1;
        if previous > level {
            level += 1;
            previous = 0;
        }
    }
    base_development = (defs.maintenance_cost + base_development) / 2;
    (region.development - base_development).clamp(0, 100)
}

fn town_type_value(town: &TownInfo, city_population: i32) -> i32 {
    let prestige = town.population * (town.development + 220) / 270;
    if prestige < city_population / 4 {
        TownType::Village as i32
    } else if prestige < city_population * 4 / 5 {
        TownType::Town as i32
    } else {
        TownType::City as i32
    }
}

fn remove_lairs(
    game: &mut Game,
    region_id: RegionId,
    tables: &RulesetTables,
) -> Result<(), SetupError> {
    let object_ids = game
        .region(region_id)
        .ok_or(SetupError::MissingRegion(region_id))?
        .object_ids()
        .to_vec();
    for object_id in object_ids {
        let object = game
            .object(object_id)
            .ok_or(SetupError::MissingObject(object_id))?;
        if object.object_type == O_DUMMY as i32 {
            continue;
        }
        let object_type = usize::try_from(object.object_type)
            .ok()
            .and_then(|index| tables.objects.get(index))
            .ok_or(SetupError::InvalidObjectType(object.object_type))?;
        if object_type.monster != -1 && object_type.flags & object_flags::CANENTER == 0 {
            game.remove_empty_object(object_id)?;
        }
    }
    Ok(())
}
