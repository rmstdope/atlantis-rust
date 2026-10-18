use std::collections::BTreeSet;

use crate::api::{TurnError, TurnFile, TurnInput, TurnOutput};
use crate::defs::{
    enums::flags,
    skills::{lookup_skill, skill_flags},
    terrains::R_NEXUS,
    BattleItemFlags, ItemFlags, ItemId, ItemTypeFlags, RulesetTables, WeaponFlags,
};
use crate::model::{Faction, Game, Object, Region, Unit, UnitType};
use crate::orders::{
    parse_order_file, MoveDir, MoveOrder, OrderData, OrderParseMode, ProduceOrder,
};
use crate::players::{read_players, write_players};
use crate::rng::Rng;
use crate::ruleset::Ruleset;
use crate::save::save_legacy;

const FLAG_AUTOTAX: i32 = 0x0004;
const TAX_AUTO: i32 = 3;
const GAME_STATUS_RUNNING: i32 = 2;
const GAME_STATUS_FINISHED: i32 = 3;
const QUIT_BY_GM: i32 = 2;
const WEATHER_NORMAL: i32 = 0;
const WEATHER_WINTER: i32 = 1;
const WEATHER_MONSOON: i32 = 2;

/// C++: `game.cpp` `Game::RunGame` followed by `Game::SaveGame`; runs one turn in memory.
///
/// The engine output starts at "Setting Up Turn...": the startup banner and `OpenGame`'s lines
/// are printed by whoever loaded the save.
pub(crate) fn run_game(input: TurnInput) -> Result<TurnOutput, TurnError> {
    let mut loaded = input.loaded_game;
    let ruleset = loaded.ruleset;
    let defs = ruleset.game_defs();
    let tables = ruleset.tables();
    let mut output = crate::output::EngineOutput::new();

    output.write_line("Setting Up Turn...");
    pre_process_turn(&mut loaded.game, ruleset, &tables, &mut loaded.rng)?;

    output.write_line("Reading the Gamemaster File...");
    read_players(&mut loaded.game, &input.players_input, defs, &tables)?;
    if loaded.game.game_status == GAME_STATUS_FINISHED {
        output.write_line("This game is finished!");
        return Err(TurnError::GameFinished {
            engine_output: output.into_string(),
        });
    }
    loaded.game.game_status = GAME_STATUS_RUNNING;

    output.write_line("Reading the Orders File...");
    read_orders(&mut loaded.game, ruleset, &tables, &input.order_files)?;

    if defs.max_inactive_turns != -1 {
        output.write_line("QUITting Inactive Factions...");
        remove_inactive_factions(&mut loaded.game, ruleset);
    }

    output.write_line("Running the Turn...");
    run_orders(&mut loaded.game, defs, &mut output);

    let mut files = Vec::new();
    if defs.world_events != 0 {
        output.write_line("Writing world events...");
        assert!(
            loaded.game.events.is_empty(),
            "the unported world-event phase has generated events"
        );
    }

    output.write_line("Writing the Report File...");
    for faction_id in loaded.game.faction_ids().to_vec() {
        let faction = loaded
            .game
            .faction(faction_id)
            .expect("ordered faction exists");
        if should_write_report(faction, &loaded.game, defs.gm_report != 0) {
            files.push(TurnFile {
                name: format!("report.{}", faction.num),
                contents: String::new(),
            });
        }
        output.write_dot();
    }
    output.write_line("");

    output.write_line("Writing order templates...");
    for faction_id in loaded.game.faction_ids().to_vec() {
        let faction = loaded
            .game
            .faction(faction_id)
            .expect("ordered faction exists");
        if !is_npc(faction) {
            files.push(TurnFile {
                name: format!("template.{}", faction.num),
                contents: String::new(),
            });
        }
        output.write_dot();
    }
    output.write_line("");

    loaded.game.empty_hell();
    output.write_line("Writing Playerinfo File...");
    files.push(TurnFile {
        name: "players.out".to_owned(),
        contents: write_players(&loaded.game)?,
    });

    output.write_line("Removing Dead Factions...");
    delete_dead_factions(&mut loaded.game);
    output.write_line("done");

    files.push(TurnFile {
        name: "game.out".to_owned(),
        contents: save_legacy(&mut loaded)?,
    });
    Ok(TurnOutput {
        files,
        engine_output: output.into_string(),
    })
}

/// C++: `game.cpp` `PreProcessTurn`.
pub fn pre_process_turn(
    game: &mut Game,
    ruleset: &dyn Ruleset,
    tables: &RulesetTables,
    rng: &mut Rng,
) -> Result<(), crate::model::ModelError> {
    game.month += 1;
    if game.month > 11 {
        game.month = 0;
        game.year += 1;
    }
    let month = game.month;
    game.setup_unit_nums()?;

    let defs = ruleset.game_defs();
    clear_faction_default_orders(game);
    for region_id in game.region_ids().to_vec() {
        let region = game.region(region_id).expect("ordered region exists");
        let weather = (defs.weather_exists != 0).then(|| weather_for_region(game, region, month));
        let region = game.region_mut(region_id).expect("ordered region exists");
        if let Some(weather) = weather {
            region.weather = weather;
        }
        if defs.gates_not_perennial != 0 {
            update_gate_status(region, defs, month);
        }
        region_default_orders(game, region_id, ruleset, tables, rng);
    }
    Ok(())
}

fn weather_for_region(game: &Game, region: &Region, month: i32) -> i32 {
    if region.zloc == 0 || region.zloc > 1 {
        return WEATHER_NORMAL;
    }

    let surface_height = game
        .region_arrays
        .get(1)
        .expect("surface region array exists when weather is enabled")
        .y;
    let latitude = (3 * (region.yloc + 1)) / surface_height;
    if latitude == 0 {
        if !(2..=9).contains(&month) {
            WEATHER_WINTER
        } else {
            WEATHER_NORMAL
        }
    } else if latitude == 1 {
        if month == 11 || month == 0 || month == 5 || month == 6 {
            WEATHER_MONSOON
        } else {
            WEATHER_NORMAL
        }
    } else if (4..=7).contains(&month) {
        WEATHER_WINTER
    } else {
        WEATHER_NORMAL
    }
}

fn update_gate_status(region: &mut Region, defs: &crate::defs::GameDefs, month: i32) {
    if region.terrain_type == Some(R_NEXUS)
        || (defs.start_gates_open != 0
            && region
                .town
                .as_ref()
                .is_some_and(|town| town.population >= defs.city_pop * 120 / 100))
    {
        region.gate_open = 1;
        return;
    }

    region.gate_open = 0;
    for offset in 0..defs.gates_not_perennial {
        let mut gate_month = region.gate_month + offset;
        if gate_month > 11 {
            gate_month -= 12;
        }
        if gate_month == month {
            region.gate_open = 1;
        }
    }
}

fn read_orders(
    game: &mut Game,
    ruleset: &dyn Ruleset,
    tables: &RulesetTables,
    order_files: &std::collections::BTreeMap<i32, Vec<u8>>,
) -> Result<(), TurnError> {
    for faction_id in game.faction_ids().to_vec() {
        let faction = game.faction(faction_id).expect("ordered faction exists");
        if is_npc(faction) {
            continue;
        }
        if let Some(orders) = order_files.get(&faction.num) {
            parse_order_file(
                game,
                ruleset.game_defs(),
                tables,
                orders,
                OrderParseMode::Turn,
            )?;
        }
        apply_default_work_orders(game, ruleset, tables);
    }
    Ok(())
}

