use crate::defs::{
    objects::object_flags, skills::lookup_skill, GameDefs, ItemFlags, ItemId, ItemTypeFlags,
    ManDef, RulesetTables, TerrainDef,
};
use crate::model::{Game, Market, MarketType, ModelError, Object, Production, Region, RegionId};
use crate::rng::Rng;

/// C++: `economy.cpp` `ARegion::SetupProds`.
pub fn setup_prods(
    region: &mut Region,
    terrain: &TerrainDef,
    defs: &GameDefs,
    tables: &RulesetTables,
    is_coastal: bool,
    weight: f64,
    rng: &mut Rng,
) {
    if defs.food_items_exist != 0 && terrain.economy != 0 {
        let food_count = 2 + i32::from(defs.coastal_fish != 0 && is_coastal);
        let food_choice = rng.get_random(food_count);
        let item = match food_choice {
            0 => Some(ItemId::Grain.index()),
            1 => Some(ItemId::Livestock.index()),
            2 => Some(ItemId::Fish.index()),
            _ => None,
        };
        if let Some(item) = item.filter(|item| !item_is_disabled(tables, *item)) {
            region.products.push(new_production(
                item as i32,
                terrain.economy,
                defs,
                tables,
                rng,
            ));
        }
    }

    for product in &terrain.products {
        let Some(item) = usize::try_from(product.item).ok() else {
            continue;
        };
        if item_is_disabled(tables, item) {
            continue;
        }
        let chance = (f64::from(product.chance) * weight) as i32;
        if rng.get_random(100) < chance {
            region.products.push(new_production(
                product.item,
                product.amount,
                defs,
                tables,
                rng,
            ));
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SetupError {
    InvalidRaceItem(i32),
    MissingRaceDefinition(i32),
    MissingArmorDefinition(i32),
    InvalidArmorDefinition(i32),
    TooFewTradeGoods,
    InvalidTownType(i32),
    InvalidObjectType(i32),
    MissingObject(crate::model::ObjectId),
    MissingRegion(RegionId),
    MissingTerrain(RegionId),
    InvalidTerrain(i32),
    Model(ModelError),
}

impl From<ModelError> for SetupError {
    fn from(error: ModelError) -> Self {
        Self::Model(error)
    }
}

/// C++: `economy.cpp` `ARegion::SetupCityMarket`.
pub fn setup_city_market(
    region: &mut Region,
    terrain: &TerrainDef,
    defs: &GameDefs,
    tables: &RulesetTables,
    rng: &mut Rng,
) -> Result<(), SetupError> {
    let race_item =
        item_definition(tables, region.race).ok_or(SetupError::InvalidRaceItem(region.race))?;
    let locals = find_race(tables, race_item.abbreviation.as_deref().unwrap_or(""))
        .or_else(|| find_race(tables, "SELF"))
        .ok_or(SetupError::MissingRaceDefinition(region.race))?;

    let item_count = tables.item_families.items.len();
    let mut supply = vec![0; item_count];
    let mut demand = vec![0; item_count];
    let mut rare = vec![0; item_count];
    let mut antiques = vec![0; item_count];
    let mut trade_count = 0;
    let city_max = defs.city_pop;

    for (index, item) in tables.item_families.items.iter().enumerate() {
        if item.flags & (ItemFlags::DISABLED | ItemFlags::NOMARKET) != 0
            || item.item_type & ItemTypeFlags::SHIP != 0
        {
            continue;
        }
        if item.item_type & ItemTypeFlags::TRADE != 0 {
            trade_count += 1;
        }
        if index == ItemId::Silver.index()
            || item.item_type & (ItemTypeFlags::MAN | ItemTypeFlags::LEADER) != 0
        {
            continue;
        }

        let can_produce_here = if item.production_inputs[0].item == -1 {
            terrain
                .products
                .iter()
                .any(|product| product.item == index as i32)
        } else {
            item.production_inputs.iter().all(|input| {
                terrain
                    .products
                    .iter()
                    .any(|product| product.item == input.item)
            })
        };
        let can_produce = can_produce_here && race_can_produce(locals, item);
        let is_useful = race_can_use(locals, item, index, tables)?;

        if item.item_type & ItemTypeFlags::NORMAL != 0 {
            if matches!(
                index,
                value if value == ItemId::Grain.index()
                    || value == ItemId::Livestock.index()
                    || value == ItemId::Fish.index()
            ) {
                let mut amount = defs.city_market_normal_amt;
                let price = if defs.random_economy != 0 {
                    amount += rng.get_random(amount);
                    (item.base_price * (100 + rng.get_random(50))) / 100
                } else {
                    item.base_price
                };
                let mut cap = city_max * 3 / 4 - 5000;
                if cap < 0 {
                    cap = city_max / 2;
                }
                push_market(
                    region,
                    MarketType::Sell,
                    index,
                    price,
                    amount,
                    region.population,
                    region.population + cap,
                    amount,
                    amount * 2,
                );
            } else if index == ItemId::Food.index() {
                let mut amount = defs.city_market_normal_amt;
                let price = if defs.random_economy != 0 {
                    amount += rng.get_random(amount);
                    (item.base_price * (120 + rng.get_random(80))) / 100
                } else {
                    item.base_price
                };
                let mut cap = city_max * 3 / 4 - 5000;
                if cap < 0 {
                    cap = city_max / 2;
                }
                push_market(
                    region,
                    MarketType::Buy,
                    index,
                    price,
                    amount,
                    region.population,
                    region.population + 2 * cap,
                    amount,
                    amount * 5,
                );
            } else if item.production_inputs[0].item == -1 {
                if can_produce {
                    supply[index] = 4;
                }
                if !can_produce_here {
                    if item.item_type & ItemTypeFlags::MOUNT != 0 {
                        if race_can_produce(locals, item) {
                            demand[index] = 4;
                        }
                    } else if is_useful {
                        demand[index] = 4;
                    }
                }
            } else if is_useful {
                if can_produce {
                    supply[index] = 2;
                }
                if !can_produce_here {
                    demand[index] = 2;
                }
            }
        } else if defs.city_market_advanced_amt != 0
            && item.item_type & ItemTypeFlags::ADVANCED != 0
        {
            if is_useful {
                rare[index] = 4;
            }
            if item.hitch_item > 0 {
                rare[index] = 2;
            }
        } else if defs.city_market_magic_amt != 0 && item.item_type & ItemTypeFlags::MAGIC != 0 {
            antiques[index] = if is_useful { 4 } else { 1 };
            if item.hitch_item > 0 {
                antiques[index] = 2;
            }
        }
    }

    if defs.city_market_advanced_amt != 0 && rng.get_random(4) == 1 {
        if let Some(index) = weighted_item(&rare, rng) {
            let item = &tables.item_families.items[index];
            let (amount, price) = randomized_market_values(
                defs.city_market_advanced_amt,
                item.base_price,
                100,
                50,
                defs,
                rng,
            );
            let mut cap = city_max * 3 / 4 - 5000;
            if cap < city_max / 2 {
                cap = city_max / 2;
            }
            let offset = city_max / 8;
            if cap + offset < city_max {
                push_market(
                    region,
                    MarketType::Sell,
                    index,
                    price,
                    amount / 6,
                    region.population + cap + offset,
                    region.population + city_max,
                    0,
                    amount,
                );
            }
        }
    }

    if defs.city_market_magic_amt != 0 && rng.get_random(8) == 1 {
        if let Some(index) = weighted_item(&antiques, rng) {
            let item = &tables.item_families.items[index];
            let (amount, price) = randomized_market_values(
                defs.city_market_magic_amt,
                item.base_price,
                100,
                50,
                defs,
                rng,
            );
            let mut cap = city_max * 3 / 4 - 5000;
            if cap < city_max / 2 {
                cap = city_max / 2;
            }
            push_market(
                region,
                MarketType::Sell,
                index,
                price,
                amount / 6,
                region.population + cap,
                region.population + city_max,
                0,
                amount,
            );
        }
    }

    let mut remaining = 4;
    let mut total = 1;
    while remaining > 0 && total > 0 {
        let Some(index) = weighted_item(&demand, rng) else {
            break;
        };
        total = demand.iter().sum();
        let item = &tables.item_families.items[index];
        let amount = demand[index] * defs.city_market_normal_amt / 4;
        let (amount, price) = randomized_market_values(amount, item.base_price, 100, 50, defs, rng);
        let cap = city_max / 4;
        let offset = -(city_max / 20) + ((5 - remaining) * city_max * 3 / 40);
        push_market(
            region,
            MarketType::Sell,
            index,
            price,
            amount / 6,
            region.population + cap + offset,
            region.population + city_max,
            0,
            amount,
        );
        demand[index] = 0;
        remaining -= 1;
    }

    remaining = 2;
    total = 1;
    while remaining > 0 && total > 0 {
        let Some(index) = weighted_item(&supply, rng) else {
            break;
        };
        total = supply.iter().sum();
        let item = &tables.item_families.items[index];
        let amount = supply[index] * defs.city_market_normal_amt / 4;
        let (amount, price) = randomized_market_values(amount, item.base_price, 150, 50, defs, rng);
        let cap = city_max / 4;
        let mut offset = (3 - remaining) * city_max * 3 / 40;
        if supply[index] < 4 {
            offset += city_max / 20;
        }
        push_market(
            region,
            MarketType::Buy,
            index,
            price,
            0,
            region.population + cap + offset,
            region.population + city_max,
            0,
            amount,
        );
        supply[index] = 0;
        remaining -= 1;
    }

    if trade_count < 4 {
        return Err(SetupError::TooFewTradeGoods);
    }
    let _ = rng.get_random(trade_count);
    let mut buy_two = rng.get_random(trade_count);
    let mut sell_one = rng.get_random(trade_count);
    let mut sell_two = rng.get_random(trade_count);
    let mut buy_one = rng.get_random(trade_count);
    while buy_one == buy_two {
        buy_two = rng.get_random(trade_count);
    }
    while sell_one == buy_one || sell_one == buy_two {
        sell_one = rng.get_random(trade_count);
    }
    while sell_two == sell_one || sell_two == buy_two || sell_two == buy_one {
        sell_two = rng.get_random(trade_count);
    }

    let mut trade_buy = 0;
    let mut trade_sell = 0;
    for (index, item) in tables.item_families.items.iter().enumerate() {
        if item.flags & (ItemFlags::DISABLED | ItemFlags::NOMARKET) != 0 {
            continue;
        }
        if item.item_type & ItemTypeFlags::TRADE == 0 {
            continue;
        }

        let add_buy = buy_one == 0 || buy_two == 0;
        buy_one -= 1;
        buy_two -= 1;
        let add_sell = sell_one == 0 || sell_two == 0;
        sell_one -= 1;
        sell_two -= 1;

        if add_buy {
            let mut amount = defs.city_market_trade_amt;
            let price = if defs.random_economy != 0 {
                amount += rng.get_random(amount);
                if defs.more_profitable_trade_goods != 0 {
                    (item.base_price * (250 + rng.get_random(100))) / 100
                } else {
                    (item.base_price * (150 + rng.get_random(50))) / 100
                }
            } else {
                item.base_price
            };
            let cap = city_max / 2;
            trade_sell += 1;
            let offset = -(city_max / 20) + trade_sell * (trade_sell * trade_sell * city_max / 40);
            if cap + offset < city_max {
                push_market(
                    region,
                    MarketType::Sell,
                    index,
                    price,
                    amount / 5,
                    region.population + cap + offset,
                    region.population + city_max,
                    0,
                    amount,
                );
            }
        }

        if add_sell {
            let mut amount = defs.city_market_trade_amt;
            let price = if defs.random_economy != 0 {
                amount += rng.get_random(amount);
                if defs.more_profitable_trade_goods != 0 {
                    (item.base_price * (100 + rng.get_random(90))) / 100
                } else {
                    (item.base_price * (100 + rng.get_random(50))) / 100
                }
            } else {
                item.base_price
            };
            let cap = city_max / 2;
            let offset = trade_buy * (city_max / 6);
            trade_buy += 1;
            if cap + offset < city_max {
                push_market(
                    region,
                    MarketType::Buy,
                    index,
                    price,
                    amount / 6,
                    region.population + cap + offset,
                    region.population + city_max,
                    0,
                    amount,
                );
            }
        }
    }

    Ok(())
}

/// C++: `economy.cpp` `ARegion::SetupPop`.
#[allow(clippy::too_many_arguments)]
pub fn setup_pop<F>(
    game: &mut Game,
    region_id: RegionId,
    terrain: &TerrainDef,
    defs: &GameDefs,
    tables: &RulesetTables,
    is_coastal: bool,
    rng: &mut Rng,
    town_name: &mut F,
) -> Result<(), SetupError>
where
    F: FnMut(&mut Rng, &Region) -> String,
{
    let neighbors = game
        .region(region_id)
        .ok_or(SetupError::MissingRegion(region_id))?
        .neighbors;
    {
        let region = game
            .region_mut(region_id)
            .ok_or(SetupError::MissingRegion(region_id))?;
        region.habitat = terrain.population + 1;
        setup_habitat(region, terrain, defs, tables, is_coastal, rng)?;
        if region.population == 0 {
            return Ok(());
        }
    }

    if defs.towns_exist != 0 {
        let adjacent = neighbors
            .iter()
            .flatten()
            .filter(|neighbor| {
                game.region(**neighbor)
                    .is_some_and(|neighbor| neighbor.town.is_some())
            })
            .count() as i32;
        let probability = if defs.town_probability < 1 {
            100
        } else {
            defs.town_probability
        };
        let mut town_chance = 80_000 / probability;
        if defs.less_arctic_towns != 0 {
            let north = pole_distance(game, region_id, 0);
            let south = pole_distance(game, region_id, 3);
            if north < 9 {
                town_chance += 25 * (9 - north) * (9 - north) * defs.less_arctic_towns;
            }
            if south < 9 {
                town_chance += 25 * (9 - south) * (9 - south) * defs.less_arctic_towns;
            }
        }
        let spread = defs.town_spread.min(100);
        let mut town_probability = (terrain.economy * 4 * (100 - spread) + 100 * spread) / 100;
        if adjacent > 0 {
            town_probability = town_probability * (100 - defs.towns_not_adjacent) / 100;
        }
        if rng.get_random(town_chance) < town_probability {
            let region = game
                .region(region_id)
                .ok_or(SetupError::MissingRegion(region_id))?;
            let name = town_name(rng, region);
            let town_size = crate::world::settlements::determine_town_size(defs, rng);
            crate::world::settlements::add_town(
                game,
                region_id,
                town_size,
                Some(name),
                terrain,
                defs,
                tables,
                rng,
            )?;
        }
    }

    setup_economy(
        game.region_mut(region_id)
            .ok_or(SetupError::MissingRegion(region_id))?,
        defs,
        tables,
    )
}

/// C++: `aregion.cpp` `ARegion::Setup`.
#[allow(clippy::too_many_arguments)]
pub fn setup_region<F>(
    game: &mut Game,
    region_id: RegionId,
    defs: &GameDefs,
    tables: &RulesetTables,
    is_coastal: bool,
    product_weight: f64,
    rng: &mut Rng,
    town_name: &mut F,
) -> Result<(), SetupError>
where
    F: FnMut(&mut Rng, &Region) -> String,
{
    let terrain_id = game
        .region(region_id)
        .ok_or(SetupError::MissingRegion(region_id))?
        .terrain_type
        .ok_or(SetupError::MissingTerrain(region_id))?;
    let terrain = tables
        .terrains
        .get(terrain_id)
        .ok_or(SetupError::InvalidTerrain(terrain_id as i32))?;
    setup_prods(
        game.region_mut(region_id)
            .ok_or(SetupError::MissingRegion(region_id))?,
        terrain,
        defs,
        tables,
        is_coastal,
        product_weight,
        rng,
    );
    setup_pop(
        game, region_id, terrain, defs, tables, is_coastal, rng, town_name,
    )?;
    game.add_object(region_id, Object::default())?;
    if defs.lair_monsters_exist != 0 {
        lair_check(game, region_id, terrain, tables, rng)?;
    }
    Ok(())
}

/// C++: `aregion.cpp` `ARegion::ManualSetup`.
#[allow(clippy::too_many_arguments)]
pub fn manual_setup(
    game: &mut Game,
    region_id: RegionId,
    terrain_id: usize,
    habitat: i32,
    product_weight: f64,
    add_lair: bool,
    settlement: Option<(crate::world::TownType, String)>,
    defs: &GameDefs,
    tables: &RulesetTables,
    is_coastal: bool,
    rng: &mut Rng,
) -> Result<(), SetupError> {
    let terrain = tables
        .terrains
        .get(terrain_id)
        .ok_or(SetupError::InvalidTerrain(terrain_id as i32))?;
    setup_prods(
        game.region_mut(region_id)
            .ok_or(SetupError::MissingRegion(region_id))?,
        terrain,
        defs,
        tables,
        is_coastal,
        product_weight,
        rng,
    );
    {
        let region = game
            .region_mut(region_id)
            .ok_or(SetupError::MissingRegion(region_id))?;
        region.habitat = habitat;
        setup_habitat(region, terrain, defs, tables, is_coastal, rng)?;
    }
    if let Some((town_type, name)) = settlement {
        crate::world::settlements::add_town(
            game,
            region_id,
            town_type,
            Some(name),
            terrain,
            defs,
            tables,
            rng,
        )?;
    }
    setup_economy(
        game.region_mut(region_id)
            .ok_or(SetupError::MissingRegion(region_id))?,
        defs,
        tables,
    )?;
    game.add_object(region_id, Object::default())?;
    if defs.lair_monsters_exist != 0 && add_lair {
        let lairs = possible_lairs(terrain, tables);
        if !lairs.is_empty() {
            let index = rng.get_random(lairs.len() as i32) as usize;
            create_lair(game, region_id, lairs[index], tables)?;
        }
    }
    Ok(())
}

/// C++: `aregion.cpp` `ARegion::LairCheck`.
pub fn lair_check(
    game: &mut Game,
    region_id: RegionId,
    terrain: &TerrainDef,
    tables: &RulesetTables,
    rng: &mut Rng,
) -> Result<(), SetupError> {
    let region = game
        .region(region_id)
        .ok_or(SetupError::MissingRegion(region_id))?;
    if region.town.is_some() || terrain.lair_chance == 0 {
        return Ok(());
    }
    if rng.get_random(100) >= terrain.lair_chance {
        return Ok(());
    }
    let lairs = possible_lairs(terrain, tables);
    if lairs.is_empty() {
        return Ok(());
    }
    let lair_index = rng.get_random(lairs.len() as i32) as usize;
    create_lair(game, region_id, lairs[lair_index], tables)
}

fn possible_lairs(terrain: &TerrainDef, tables: &RulesetTables) -> Vec<i32> {
    terrain
        .lairs
        .iter()
        .copied()
        .filter(|lair| *lair != -1)
        .filter(|lair| {
            usize::try_from(*lair)
                .ok()
                .and_then(|index| tables.objects.get(index))
                .is_some_and(|object| object.flags & object_flags::DISABLED == 0)
        })
        .collect()
}

fn create_lair(
    game: &mut Game,
    region_id: RegionId,
    lair: i32,
    tables: &RulesetTables,
) -> Result<(), SetupError> {
    let definition = usize::try_from(lair)
        .ok()
        .and_then(|index| tables.objects.get(index))
        .ok_or(SetupError::InvalidObjectType(lair))?;
    let region = game
        .region_mut(region_id)
        .ok_or(SetupError::MissingRegion(region_id))?;
    let number = region.building_seq;
    region.building_seq += 1;
    let mut object = Object::default();
    object.name = Some(format!("{} [{}]", definition.name, number));
    object.num = number;
    object.object_type = lair;
    object.incomplete = 0;
    object.inner = -1;
    game.add_object(region_id, object)?;
    Ok(())
}

fn pole_distance(game: &Game, region_id: RegionId, direction: usize) -> i32 {
    let mut distance = 1;
    let mut next = game
        .region(region_id)
        .and_then(|region| region.neighbors.get(direction).copied().flatten());
    while let Some(id) = next {
        distance += 1;
        next = game
            .region(id)
            .and_then(|region| region.neighbors.get(direction).copied().flatten());
    }
    distance
}

fn item_definition(tables: &RulesetTables, item: i32) -> Option<&crate::defs::ItemDef> {
    usize::try_from(item)
        .ok()
        .and_then(|index| tables.item_families.items.get(index))
}

fn find_race<'a>(tables: &'a RulesetTables, abbreviation: &str) -> Option<&'a ManDef> {
    tables.item_families.races.iter().find(|race| {
        race.abbreviation
            .as_deref()
            .is_some_and(|candidate| crate::text::is_equal(candidate, abbreviation))
    })
}

fn race_can_produce(race: &ManDef, item: &crate::defs::ItemDef) -> bool {
    if item.flags & ItemFlags::DISABLED != 0 {
        return false;
    }
    race.skills.iter().flatten().any(|skill| {
        item.production_skill
            .as_deref()
            .is_some_and(|production_skill| crate::text::is_equal(skill, production_skill))
    })
}

fn race_can_use(
    race: &ManDef,
    item: &crate::defs::ItemDef,
    item_id: usize,
    tables: &RulesetTables,
) -> Result<bool, SetupError> {
    if item.flags & ItemFlags::DISABLED != 0 {
        return Ok(false);
    }
    if item.item_type & (ItemTypeFlags::MOUNT | ItemTypeFlags::WEAPON | ItemTypeFlags::TOOL) != 0 {
        return Ok(true);
    }
    if item.item_type & ItemTypeFlags::ARMOR == 0 {
        return Ok(false);
    }

    let abbreviation = item.abbreviation.as_deref().unwrap_or("");
    let armor = tables
        .item_families
        .armor
        .iter()
        .find(|armor| {
            armor
                .abbreviation
                .as_deref()
                .is_some_and(|candidate| crate::text::is_equal(candidate, abbreviation))
        })
        .ok_or(SetupError::MissingArmorDefinition(item_id as i32))?;
    let saving = armor.saves[3];
    if saving == 0 {
        return Err(SetupError::InvalidArmorDefinition(item_id as i32));
    }
    let protection = armor.from_value / saving;
    if protection > 4 {
        return Ok(!race_has_skill(race, "COMB"));
    }
    if protection > 3 {
        return Ok(true);
    }
    Ok(!["SAIL", "HUNT", "STEA", "LBOW"]
        .iter()
        .any(|skill| race_has_skill(race, skill)))
}

fn race_has_skill(race: &ManDef, abbreviation: &str) -> bool {
    race.skills
        .iter()
        .flatten()
        .any(|skill| crate::text::is_equal(skill, abbreviation))
}

fn weighted_item(weights: &[i32], rng: &mut Rng) -> Option<usize> {
    let draw = rng.get_random(weights.iter().sum());
    let mut total = 0;
    for (index, weight) in weights.iter().enumerate() {
        total += weight;
        if draw < total {
            return Some(index);
        }
    }
    None
}

fn randomized_market_values(
    base_amount: i32,
    base_price: i32,
    price_floor: i32,
    price_range: i32,
    defs: &GameDefs,
    rng: &mut Rng,
) -> (i32, i32) {
    if defs.random_economy == 0 {
        return (base_amount, base_price);
    }
    let amount = base_amount + rng.get_random(base_amount);
    let price = (base_price * (price_floor + rng.get_random(price_range))) / 100;
    (amount, price)
}

#[allow(clippy::too_many_arguments)]
fn push_market(
    region: &mut Region,
    market_type: MarketType,
    item: usize,
    price: i32,
    amount: i32,
    min_population: i32,
    max_population: i32,
    min_amount: i32,
    max_amount: i32,
) {
    region.markets.push(Market {
        market_type,
        item: item as i32,
        price,
        amount,
        min_population,
        max_population,
        min_amount,
        max_amount,
        base_price: price,
        activity: 0,
    });
}

/// C++: `economy.cpp` `ARegion::SetupEconomy`.
pub fn setup_economy(
    region: &mut Region,
    defs: &GameDefs,
    tables: &RulesetTables,
) -> Result<(), SetupError> {
    let population = region.population + region.town.as_ref().map_or(0, |town| town.population);
    region.wages = region_wages(region, defs, population);
    region.max_wages = region.wages;

    let maintenance_wages = 10 * defs.maintenance_cost;
    region.wealth = ((population * (region.wages - maintenance_wages) / 50) as f32) as i32;
    if region.wealth < 0 {
        region.wealth = 0;
    }

    let mut wage_population = population;
    if wage_population < 3000 {
        let factor = (120 / (61 - wage_population / 50)) as f32;
        wage_population += ((factor * wage_population as f32 + 3000.0) / (factor + 1.0)) as i32;
    }
    let mut wage_limit =
        ((wage_population * (region.wages - maintenance_wages) / 50) as f32) as i32;
    if wage_limit < 0 {
        wage_limit = 0;
    }

    let mut entertainment_population = population;
    if entertainment_population < 3000 {
        let factor = (entertainment_population / 10 + 300) / 6;
        entertainment_population = entertainment_population * factor / 100;
    }
    let mut entertainment_limit =
        ((entertainment_population * (region.wages - maintenance_wages + 1) / 50) as f32) as i32;
    if entertainment_limit < 0 {
        entertainment_limit = 0;
    }

    let mut wage_production = Production {
        item_type: ItemId::Silver.index() as i32,
        base_amount: wage_limit / defs.work_fraction,
        amount: wage_limit / defs.work_fraction,
        skill: -1,
        productivity: region.wages,
        activity: 0,
    };
    let entertainment_skill = lookup_skill(&tables.skills.skills, "ENTE")
        .and_then(|skill| i32::try_from(skill).ok())
        .unwrap_or(-1);
    let mut entertainment_production = Production {
        item_type: ItemId::Silver.index() as i32,
        base_amount: entertainment_limit / defs.entertain_fraction,
        amount: entertainment_limit / defs.entertain_fraction,
        skill: entertainment_skill,
        productivity: defs.entertain_income * 10,
        activity: 0,
    };

    let is_starting_city = defs.safe_start_cities != 0
        && region
            .town
            .as_ref()
            .is_some_and(|town| town.population >= defs.city_pop * 120 / 100);
    if is_starting_city {
        let bonus = (population / 5) * defs.maintenance_cost;
        region.wealth += bonus;
        let wage_bonus = bonus / defs.work_fraction;
        wage_production.amount += wage_bonus;
        wage_production.base_amount += wage_bonus;
        let entertainment_bonus = bonus / defs.entertain_fraction;
        entertainment_production.amount += entertainment_bonus;
        entertainment_production.base_amount += entertainment_bonus;
    }

    region.products.push(wage_production);
    region.products.push(entertainment_production);

    let race_item = usize::try_from(region.race)
        .ok()
        .and_then(|race| tables.item_families.items.get(race))
        .ok_or(SetupError::InvalidRaceItem(region.race))?;
    let race_ratio = race_item.base_price as f32 / (defs.base_man_cost as f32 * 10.0);
    region.markets.push(Market {
        market_type: MarketType::Buy,
        item: region.race,
        price: ((region.wages * 4) as f32 * race_ratio) as i32,
        amount: population / 25,
        min_population: 0,
        max_population: 10_000,
        min_amount: 0,
        max_amount: 2_000,
        base_price: ((region.wages * 4) as f32 * race_ratio) as i32,
        activity: 0,
    });

    if defs.leaders_exist != 0 {
        let leader_item = &tables.item_families.items[ItemId::Leaders.index()];
        let leader_ratio = leader_item.base_price as f32 / (defs.base_man_cost as f32 * 10.0);
        let price = ((region.wages * 4) as f32 * leader_ratio) as i32;
        region.markets.push(Market {
            market_type: MarketType::Buy,
            item: ItemId::Leaders.index() as i32,
            price,
            amount: population / 125,
            min_population: 0,
            max_population: 10_000,
            min_amount: 0,
            max_amount: 400,
            base_price: price,
            activity: 0,
        });
    }

    Ok(())
}

/// C++: `economy.cpp` `ARegion::Wages`.
/// Setup callers run before world generation adds road exits, so `RoadDevelopment()` is zero.
pub(crate) fn region_wages(region: &Region, defs: &GameDefs, population: i32) -> i32 {
    if population == 0 {
        return 0;
    }

    let town_term = region.town.as_ref().map_or(0, |town| {
        let town_type = town_type(town.population, town.development, defs.city_pop);
        town_type * town_type + 1
    });
    let development = region.development + (region.earthlore + region.clearskies) * 12 + town_term;
    let mut level = 1;
    let mut last = 0;
    let mut wage_steps = 0;
    while development >= level {
        wage_steps += 1;
        last = level;
        level += wage_steps + 1;
    }
    let mut result = wage_steps * 10;
    if development > last {
        result += 10 * (development - last) / (level - last);
    }
    result
}

fn town_type(population: i32, development: i32, city_population: i32) -> i32 {
    let prestige = population * (development + 220) / 270;
    if prestige < city_population / 4 {
        0
    } else if prestige < city_population * 4 / 5 {
        1
    } else {
        2
    }
}

/// C++: `economy.cpp` `ARegion::SetupHabitat`.
pub fn setup_habitat(
    region: &mut Region,
    terrain: &TerrainDef,
    defs: &GameDefs,
    tables: &RulesetTables,
    is_coastal: bool,
    rng: &mut Rng,
) -> Result<(), SetupError> {
    if region.habitat > 1 {
        region.habitat *= 5;
    }
    if region.habitat < 100 && terrain.similar_type != crate::defs::terrains::R_OCEAN as i32 {
        region.habitat = 100;
    }

    let mut minimum_wages = terrain.wages + defs.maintenance_cost - 10;
    if minimum_wages < 0 {
        minimum_wages = 0;
    }

    if terrain.population == 0 {
        region.population = 0;
        region.base_population = 0;
        region.wages = 0;
        region.max_wages = 0;
        region.wealth = 0;
        return Ok(());
    }

    if region.race == -1 || defs.grow_races == 0 {
        let noncoastal_races = terrain.races.len();
        let all_races = noncoastal_races + terrain.coastal_races.len();
        region.race = -1;
        while region.race == -1 || item_is_disabled(tables, region.race as usize) {
            let race_choice = rng.get_random(if is_coastal {
                all_races as i32
            } else {
                noncoastal_races as i32
            }) as usize;
            region.race = if race_choice >= noncoastal_races {
                let coastal_index = race_choice as isize - noncoastal_races as isize - 1;
                if coastal_index < 0 {
                    // C++: economy.cpp SetupHabitat reads coastal_races[-1], which aliases races[3].
                    terrain.races[terrain.races.len() - 1]
                } else {
                    terrain.coastal_races[coastal_index as usize]
                }
            } else {
                terrain.races[race_choice]
            };
        }
    }

    let race_item = usize::try_from(region.race)
        .ok()
        .and_then(|race| tables.item_families.items.get(race))
        .ok_or(SetupError::InvalidRaceItem(region.race))?;
    let race_abbreviation = race_item
        .abbreviation
        .as_deref()
        .ok_or(SetupError::InvalidRaceItem(region.race))?;
    let race_definition = tables
        .item_families
        .races
        .iter()
        .find(|race| {
            race.abbreviation
                .as_deref()
                .is_some_and(|abbreviation| crate::text::is_equal(abbreviation, race_abbreviation))
        })
        .ok_or(SetupError::MissingRaceDefinition(region.race))?;

    region.habitat = region.habitat * 2 / 3 + rng.get_random(region.habitat / 3);
    if race_definition.terrain == terrain.similar_type {
        region.habitat = region.habitat * 9 / 8;
    }
    if !is_native_race(region.race, terrain, is_coastal) {
        region.habitat = region.habitat * 4 / 5;
    }
    region.base_population = region.habitat / 3;
    region.population = region.habitat * (60 + rng.get_random(6) + rng.get_random(6)) / 100;

    let mut level = 1;
    region.development = 1;
    let mut previous = 0;
    while level < minimum_wages {
        region.development += 1;
        previous += 1;
        if previous > level {
            level += 1;
            previous = 0;
        }
    }
    region.development += rng.get_random(25);
    region.max_development = region.development;

    Ok(())
}

fn is_native_race(race: i32, terrain: &TerrainDef, is_coastal: bool) -> bool {
    (is_coastal && terrain.coastal_races.contains(&race)) || terrain.races.contains(&race)
}

fn item_is_disabled(tables: &RulesetTables, item: usize) -> bool {
    tables
        .item_families
        .items
        .get(item)
        .is_none_or(|definition| definition.flags & ItemFlags::DISABLED != 0)
}

fn new_production(
    item: i32,
    maximum_amount: i32,
    defs: &GameDefs,
    tables: &RulesetTables,
    rng: &mut Rng,
) -> Production {
    let amount = maximum_amount
        + if defs.random_economy != 0 {
            rng.get_random(maximum_amount)
        } else {
            0
        };
    let skill = usize::try_from(item)
        .ok()
        .and_then(|index| tables.item_families.items.get(index))
        .and_then(|definition| definition.production_skill.as_deref())
        .and_then(|abbreviation| lookup_skill(&tables.skills.skills, abbreviation))
        .and_then(|skill| i32::try_from(skill).ok())
        .unwrap_or(-1);

    Production {
        item_type: item,
        base_amount: amount,
        amount,
        skill,
        productivity: 10,
        activity: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        lair_check, setup_city_market, setup_economy, setup_habitat, setup_pop, setup_prods,
    };
    use crate::defs::{terrains::R_PLAIN, ItemId};
    use crate::model::{Game, Region, TownInfo};
    use crate::model::{Market, MarketType};
    use crate::rng::Rng;
    use crate::ruleset::Ruleset;
    use crate::world::{make_starting_city, TownType};

    #[test]
    fn setup_prods_uses_terrain_table_order() {
        let ruleset = <dyn Ruleset>::from_name("standard").expect("standard ruleset");
        let tables = ruleset.tables();
        let mut defs = *ruleset.game_defs();
        defs.random_economy = 0;
        let mut region = Region::default();
        region.terrain_type = Some(R_PLAIN);
        let mut rng = Rng::new(424242);

        setup_prods(
            &mut region,
            &tables.terrains[R_PLAIN],
            &defs,
            &tables,
            false,
            1.0,
            &mut rng,
        );

        assert_eq!(
            region
                .products
                .iter()
                .map(|product| (product.item_type, product.amount))
                .collect::<Vec<_>>(),
            [(ItemId::Grain.index() as i32, 40), (28, 20),]
        );
    }

    #[test]
    fn setup_habitat_preserves_reference_race_and_population_draws() {
        let ruleset = <dyn Ruleset>::from_name("standard").expect("standard ruleset");
        let tables = ruleset.tables();
        let mut defs = *ruleset.game_defs();
        defs.grow_races = 1;
        let terrain = &tables.terrains[R_PLAIN];
        let mut region = Region::default();
        region.terrain_type = Some(R_PLAIN);
        region.race = ItemId::Plainsman.index() as i32;
        region.habitat = 10;
        let mut rng = Rng::new(424242);

        setup_habitat(&mut region, terrain, &defs, &tables, false, &mut rng)
            .expect("valid plain region and race");

        assert_eq!(region.race, ItemId::Plainsman.index() as i32);
        assert_eq!(region.habitat, 92);
        assert_eq!(region.base_population, 30);
        assert_eq!(region.population, 57);
        assert_eq!(region.development, 129);
        assert_eq!(region.max_development, 129);
    }

    #[test]
    fn setup_economy_uses_setup_entertainment_bonus() {
        let ruleset = <dyn Ruleset>::from_name("standard").expect("standard ruleset");
        let tables = ruleset.tables();
        let mut region = Region::default();
        region.population = 1000;
        region.race = ItemId::Plainsman.index() as i32;
        region.development = 64;

        setup_economy(&mut region, ruleset.game_defs(), &tables).expect("valid region race");

        assert_eq!(region.wages, 108);
        assert_eq!(region.max_wages, 108);
        assert_eq!(region.wealth, 160);
        assert_eq!(region.products.len(), 2);
        assert_eq!(region.products[0].amount, 142);
        assert_eq!(region.products[1].amount, 5);
        assert_eq!(region.products[1].productivity, 200);
    }

    #[test]
    fn setup_city_market_preserves_market_and_draw_order() {
        let ruleset = <dyn Ruleset>::from_name("standard").expect("standard ruleset");
        let tables = ruleset.tables();
        let mut defs = *ruleset.game_defs();
        defs.random_economy = 0;
        defs.city_market_advanced_amt = 0;
        defs.city_market_magic_amt = 0;
        let mut region = Region::default();
        region.terrain_type = Some(R_PLAIN);
        region.population = 20_000;
        region.race = ItemId::Plainsman.index() as i32;
        let mut rng = Rng::new(424242);

        setup_city_market(
            &mut region,
            &tables.terrains[R_PLAIN],
            &defs,
            &tables,
            &mut rng,
        )
        .expect("valid standard region and race");

        assert_eq!(region.markets[0].item, ItemId::Grain.index() as i32);
        assert_eq!(
            region.markets[0].market_type,
            crate::model::MarketType::Sell
        );
        assert_eq!(region.markets[0].amount, defs.city_market_normal_amt);
        assert_eq!(region.markets[0].min_population, 20_000);
        assert_eq!(region.markets[1].item, ItemId::Livestock.index() as i32);
        assert_eq!(
            region.markets[1].market_type,
            crate::model::MarketType::Sell
        );
    }

    #[test]
    fn setup_pop_respects_neighbor_towns() {
        let ruleset = <dyn Ruleset>::from_name("standard").expect("standard ruleset");
        let tables = ruleset.tables();
        let mut defs = *ruleset.game_defs();
        defs.grow_races = 1;
        defs.towns_exist = 1;
        defs.town_probability = 1;
        defs.town_spread = 100;
        defs.towns_not_adjacent = 100;
        let mut game = Game::default();
        let mut neighbor = Region::default();
        neighbor.town = Some(TownInfo {
            name: Some("Neighbor".to_owned()),
            population: 1000,
            habitat: 1500,
            ..TownInfo::default()
        });
        let neighbor_id = game.add_region(neighbor);
        let mut region = Region::default();
        region.terrain_type = Some(R_PLAIN);
        region.race = ItemId::Plainsman.index() as i32;
        let region_id = game.add_region(region);
        game.region_mut(region_id).expect("region exists").neighbors[0] = Some(neighbor_id);
        let mut rng = Rng::new(424242);

        setup_pop(
            &mut game,
            region_id,
            &tables.terrains[R_PLAIN],
            &defs,
            &tables,
            false,
            &mut rng,
            &mut |_, _| "Generated town".to_owned(),
        )
        .expect("valid region and neighbor");

        let region = game.region(region_id).expect("region exists");
        assert!(region.population > 0);
        assert!(region.town.is_none());
    }

    #[test]
    fn lair_check_skips_towns_and_disabled_lairs() {
        let ruleset = <dyn Ruleset>::from_name("standard").expect("standard ruleset");
        let mut tables = ruleset.tables();
        let mut terrain = tables.terrains[R_PLAIN];
        terrain.lair_chance = 100;
        terrain.lairs = [0; 6];
        tables.objects[0].flags |= crate::defs::objects::object_flags::DISABLED;
        let mut game = Game::default();
        let mut region = Region::default();
        region.town = Some(TownInfo::default());
        let region_id = game.add_region(region);
        let mut rng = Rng::new(424242);
        let mut expected_rng = Rng::new(424242);

        lair_check(&mut game, region_id, &terrain, &tables, &mut rng)
            .expect("town regions skip lair checks");
        game.region_mut(region_id).expect("region exists").town = None;
        lair_check(&mut game, region_id, &terrain, &tables, &mut rng)
            .expect("disabled lairs are ignored");

        assert_eq!(rng.get_random(100), {
            let _ = expected_rng.get_random(100);
            expected_rng.get_random(100)
        });
        assert!(game
            .region(region_id)
            .expect("region exists")
            .object_ids()
            .is_empty());
    }

    #[test]
    fn manual_setup_adds_named_settlement_and_requested_lair() {
        let ruleset = <dyn Ruleset>::from_name("standard").expect("standard ruleset");
        let mut tables = ruleset.tables();
        let lair = tables.terrains[R_PLAIN]
            .lairs
            .iter()
            .copied()
            .find(|lair| {
                usize::try_from(*lair)
                    .ok()
                    .and_then(|index| tables.objects.get(index))
                    .is_some_and(|object| {
                        object.flags & crate::defs::objects::object_flags::DISABLED == 0
                    })
            })
            .expect("plain terrain has an enabled lair");
        tables.terrains[R_PLAIN].lairs = [lair; 6];

        let mut defs = *ruleset.game_defs();
        defs.grow_races = 1;
        defs.towns_exist = 1;
        defs.lair_monsters_exist = 1;
        defs.random_economy = 0;
        let mut game = Game::default();
        let mut region = Region::default();
        region.terrain_type = Some(R_PLAIN);
        region.race = ItemId::Plainsman.index() as i32;
        let region_id = game.add_region(region);
        let mut rng = Rng::new(424242);

        super::manual_setup(
            &mut game,
            region_id,
            R_PLAIN,
            1000,
            0.0,
            true,
            Some((TownType::City, "Reference City".to_owned())),
            &defs,
            &tables,
            false,
            &mut rng,
        )
        .expect("valid manual setup");

        let region = game.region(region_id).expect("region exists");
        let town = region.town.as_ref().expect("settlement was added");
        assert_eq!(town.name.as_deref(), Some("Reference City"));
        assert!(town.population > 0);
        assert!(region.population > 0);
        assert!(region.markets.len() > 2);
        assert_eq!(region.object_ids().len(), 2);
        let dummy = game.object(region.object_ids()[0]).expect("dummy object");
        assert_eq!(dummy.object_type, crate::defs::objects::O_DUMMY as i32);
        let lair_object = game.object(region.object_ids()[1]).expect("lair object");
        assert_eq!(lair_object.object_type, lair);
        assert_eq!(region.building_seq, 2);
    }

    #[test]
    fn starting_city_replaces_markets() {
        let ruleset = <dyn Ruleset>::from_name("standard").expect("standard ruleset");
        let tables = ruleset.tables();
        let mut defs = *ruleset.game_defs();
        defs.random_economy = 0;
        defs.city_market_advanced_amt = 0;
        defs.city_market_magic_amt = 0;
        defs.start_cities_exist = 1;
        defs.start_cities_start_unlimited = 1;
        defs.gates_exist = 1;
        let mut game = Game::default();
        let mut region = Region::default();
        region.terrain_type = Some(R_PLAIN);
        region.race = ItemId::Plainsman.index() as i32;
        region.population = 1000;
        region.habitat = 1000;
        region.development = 100;
        region.town = Some(TownInfo {
            name: Some("Old Town".to_owned()),
            population: 500,
            habitat: 750,
            development: 10,
            activity: 0,
        });
        region.markets.push(Market {
            market_type: MarketType::Buy,
            item: ItemId::Silver.index() as i32,
            price: 1,
            ..Market::default()
        });
        let region_id = game.add_region(region);
        let mut rng = Rng::new(424242);

        make_starting_city(
            &mut game,
            region_id,
            &defs,
            &tables,
            &mut rng,
            &mut |_, region| {
                assert_eq!(region.gate, -1);
                assert!(region.town.is_none());
                "Start".to_owned()
            },
        )
        .expect("valid starting city");

        let region = game.region(region_id).expect("region exists");
        assert!(region
            .markets
            .iter()
            .all(|market| market.min_population == 5000));
        assert_ne!(
            region.markets.first().map(|market| market.item),
            Some(ItemId::Silver.index() as i32)
        );
        let town = region.town.as_ref().expect("starting city has a town");
        assert_eq!(town.name.as_deref(), Some("Start"));
        assert_eq!(town.habitat, defs.city_pop * 125 / 100);
        assert!(town.population >= town.habitat);
    }
}