/// C++: `game.cpp` `RemoveInactiveFactions`.
fn remove_inactive_factions(game: &mut Game, ruleset: &dyn Ruleset) {
    let max_inactive_turns = ruleset.game_defs().max_inactive_turns;
    let turn_number = game.turn_number();
    for faction_id in game.faction_ids().to_vec() {
        let faction = game
            .faction_mut(faction_id)
            .expect("ordered faction exists");
        if !is_npc(faction) && turn_number - faction.last_orders >= max_inactive_turns {
            faction.quit = QUIT_BY_GM;
        }
    }
}

fn should_write_report(faction: &Faction, game: &Game, gm_report: bool) -> bool {
    !is_npc(faction) || (((game.month == 0 && game.year == 1) || gm_report) && faction.num == 1)
}

fn delete_dead_factions(game: &mut Game) {
    let dead = game
        .faction_ids()
        .iter()
        .copied()
        .filter(|faction_id| {
            game.faction(*faction_id)
                .is_some_and(|faction| faction.exists == 0 && !is_npc(faction))
        })
        .collect::<Vec<_>>();
    for faction_id in dead {
        game.remove_faction(faction_id);
    }
}

/// C++: `runorders.cpp` `Game::RunOrders`; the phase list in the original's order.
///
/// Only `FindDeadFactions` and the no-op `ProcessMigration` are ported. Every other phase is a
/// guard that panics when the world gives it anything to act on, so a turn can never complete
/// with work silently skipped. The guards are deliberately coarse (any unit, any region, any
/// non-dummy object): each phase walks every unit or region and acts on more than its order
/// slots (default guards, invisibility flags, maintenance bookkeeping, region growth), so a
/// finer predicate would risk passing a turn whose C++ run changes state or draws.
fn run_orders(
    game: &mut Game,
    defs: &crate::defs::GameDefs,
    output: &mut crate::output::EngineOutput,
) {
    use Walks::{Factions, NoLivingFaction, Objects, Regions, Units};

    output.write_line("Running FIND Orders...");
    unported(game, "RunFindOrders", Units);
    output.write_line("Running ENTER/LEAVE Orders...");
    unported(game, "RunEnterOrders(0)", Units);
    output.write_line("Running PROMOTE/EVICT Orders...");
    unported(game, "RunPromoteOrders", Units);
    output.write_line("Running Combat...");
    unported(game, "DoAttackOrders", Units);
    unported(game, "DoAutoAttacks", Units);
    output.write_line("Running STEAL/ASSASSINATE Orders...");
    unported(game, "RunStealOrders", Units);
    output.write_line("Running GIVE Orders...");
    unported(game, "DoGiveOrders", Units);
    output.write_line("Running ENTER NEW Orders...");
    unported(game, "RunEnterOrders(1)", Units);
    output.write_line("Running EXCHANGE Orders...");
    unported(game, "DoExchangeOrders", Units);
    output.write_line("Running DESTROY Orders...");
    unported(game, "RunDestroyOrders", Units);
    output.write_line("Running PILLAGE Orders...");
    unported(game, "RunPillageOrders", Units);
    output.write_line("Running TAX Orders...");
    unported(game, "RunTaxOrders", Units);
    output.write_line("Running GUARD 1 Orders...");
    unported(game, "DoGuard1Orders", Units);
    output.write_line("Running Magic Orders...");
    unported(game, "ClearCastEffects", Units);
    unported(game, "RunCastOrders", Units);
    output.write_line("Running SELL Orders...");
    unported(game, "RunSellOrders", Units);
    output.write_line("Running BUY Orders...");
    unported(game, "RunBuyOrders", Units);
    output.write_line("Running FORGET Orders...");
    unported(game, "RunForgetOrders", Units);
    output.write_line("Mid-Turn Processing...");
    unported(game, "MidProcessTurn", Units);
    output.write_line("Running QUIT Orders...");
    unported(game, "RunQuitOrders", Factions(|faction| faction.quit != 0));
    output.write_line("Removing Empty Units...");
    unported(game, "DeleteEmptyUnits", Units);
    if defs.allow_withdraw != 0 {
        output.write_line("Running WITHDRAW Orders...");
        unported(game, "DoWithdrawOrders", Units);
    }
    output.write_line("Running Consolidated Movement Orders...");
    unported(game, "RunMovementOrders", Units);
    unported(game, "SinkUncrewedFleets", Objects);
    unported(game, "DrownUnits", Units);
    find_dead_factions(game);
    output.write_line("Running Teach Orders...");
    unported(game, "RunTeachOrders", Units);
    output.write_line("Running Month-long Orders...");
    unported(game, "RunMonthOrders", Units);
    output.write_line("Running Economics...");
    if defs.dynamic_population != 0 || defs.regions_economy != 0 {
        unported(game, "ProcessEconomics", Regions);
    }
    output.write_line("Running Teleport Orders...");
    unported(game, "RunTeleportOrders", Units);
    if defs.transport & flags::ALLOW_TRANSPORT != 0 {
        output.write_line("Running Transport Orders...");
        unported(game, "CheckTransportOrders", Units);
        unported(game, "RunTransportOrders", Units);
    }
    output.write_line("Assessing Maintenance costs...");
    unported(game, "AssessMaintenance", Units);
    if defs.dynamic_population != 0 {
        output.write_line("Processing Migration...");
        // C++: `ProcessMigration` begins with `return;`, so there is nothing to port.
    }
    output.write_line("Post-Turn Processing...");
    // C++: `PostProcessTurn` ends the game when no faction exists, then walks every region
    // (`PostTurn`, `AdjustCityMons`) and grows monsters across the region levels.
    unported(game, "PostProcessTurn", NoLivingFaction);
    unported(game, "PostProcessTurn", Regions);
    unported(game, "DeleteEmptyUnits", Units);
    unported(game, "RemoveEmptyObjects", Objects);
}

/// What an unported phase walks, and so what makes it act.
enum Walks {
    /// Every unit in every region: any unit at all.
    Units,
    /// Every region, or the region levels: any region or region level at all.
    Regions,
    /// Objects other than a region's dummy: any building or fleet.
    Objects,
    /// Any faction for which the predicate holds.
    Factions(fn(&Faction) -> bool),
    /// No faction exists any more (C++ `EndGame(0)`).
    NoLivingFaction,
}

fn unported(game: &Game, phase: &str, walks: Walks) {
    let units = || {
        game.region_ids()
            .iter()
            .filter_map(|id| game.region(*id))
            .flat_map(|region| region.object_ids())
            .filter_map(|id| game.object(*id))
            .flat_map(|object| object.unit_ids())
            .filter_map(|id| game.unit(*id))
    };
    let factions = || game.faction_ids().iter().filter_map(|id| game.faction(*id));
    let blocker = match walks {
        Walks::Units => units().next().map(|unit| format!("unit {}", unit.num)),
        Walks::Regions => (!game.region_ids().is_empty() || !game.region_arrays.is_empty())
            .then(|| "a region".to_owned()),
        Walks::Objects => game
            .region_ids()
            .iter()
            .filter_map(|id| game.region(*id))
            .flat_map(|region| region.object_ids())
            .filter_map(|id| game.object(*id))
            .find(|object| object.object_type != crate::defs::objects::O_DUMMY as i32)
            .map(|object| format!("object {}", object.num)),
        Walks::Factions(predicate) => factions()
            .find(|faction| predicate(faction))
            .map(|faction| format!("faction {}", faction.num)),
        Walks::NoLivingFaction => factions()
            .all(|faction| faction.exists == 0)
            .then(|| "no living faction".to_owned()),
    };
    if let Some(blocker) = blocker {
        panic!("unported RunOrders phase {phase} has work: {blocker}");
    }
}

/// C++: `runorders.cpp` `Game::FindDeadFactions`; `Faction::CheckExist` for every faction.
/// NPC factions are left alone; any other faction exists only while one of its units stands in
/// a region (`ARegion::Present`).
fn find_dead_factions(game: &mut Game) {
    for faction_id in game.faction_ids().to_vec() {
        let faction = game.faction(faction_id).expect("ordered faction exists");
        if is_npc(faction) {
            continue;
        }
        let present = game.region_ids().iter().any(|region_id| {
            game.region(*region_id)
                .expect("ordered region exists")
                .object_ids()
                .iter()
                .filter_map(|object_id| game.object(*object_id))
                .flat_map(|object| object.unit_ids())
                .filter_map(|unit_id| game.unit(*unit_id))
                .any(|unit| unit.faction == Some(faction_id))
        });
        game.faction_mut(faction_id)
            .expect("ordered faction exists")
            .exists = i8::from(present);
    }
}

/// C++: `game.cpp` `PreProcessTurn` faction and region default-order fan-out.
pub fn pre_process_default_orders(
    game: &mut Game,
    ruleset: &dyn Ruleset,
    tables: &RulesetTables,
    rng: &mut Rng,
) {
    clear_faction_default_orders(game);
    for region_id in game.region_ids().to_vec() {
        region_default_orders(game, region_id, ruleset, tables, rng);
    }
}

/// C++: `faction.cpp` `Faction::DefaultOrders`, for every faction in order.
fn clear_faction_default_orders(game: &mut Game) {
    for faction_id in game.faction_ids().to_vec() {
        let faction = game
            .faction_mut(faction_id)
            .expect("ordered faction exists");
        faction.activity.clear();
        faction.num_shows = 0;
    }
}

/// C++: `aregion.cpp` `ARegion::DefaultOrders`, calling `Unit::DefaultOrders` for each unit.
fn region_default_orders(
    game: &mut Game,
    region_id: crate::model::RegionId,
    ruleset: &dyn Ruleset,
    tables: &RulesetTables,
    rng: &mut Rng,
) {
    let region = game
        .region(region_id)
        .cloned()
        .expect("ordered region exists");
    for object_id in region.object_ids().to_vec() {
        let object = game
            .object(object_id)
            .cloned()
            .expect("region object exists");
        for unit_id in object.unit_ids().to_vec() {
            let mut unit = game.unit(unit_id).cloned().expect("object unit exists");
            default_unit_orders(&mut unit, &object, &region, game, ruleset, tables, rng);
            *game.unit_mut(unit_id).expect("object unit exists") = unit;
        }
    }
}

/// C++: `game.cpp` `DefaultWorkOrder`, called after each non-NPC faction's orders are read.
pub fn apply_default_work_orders(game: &mut Game, ruleset: &dyn Ruleset, tables: &RulesetTables) {
    let defs = ruleset.game_defs();
    for region_id in game.region_ids().to_vec() {
        let region = game
            .region(region_id)
            .cloned()
            .expect("ordered region exists");
        if region.terrain_type == Some(R_NEXUS) {
            continue;
        }
        for object_id in region.object_ids().to_vec() {
            let object = game
                .object(object_id)
                .cloned()
                .expect("region object exists");
            for unit_id in object.unit_ids().to_vec() {
                let mut unit = game.unit(unit_id).cloned().expect("object unit exists");
                if unit.orders.month_orders.is_some()
                    || is_npc(
                        game.faction(
                            unit.faction
                                .expect("unit has a faction before default work orders"),
                        )
                        .expect("unit faction exists"),
                    )
                    || (defs.tax_pillage_month_long != 0 && unit.taxing != 0)
                {
                    continue;
                }
                if unit.flags & FLAG_AUTOTAX != 0
                    && defs.tax_pillage_month_long != 0
                    && taxers(&unit, ruleset, tables) > 0
                {
                    unit.taxing = TAX_AUTO;
                } else if defs.default_work_order != 0 {
                    set_default_work_order(&mut unit, defs.tax_pillage_month_long != 0);
                }
                *game.unit_mut(unit_id).expect("object unit exists") = unit;
            }
        }
    }
}

fn default_unit_orders(
    unit: &mut Unit,
    object: &Object,
    region: &Region,
    game: &Game,
    ruleset: &dyn Ruleset,
    tables: &RulesetTables,
    rng: &mut Rng,
) {
    unit.clear_orders();
    match unit.unit_type {
        UnitType::Guard => {
            if unit.guard != crate::model::GuardMode::Guard {
                unit.guard = crate::model::GuardMode::Set;
            }
        }
        UnitType::GuardMage => {
            unit.combat =
                lookup_skill(&tables.skills.skills, "FIRE").expect("Fire skill is defined") as i32;
        }
        UnitType::WanderingMonster => {
            wandering_monster_orders(unit, object, region, game, ruleset, tables, rng);
        }
        _ if region.terrain_type != Some(R_NEXUS) => {
            if unit.flags & FLAG_AUTOTAX != 0
                && ruleset.game_defs().tax_pillage_month_long != 0
                && taxers(unit, ruleset, tables) > 0
            {
                unit.taxing = TAX_AUTO;
            } else {
                set_default_work_order(unit, ruleset.game_defs().tax_pillage_month_long != 0);
            }
        }
        _ => {}
    }
}

fn set_default_work_order(unit: &mut Unit, tax_pillage_month_long: bool) {
    let mut order = ProduceOrder {
        item: ItemId::Silver.index() as i32,
        skill: -1,
        ..ProduceOrder::default()
    };
    order.order.quiet = 1;
    unit.orders.month_orders = Some(OrderData::Produce(order));
    if tax_pillage_month_long {
        unit.taxing = 0;
    }
}

fn is_npc(faction: &Faction) -> bool {
    faction.faction_types.iter().any(|(name, value)| {
        *value == -1
            && (crate::text::is_equal(name, "WAR") || crate::text::is_equal(name, "MARTIAL"))
    })
}

/// C++: `unit.cpp` `Unit::Taxers`; preserves its `numMelee` count under `TAX_USABLE_WEAPON`.
fn taxers(unit: &Unit, ruleset: &dyn Ruleset, tables: &RulesetTables) -> i32 {
    let tax_flags = ruleset.game_defs().who_can_tax;
    let men = unit_men(unit, tables);
    let any_skill = (tax_flags & flags::TAX_COMBAT_SKILL != 0
        && unit_skill_level(unit, "COMB", tables) > 0)
        || (tax_flags & flags::TAX_BOW_SKILL != 0
            && (unit_skill_level(unit, "XBOW", tables) > 0
                || unit_skill_level(unit, "LBOW", tables) > 0))
        || (tax_flags & flags::TAX_RIDING_SKILL != 0 && unit_skill_level(unit, "RIDE", tables) > 0)
        || (tax_flags & flags::TAX_STEALTH_SKILL != 0
            && unit_skill_level(unit, "STEA", tables) > 0);
    if men > 0 && (tax_flags & flags::TAX_ANYONE != 0 || any_skill) {
        return men;
    }

    let mut melee = 0;
    let mut usable_melee = 0;
    let mut bows = 0;
    let mut usable_bows = 0;
    let mut mounted = 0;
    let mut usable_mounted = 0;
    let mut mounts = 0;
    let mut usable_mounts = 0;
    let mut battle_items = 0;
    let mut usable_battle_items = 0;
    let mut creatures = 0;
    let mut illusions = 0;

    for item in unit.items.entries() {
        let definition = &tables.item_families.items[item.item_type.index()];
        if definition.item_type & ItemTypeFlags::BATTLE != 0 {
            let battle_item = tables.item_families.battle_items.iter().find(|battle| {
                battle.abbreviation.as_deref().is_some_and(|abbreviation| {
                    definition
                        .abbreviation
                        .as_deref()
                        .is_some_and(|item_abbreviation| {
                            crate::text::is_equal(abbreviation, item_abbreviation)
                        })
                })
            });
            if battle_item.is_some_and(|battle| battle.flags & BattleItemFlags::SPECIAL != 0) {
                let usable = tax_flags & flags::TAX_USABLE_BATTLE_ITEM != 0
                    && (battle_item.expect("special battle item").flags
                        & BattleItemFlags::MAGEONLY
                        == 0
                        || matches!(unit.unit_type, UnitType::Mage | UnitType::Apprentice));
                if usable {
                    usable_battle_items += item.num;
                    battle_items += item.num;
                    continue;
                }
                if tax_flags & flags::TAX_BATTLE_ITEM != 0 {
                    battle_items += item.num;
                    continue;
                }
            }
        }

        if definition.item_type & ItemTypeFlags::WEAPON != 0 {
            let abbreviation = definition
                .abbreviation
                .as_deref()
                .expect("weapon item has an abbreviation");
            let weapon = tables
                .item_families
                .find_weapon(abbreviation)
                .expect("weapon item has a weapon definition");
            let base_skill = weapon
                .base_skill
                .as_deref()
                .is_some_and(|skill| unit_skill_level(unit, skill, tables) > 0)
                || weapon
                    .or_skill
                    .as_deref()
                    .is_some_and(|skill| unit_skill_level(unit, skill, tables) > 0);
            if weapon.flags & WeaponFlags::NEEDSKILL == 0 {
                if base_skill {
                    usable_melee += item.num;
                }
                melee += item.num;
            } else if weapon.flags & WeaponFlags::NOFOOT != 0 {
                if base_skill {
                    usable_mounted += item.num;
                }
                mounted += item.num;
            } else if weapon.flags & WeaponFlags::RANGED != 0 {
                if base_skill {
                    usable_bows += item.num;
                }
                bows += item.num;
            } else {
                if base_skill {
                    usable_melee += item.num;
                }
                melee += item.num;
            }
        }

        if definition.item_type & ItemTypeFlags::MOUNT != 0 {
            let abbreviation = definition
                .abbreviation
                .as_deref()
                .expect("mount item has an abbreviation");
            let mount = tables
                .item_families
                .find_mount(abbreviation)
                .expect("mount item has a mount definition");
            let usable = mount
                .skill
                .as_deref()
                .is_none_or(|skill| unit_skill_level(unit, skill, tables) >= mount.minimum_bonus);
            if usable {
                usable_mounts += item.num;
            }
            mounts += item.num;
        }

        if definition.item_type & ItemTypeFlags::MONSTER != 0 {
            if definition.item_type & ItemTypeFlags::ILLUSION != 0 {
                illusions += item.num;
            } else {
                creatures += item.num;
            }
        }
    }

    let mut count = if tax_flags & flags::TAX_USABLE_WEAPON != 0 {
        usable_mounted.min(usable_mounts) + melee + usable_bows
    } else if tax_flags & flags::TAX_ANY_WEAPON != 0 {
        melee + bows + mounted
    } else {
        let mut count = 0;
        if tax_flags & flags::TAX_MELEE_WEAPON_AND_MATCHING_SKILL != 0 {
            count += usable_mounted.min(usable_mounts) + usable_melee;
        }
        if tax_flags & flags::TAX_BOW_SKILL_AND_MATCHING_WEAPON != 0 {
            count += usable_bows;
        }
        count
    };
    if tax_flags & flags::TAX_HORSE != 0 {
        count += mounts;
    } else if tax_flags & flags::TAX_HORSE_AND_RIDING_SKILL != 0 {
        count += usable_mounts;
    }
    if tax_flags & flags::TAX_BATTLE_ITEM != 0 {
        count += battle_items;
    } else if tax_flags & flags::TAX_USABLE_BATTLE_ITEM != 0 {
        count += usable_battle_items;
    }
    if unit.unit_type == UnitType::Mage {
        count += mage_taxers(unit, men, tax_flags, tables);
    }
    count = count.min(men);
    if tax_flags & flags::TAX_CREATURES != 0 {
        count += creatures;
    }
    if tax_flags & flags::TAX_ILLUSIONS != 0 {
        count += illusions;
    }
    count
}

fn mage_taxers(unit: &Unit, men: i32, tax_flags: i32, tables: &RulesetTables) -> i32 {
    if tax_flags & flags::TAX_ANY_MAGE != 0 {
        return men;
    }
    let tax_spell = |skill_id: usize| {
        let Some(skill) = tables.skills.skills.get(skill_id) else {
            return false;
        };
        (tax_flags & flags::TAX_MAGE_DAMAGE != 0 && skill.flags & skill_flags::DAMAGE != 0)
            || (tax_flags & flags::TAX_MAGE_FEAR != 0 && skill.flags & skill_flags::FEAR != 0)
            || (tax_flags & flags::TAX_MAGE_OTHER != 0
                && skill.flags & skill_flags::MAGE_OTHER != 0)
    };
    if tax_flags & flags::TAX_MAGE_COMBAT_SPELL != 0 {
        return if usize::try_from(unit.combat).ok().is_some_and(tax_spell) {
            men
        } else {
            0
        };
    }
    if unit
        .skills
        .entries()
        .iter()
        .any(|skill| tax_spell(skill.skill_type))
    {
        men
    } else {
        0
    }
}

fn unit_skill_level(unit: &Unit, abbreviation: &str, tables: &RulesetTables) -> i32 {
    lookup_skill(&tables.skills.skills, abbreviation)
        .map(|skill| available_skill_level(unit, skill, tables))
        .unwrap_or(0)
}

fn available_skill_level(unit: &Unit, skill_id: usize, tables: &RulesetTables) -> i32 {
    let mut level = real_skill_level(unit, skill_id, tables);
    let men = unit_men(unit, tables);
    let is_mage = matches!(
        unit.unit_type,
        UnitType::Mage | UnitType::Apprentice | UnitType::GuardMage
    );
    let skill_is_magic = tables
        .skills
        .skills
        .get(skill_id)
        .is_some_and(|skill| skill.flags & skill_flags::MAGIC != 0);

    for item in unit.items.entries() {
        let definition = &tables.item_families.items[item.item_type.index()];
        if definition.flags & ItemFlags::DISABLED != 0
            || (definition.item_type & ItemTypeFlags::MAGE_ONLY != 0 && !is_mage)
            || (skill_is_magic && !is_mage)
            || item.num < men
            || definition
                .grant_skill
                .as_deref()
                .and_then(|grant| lookup_skill(&tables.skills.skills, grant))
                != Some(skill_id)
        {
            continue;
        }
        let mut grant = definition
            .grant_from
            .iter()
            .flatten()
            .filter_map(|abbreviation| lookup_skill(&tables.skills.skills, abbreviation))
            .map(|from_skill| real_skill_level(unit, from_skill, tables))
            .max()
            .unwrap_or(0);
        grant = grant.max(definition.min_grant).min(definition.max_grant);
        level = level.max(grant);
    }
    level
}

fn real_skill_level(unit: &Unit, skill_id: usize, tables: &RulesetTables) -> i32 {
    let men = unit_men(unit, tables);
    if men <= 0 {
        return 0;
    }
    let days_per_man = unit.skills.get_days(skill_id) / men as u32;
    let mut required = 30;
    let mut level = 0;
    while days_per_man >= required {
        level += 1;
        required += 30;
    }
    level
}

fn wandering_monster_orders(
    unit: &mut Unit,
    object: &Object,
    region: &Region,
    game: &Game,
    ruleset: &dyn Ruleset,
    tables: &RulesetTables,
    rng: &mut Rng,
) {
    let object_type = usize::try_from(object.object_type).expect("object type is nonnegative");
    let object_def = tables
        .objects
        .get(object_type)
        .expect("object type is defined");
    if object_def.monster != -1 {
        return;
    }

    let defs = ruleset.game_defs();
    let mut aggression = 0;
    let mut preferred_terrain = BTreeSet::new();
    let mut forbidden_terrain = BTreeSet::new();
    for item in unit.items.entries() {
        let item_def = &tables.item_families.items[item.item_type.index()];
        if item_def.item_type & ItemTypeFlags::MONSTER == 0 {
            continue;
        }
        let abbreviation = item_def
            .abbreviation
            .as_deref()
            .expect("monster item has an abbreviation");
        let monster = tables
            .item_families
            .find_monster(
                abbreviation,
                item_def.item_type & ItemTypeFlags::ILLUSION != 0,
            )
            .expect("monster item has a monster definition");
        let hostility = monster.hostile * defs.monster_advance_hostile_percent / 100;
        aggression = aggression.max(hostility.max(defs.monster_advance_min_percent));
        preferred_terrain.extend(monster.preferred_terrain.iter().copied());
        forbidden_terrain.extend(monster.forbidden_terrain.iter().copied());
    }

    let directions = wandering_direction_candidates(
        unit,
        region,
        game,
        ruleset,
        tables,
        &preferred_terrain,
        &forbidden_terrain,
    );
    let direction_index = rng.get_random(directions.len() as i32) as usize;
    let direction = directions[direction_index];
    if direction >= 0 {
        let mut order = MoveOrder {
            advancing: i32::from(rng.get_random(100) < aggression),
            ..MoveOrder::default()
        };
        order.dirs.push(MoveDir { dir: direction });
        unit.orders.month_orders = Some(OrderData::Move(order));
    }
}

fn wandering_direction_candidates(
    unit: &Unit,
    region: &Region,
    game: &Game,
    ruleset: &dyn Ruleset,
    tables: &RulesetTables,
    preferred_terrain: &BTreeSet<i32>,
    forbidden_terrain: &BTreeSet<i32>,
) -> Vec<i32> {
    let defs = ruleset.game_defs();
    let weight = item_weight(unit, tables);
    let can_fly = flying_capacity(unit, tables) >= weight;
    let can_really_swim = is_alive(unit, tables) && swimming_capacity(unit, tables) >= weight;
    let can_walk = walking_capacity(unit, tables) >= weight;
    let can_ride = riding_capacity(unit, tables) >= weight;
    let mut directions = vec![-1; 4];
    for (direction, neighbor_id) in region.neighbors.iter().enumerate() {
        let Some(neighbor_id) = neighbor_id else {
            continue;
        };
        let neighbor = game.region(*neighbor_id).expect("neighbor region exists");
        let terrain_id = neighbor.terrain_type.expect("neighbor has a terrain");
        let similar_type = tables.terrains[terrain_id].similar_type;
        let is_ocean = similar_type == crate::defs::terrains::R_OCEAN as i32;

        if is_ocean
            && !can_really_swim
            && !(can_fly && defs.flight_over_water == flags::WFLIGHT_UNLIMITED)
        {
            continue;
        }
        if !is_ocean && !can_walk && !can_ride && !can_fly {
            continue;
        }
        if forbidden_terrain.contains(&similar_type) {
            continue;
        }
        if preferred_terrain.is_empty() || preferred_terrain.contains(&similar_type) {
            directions.extend([direction as i32; 2]);
            continue;
        }

        let connected_to_preferred = neighbor.neighbors.iter().flatten().any(|next_id| {
            game.region(*next_id)
                .and_then(|next| next.terrain_type)
                .is_some_and(|terrain| {
                    preferred_terrain.contains(&tables.terrains[terrain].similar_type)
                })
        });
        if connected_to_preferred {
            directions.push(direction as i32);
        }
    }
    directions
}

fn item_weight(unit: &Unit, tables: &RulesetTables) -> i32 {
    unit.items
        .entries()
        .iter()
        .map(|item| tables.item_families.items[item.item_type.index()].weight * item.num)
        .sum()
}

fn is_alive(unit: &Unit, tables: &RulesetTables) -> bool {
    match unit.unit_type {
        UnitType::Mage | UnitType::Apprentice => unit_men(unit, tables) > 0,
        _ => unit.items.entries().iter().any(|item| {
            let definition = &tables.item_families.items[item.item_type.index()];
            definition.item_type & (ItemTypeFlags::MAN | ItemTypeFlags::MONSTER) != 0
                && item.num > 0
        }),
    }
}

fn unit_men(unit: &Unit, tables: &RulesetTables) -> i32 {
    unit.items
        .entries()
        .iter()
        .filter(|item| {
            tables.item_families.items[item.item_type.index()].item_type & ItemTypeFlags::MAN != 0
        })
        .map(|item| item.num)
        .sum()
}

fn flying_capacity(unit: &Unit, tables: &RulesetTables) -> i32 {
    unit.items
        .entries()
        .iter()
        .filter_map(|item| {
            let definition = &tables.item_families.items[item.item_type.index()];
            (definition.item_type & ItemTypeFlags::SHIP == 0).then_some(definition.fly * item.num)
        })
        .sum()
}

fn swimming_capacity(unit: &Unit, tables: &RulesetTables) -> i32 {
    unit.items
        .entries()
        .iter()
        .filter_map(|item| {
            let definition = &tables.item_families.items[item.item_type.index()];
            (definition.item_type & ItemTypeFlags::SHIP == 0).then_some(definition.swim * item.num)
        })
        .sum()
}

fn walking_capacity(unit: &Unit, tables: &RulesetTables) -> i32 {
    unit.items
        .entries()
        .iter()
        .map(|item| {
            let definition = &tables.item_families.items[item.item_type.index()];
            let hitch_walk = usize::try_from(definition.hitch_item)
                .ok()
                .and_then(|hitch| tables.item_families.items.get(hitch))
                .filter(|hitch| hitch.flags & ItemFlags::DISABLED == 0)
                .map(|_| {
                    let hitches = unit
                        .items
                        .entries()
                        .iter()
                        .find(|candidate| {
                            candidate.item_type.index() as i32 == definition.hitch_item
                        })
                        .map_or(0, |hitch| hitch.num);
                    item.num.min(hitches) * definition.hitch_walk
                })
                .unwrap_or(0);
            definition.walk * item.num + hitch_walk
        })
        .sum()
}

fn riding_capacity(unit: &Unit, tables: &RulesetTables) -> i32 {
    unit.items
        .entries()
        .iter()
        .map(|item| tables.item_families.items[item.item_type.index()].ride * item.num)
        .sum()
}

#[cfg(test)]
mod tests {
    use super::{apply_default_work_orders, pre_process_default_orders, update_gate_status};
    use crate::defs::{
        skills::lookup_skill,
        terrains::{R_NEXUS, R_PLAIN},
        ItemId, ItemTypeFlags, WeaponFlags,
    };
    use crate::model::{
        Faction, FactionActivity, FactionActivityRecord, Game, GuardMode, Item, Object, Region,
        Skill, Unit, UnitType,
    };
    use crate::orders::{AttackOrder, IdleOrder, OrderData};
    use crate::rng::Rng;
    use crate::ruleset::{FactionLimits, Ruleset};

    struct NoDefaultWorkRuleset;

    static NO_DEFAULT_WORK_DEFS: std::sync::OnceLock<crate::defs::GameDefs> =
        std::sync::OnceLock::new();

    impl Ruleset for NoDefaultWorkRuleset {
        fn game_defs(&self) -> &'static crate::defs::GameDefs {
            NO_DEFAULT_WORK_DEFS.get_or_init(|| {
                let mut defs = *<dyn Ruleset>::from_name("neworigins")
                    .expect("NewOrigins ruleset")
                    .game_defs();
                defs.default_work_order = 0;
                defs
            })
        }

        fn faction_limits(&self) -> &'static FactionLimits {
            <dyn Ruleset>::from_name("neworigins")
                .expect("NewOrigins ruleset")
                .faction_limits()
        }

        fn faction_types(&self) -> &'static [&'static str] {
            <dyn Ruleset>::from_name("neworigins")
                .expect("NewOrigins ruleset")
                .faction_types()
        }
    }

    #[test]
    fn pre_process_default_orders_resets_factions_and_assigns_unit_type_defaults() {
        let ruleset = <dyn Ruleset>::from_name("neworigins").expect("NewOrigins ruleset");
        let tables = ruleset.tables();
        let mut game = Game::default();
        let mut region = Region::default();
        region.terrain_type = Some(R_PLAIN);
        let region_id = game.add_region(region);
        let faction_id = game.add_faction(Faction::new(3, &["WAR", "TRADE", "MAGIC"]));
        let faction = game.faction_mut(faction_id).expect("faction");
        faction.activity.push(FactionActivityRecord {
            region: region_id,
            activities: vec![FactionActivity::Tax],
        });
        faction.num_shows = 4;

        let object_id = game
            .add_object(region_id, Object::default())
            .expect("object");
        let mut worker = Unit::new(27, Some(faction_id), 0, 0);
        worker.orders.attack_orders = Some(AttackOrder::default());
        let worker_id = game.add_unit(object_id, worker).expect("unit");
        let mut guard = Unit::new(28, Some(faction_id), 0, 0);
        guard.unit_type = UnitType::Guard;
        let guard_id = game.add_unit(object_id, guard).expect("guard");
        let mut guard_mage = Unit::new(29, Some(faction_id), 0, 0);
        guard_mage.unit_type = UnitType::GuardMage;
        let guard_mage_id = game.add_unit(object_id, guard_mage).expect("guard mage");
        let combat = lookup_skill(&tables.skills.skills, "COMB").expect("combat skill");
        let mut auto_taxer = Unit::new(33, Some(faction_id), 0, 0);
        auto_taxer.flags |= 0x0004;
        auto_taxer.items.push(Item {
            item_type: ItemId::Leaders,
            num: 1,
            selling: 0,
            checked: false,
        });
        auto_taxer.skills.push(Skill {
            skill_type: combat,
            days: 30,
            exp: 0,
        });
        let auto_taxer_id = game.add_unit(object_id, auto_taxer).expect("auto-tax unit");

        let mut nexus = Region::default();
        nexus.terrain_type = Some(R_NEXUS);
        let nexus_id = game.add_region(nexus);
        let nexus_object = game
            .add_object(nexus_id, Object::default())
            .expect("Nexus object");
        let nexus_worker = game
            .add_unit(nexus_object, Unit::new(30, Some(faction_id), 0, 0))
            .expect("Nexus unit");

        pre_process_default_orders(&mut game, ruleset, &tables, &mut Rng::new(42));

        let faction = game.faction(faction_id).expect("faction");
        assert!(faction.activity.is_empty());
        assert_eq!(faction.num_shows, 0);
        let unit = game.unit(worker_id).expect("unit");
        assert_eq!(unit.orders.attack_orders, None);
        let Some(OrderData::Produce(order)) = &unit.orders.month_orders else {
            panic!("ordinary unit should receive a default produce order");
        };
        assert_eq!(order.item, ItemId::Silver.index() as i32);
        assert_eq!(order.skill, -1);
        assert_eq!(order.order.quiet, 1);
        assert_eq!(game.unit(guard_id).expect("guard").guard, GuardMode::Set);
        assert_eq!(
            game.unit(guard_mage_id).expect("guard mage").combat,
            lookup_skill(&tables.skills.skills, "FIRE").expect("Fire skill") as i32
        );
        assert_eq!(game.unit(auto_taxer_id).expect("auto-tax").taxing, 3);
        assert_eq!(
            game.unit(auto_taxer_id)
                .expect("auto-tax")
                .orders
                .month_orders,
            None
        );
        assert_eq!(
            game.unit(nexus_worker)
                .expect("Nexus unit")
                .orders
                .month_orders,
            None
        );
    }

    #[test]
    fn wandering_monster_defaults_match_cpp_direction_weights_and_draw_order() {
        let ruleset = <dyn Ruleset>::from_name("neworigins").expect("NewOrigins ruleset");
        let mut tables = ruleset.tables();
        let lion = ItemId::Lion;
        let lion_definition = &tables.item_families.items[lion.index()];
        assert_ne!(lion_definition.item_type & ItemTypeFlags::MONSTER, 0);
        let lion_abbreviation = lion_definition
            .abbreviation
            .clone()
            .expect("lion abbreviation");
        let monster = tables
            .item_families
            .monsters
            .iter_mut()
            .find(|monster| {
                monster.abbreviation.as_deref().is_some_and(|abbreviation| {
                    crate::text::is_equal(abbreviation, &lion_abbreviation)
                })
            })
            .expect("lion monster definition");
        monster.hostile = 100;
        monster.preferred_terrain.clear();
        monster.forbidden_terrain.clear();

        let mut game = Game::default();
        let mut source = Region::default();
        source.terrain_type = Some(R_PLAIN);
        let source_id = game.add_region(source);
        let mut destination = Region::default();
        destination.terrain_type = Some(R_PLAIN);
        let destination_id = game.add_region(destination);
        game.region_mut(source_id).expect("source").neighbors[0] = Some(destination_id);
        let faction_id = game.add_faction(Faction::new(3, &["WAR", "TRADE", "MAGIC"]));
        let object_id = game
            .add_object(source_id, Object::default())
            .expect("object");
        let mut monster_unit = Unit::new(31, Some(faction_id), 0, 0);
        monster_unit.unit_type = UnitType::WanderingMonster;
        monster_unit.items.push(Item {
            item_type: lion,
            num: 1,
            selling: 0,
            checked: false,
        });
        let monster_unit_id = game
            .add_unit(object_id, monster_unit)
            .expect("wandering monster");

        let seed = (0..10_000)
            .find(|seed| {
                let mut probe = Rng::new(*seed);
                probe.get_random(6) >= 4
            })
            .expect("seed selecting the only moving direction");
        let mut expected_rng = Rng::new(seed);
        let expected_choice = expected_rng.get_random(6);
        let expected_advance = expected_rng.get_random(100) < 100;
        let mut actual_rng = Rng::new(seed);

        pre_process_default_orders(&mut game, ruleset, &tables, &mut actual_rng);

        let unit = game.unit(monster_unit_id).expect("wandering monster");
        let Some(OrderData::Move(order)) = &unit.orders.month_orders else {
            panic!("the selected candidate should move");
        };
        assert_eq!(order.dirs.len(), 1);
        assert_eq!(order.dirs[0].dir, 0);
        assert_eq!(order.advancing, i32::from(expected_advance));
        assert_eq!(
            actual_rng.get_random(100),
            expected_rng.get_random(100),
            "the direction choice and conditional advance roll must both be consumed"
        );
        assert!(expected_choice >= 4);

        let mut stay_game = Game::default();
        let mut stay_region = Region::default();
        stay_region.terrain_type = Some(R_PLAIN);
        let stay_region_id = stay_game.add_region(stay_region);
        let stay_faction = stay_game.add_faction(Faction::new(4, &["WAR", "TRADE", "MAGIC"]));
        let stay_object = stay_game
            .add_object(stay_region_id, Object::default())
            .expect("stay object");
        let mut stay_unit = Unit::new(32, Some(stay_faction), 0, 0);
        stay_unit.unit_type = UnitType::WanderingMonster;
        let stay_unit_id = stay_game
            .add_unit(stay_object, stay_unit)
            .expect("staying monster");
        let mut actual_stay_rng = Rng::new(42);
        let mut expected_stay_rng = Rng::new(42);
        expected_stay_rng.get_random(4);

        pre_process_default_orders(&mut stay_game, ruleset, &tables, &mut actual_stay_rng);

        assert_eq!(
            stay_game
                .unit(stay_unit_id)
                .expect("staying monster")
                .orders
                .month_orders,
            None
        );
        assert_eq!(
            actual_stay_rng.get_random(100),
            expected_stay_rng.get_random(100),
            "staying consumes the direction choice but no advance roll"
        );
    }

    #[test]
    fn wandering_monster_routes_match_cpp_terrain_weights_and_mobility() {
        use super::wandering_direction_candidates;
        use std::collections::BTreeSet;

        use crate::defs::terrains::{R_FOREST, R_MOUNTAIN, R_OCEAN};

        let ruleset = <dyn Ruleset>::from_name("neworigins").expect("NewOrigins ruleset");
        let tables = ruleset.tables();
        let mut game = Game::default();
        let mut source = Region::default();
        source.terrain_type = Some(R_PLAIN);
        let source_id = game.add_region(source);
        let mut preferred_region = Region::default();
        preferred_region.terrain_type = Some(R_MOUNTAIN);
        let preferred_id = game.add_region(preferred_region);
        let mut neutral_region = Region::default();
        neutral_region.terrain_type = Some(R_PLAIN);
        let neutral_id = game.add_region(neutral_region);
        let mut forbidden_region = Region::default();
        forbidden_region.terrain_type = Some(R_FOREST);
        let forbidden_id = game.add_region(forbidden_region);
        let mut unrelated_region = Region::default();
        unrelated_region.terrain_type = Some(R_PLAIN);
        let unrelated_id = game.add_region(unrelated_region);
        let mut preferred_neighbor = Region::default();
        preferred_neighbor.terrain_type = Some(R_MOUNTAIN);
        let preferred_neighbor_id = game.add_region(preferred_neighbor);
        game.region_mut(source_id).expect("source").neighbors[0] = Some(preferred_id);
        game.region_mut(source_id).expect("source").neighbors[1] = Some(neutral_id);
        game.region_mut(source_id).expect("source").neighbors[2] = Some(forbidden_id);
        game.region_mut(source_id).expect("source").neighbors[3] = Some(unrelated_id);
        game.region_mut(neutral_id).expect("neutral").neighbors[0] = Some(preferred_neighbor_id);
        game.region_mut(forbidden_id).expect("forbidden").neighbors[0] =
            Some(preferred_neighbor_id);

        let mut monster = Unit::new(51, None, 0, 0);
        monster.unit_type = UnitType::WanderingMonster;
        monster.items.push(Item {
            item_type: ItemId::Lion,
            num: 1,
            selling: 0,
            checked: false,
        });
        let preferred = BTreeSet::from([tables.terrains[R_MOUNTAIN].similar_type]);
        let forbidden = BTreeSet::from([tables.terrains[R_FOREST].similar_type]);
        assert_eq!(
            wandering_direction_candidates(
                &monster,
                game.region(source_id).expect("source"),
                &game,
                ruleset,
                &tables,
                &preferred,
                &forbidden
            ),
            [-1, -1, -1, -1, 0, 0, 1]
        );

        let mut water_game = Game::default();
        let mut water_source = Region::default();
        water_source.terrain_type = Some(R_PLAIN);
        let water_source_id = water_game.add_region(water_source);
        let mut ocean = Region::default();
        ocean.terrain_type = Some(R_OCEAN);
        let ocean_id = water_game.add_region(ocean);
        water_game
            .region_mut(water_source_id)
            .expect("water source")
            .neighbors[0] = Some(ocean_id);
        let mut land_unit = Unit::new(52, None, 0, 0);
        land_unit.unit_type = UnitType::WanderingMonster;
        land_unit.items.push(Item {
            item_type: ItemId::Leaders,
            num: 1,
            selling: 0,
            checked: false,
        });
        assert_eq!(
            wandering_direction_candidates(
                &land_unit,
                water_game.region(water_source_id).expect("water source"),
                &water_game,
                ruleset,
                &tables,
                &BTreeSet::new(),
                &BTreeSet::new()
            ),
            [-1, -1, -1, -1]
        );
    }

    #[test]
    fn default_work_orders_match_cpp_autotax_and_work_rules() {
        let ruleset = <dyn Ruleset>::from_name("neworigins").expect("NewOrigins ruleset");
        let mut tables = ruleset.tables();
        let sword_weapon = tables
            .item_families
            .weapons
            .iter_mut()
            .find(|weapon| {
                weapon
                    .abbreviation
                    .as_deref()
                    .is_some_and(|abbreviation| crate::text::is_equal(abbreviation, "SWOR"))
            })
            .expect("sword weapon definition");
        sword_weapon.flags |= WeaponFlags::NEEDSKILL;
        let mut game = Game::default();
        let mut region = Region::default();
        region.terrain_type = Some(R_PLAIN);
        let region_id = game.add_region(region);
        let mut nexus = Region::default();
        nexus.terrain_type = Some(R_NEXUS);
        let nexus_id = game.add_region(nexus);

        let player_id = game.add_faction(Faction::new(3, ruleset.faction_types()));
        let npc_id = game.add_faction(Faction::new(1, ruleset.faction_types()));
        game.faction_mut(npc_id).expect("NPC faction").faction_types[0].1 = -1;

        let object_id = game
            .add_object(region_id, Object::default())
            .expect("plain object");
        let combat = lookup_skill(&tables.skills.skills, "COMB").expect("combat skill");
        let mut auto_taxer = Unit::new(41, Some(player_id), 0, 0);
        auto_taxer.flags |= 0x0004;
        auto_taxer.items.push(Item {
            item_type: ItemId::Leaders,
            num: 1,
            selling: 0,
            checked: false,
        });
        auto_taxer.skills.push(Skill {
            skill_type: combat,
            days: 30,
            exp: 0,
        });
        let auto_taxer_id = game.add_unit(object_id, auto_taxer).expect("auto-tax unit");

        let mut worker_without_taxers = Unit::new(42, Some(player_id), 0, 0);
        worker_without_taxers.flags |= 0x0004;
        let worker_without_taxers_id = game
            .add_unit(object_id, worker_without_taxers)
            .expect("ordinary unit");

        let mut ordered = Unit::new(43, Some(player_id), 0, 0);
        ordered.orders.month_orders = Some(OrderData::Idle(IdleOrder::default()));
        let ordered_id = game.add_unit(object_id, ordered).expect("ordered unit");

        let mut taxed = Unit::new(44, Some(player_id), 0, 0);
        taxed.taxing = 1;
        let taxed_id = game.add_unit(object_id, taxed).expect("taxed unit");

        let mut unskilled_swordsman = Unit::new(47, Some(player_id), 0, 0);
        unskilled_swordsman.flags |= 0x0004;
        unskilled_swordsman.items.push(Item {
            item_type: ItemId::Leaders,
            num: 1,
            selling: 0,
            checked: false,
        });
        unskilled_swordsman.items.push(Item {
            item_type: ItemId::Sword,
            num: 1,
            selling: 0,
            checked: false,
        });
        let unskilled_swordsman_id = game
            .add_unit(object_id, unskilled_swordsman)
            .expect("unskilled swordsman");

        let npc_object = game
            .add_object(region_id, Object::default())
            .expect("NPC object");
        let npc_unit = game
            .add_unit(npc_object, Unit::new(45, Some(npc_id), 0, 0))
            .expect("NPC unit");

        let nexus_object = game
            .add_object(nexus_id, Object::default())
            .expect("Nexus object");
        let nexus_unit = game
            .add_unit(nexus_object, Unit::new(46, Some(player_id), 0, 0))
            .expect("Nexus unit");

        apply_default_work_orders(&mut game, ruleset, &tables);

        assert_eq!(game.unit(auto_taxer_id).expect("auto-tax").taxing, 3);
        assert_eq!(
            game.unit(auto_taxer_id)
                .expect("auto-tax")
                .orders
                .month_orders,
            None
        );
        for unit_id in [worker_without_taxers_id, npc_unit, nexus_unit] {
            let unit = game.unit(unit_id).expect("unit");
            if unit_id == worker_without_taxers_id {
                assert!(matches!(
                    unit.orders.month_orders,
                    Some(OrderData::Produce(_))
                ));
            } else {
                assert_eq!(
                    unit.orders.month_orders, None,
                    "unit {unit_id:?} should not receive a work order"
                );
            }
        }
        assert!(matches!(
            game.unit(ordered_id)
                .expect("ordered unit")
                .orders
                .month_orders,
            Some(OrderData::Idle(_))
        ));
        assert_eq!(
            game.unit(taxed_id).expect("taxed unit").orders.month_orders,
            None
        );
        let unskilled_swordsman = game
            .unit(unskilled_swordsman_id)
            .expect("unskilled swordsman");
        let sword = &tables.item_families.items[ItemId::Sword.index()];
        let sword_weapon = tables
            .item_families
            .find_weapon(sword.abbreviation.as_deref().expect("sword abbreviation"))
            .expect("sword weapon definition");
        assert_ne!(sword_weapon.flags & WeaponFlags::NEEDSKILL, 0);
        assert_eq!(
            super::unit_skill_level(unskilled_swordsman, "COMB", &tables),
            0
        );
        assert_eq!(unskilled_swordsman.taxing, 3);
        assert_eq!(unskilled_swordsman.orders.month_orders, None);
        let Some(OrderData::Produce(order)) = &game
            .unit(worker_without_taxers_id)
            .expect("worker")
            .orders
            .month_orders
        else {
            panic!("default work should produce silver");
        };
        assert_eq!(order.order.quiet, 1);
        assert_eq!(order.item, ItemId::Silver.index() as i32);

        let disabled_ruleset = NoDefaultWorkRuleset;
        let mut disabled_game = Game::default();
        let mut disabled_region = Region::default();
        disabled_region.terrain_type = Some(R_PLAIN);
        let disabled_region_id = disabled_game.add_region(disabled_region);
        let disabled_faction =
            disabled_game.add_faction(Faction::new(3, disabled_ruleset.faction_types()));
        let disabled_object = disabled_game
            .add_object(disabled_region_id, Object::default())
            .expect("disabled-work object");
        let disabled_unit = disabled_game
            .add_unit(disabled_object, Unit::new(48, Some(disabled_faction), 0, 0))
            .expect("disabled-work unit");

        apply_default_work_orders(&mut disabled_game, &disabled_ruleset, &tables);

        assert_eq!(
            disabled_game
                .unit(disabled_unit)
                .expect("disabled-work unit")
                .orders
                .month_orders,
            None
        );
    }

    #[cfg(feature = "rng-trace")]
    #[test]
    fn pre_process_default_orders_matches_neworigins_reference_rng_prefix() {
        let loaded =
            crate::save::load_legacy(include_str!("../../../fixtures/neworigins/turn_1/game.in"))
                .expect("load NewOrigins reference fixture");
        let ruleset = loaded.ruleset;
        let tables = ruleset.tables();
        let mut game = loaded.game;
        let mut rng = loaded.rng;

        pre_process_default_orders(&mut game, ruleset, &tables, &mut rng);

        assert_eq!(
            rng.trace(),
            include_str!("../../../fixtures/neworigins/turn_1/preprocess.rng"),
            "preprocessing random draws differ from the C++ reference"
        );
    }

    // Neither supported ruleset sets GATES_NOT_PERENNIAL, so these pin the C++ logic directly.
    fn gate_after(gate_month: i32, month: i32, region: &mut Region) -> i32 {
        let mut defs = *<dyn crate::ruleset::Ruleset>::from_name("standard")
            .expect("standard")
            .game_defs();
        defs.gates_not_perennial = 3;
        defs.start_gates_open = 0;
        region.gate_month = gate_month;
        update_gate_status(region, &defs, month);
        region.gate_open
    }

    #[test]
    fn gate_status_opens_for_gates_not_perennial_months_from_its_gate_month() {
        let mut region = Region::default();
        assert_eq!(gate_after(5, 4, &mut region), 0);
        assert_eq!(gate_after(5, 5, &mut region), 1);
        assert_eq!(gate_after(5, 7, &mut region), 1);
        assert_eq!(gate_after(5, 8, &mut region), 0);
        // The window wraps past December.
        assert_eq!(gate_after(11, 1, &mut region), 1);
        assert_eq!(gate_after(11, 2, &mut region), 0);
    }

    #[test]
    fn gate_status_keeps_the_nexus_open() {
        let mut region = Region::default();
        region.terrain_type = Some(R_NEXUS);
        assert_eq!(gate_after(5, 0, &mut region), 1);
    }
}
