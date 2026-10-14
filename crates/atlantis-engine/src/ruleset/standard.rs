use crate::defs::{
    enums::{BattleLogLevel, DestroyBehavior, FactionActivityRules},
    modify::TableModifier,
    objects::{
        O_AMINE, O_BKEEP, O_DERELICT, O_ISLE, O_MQUARRY, O_OCAVE, O_PRESERVE, O_ROADN, O_ROADNE,
        O_ROADNW, O_ROADS, O_ROADSE, O_ROADSW, O_SACGROVE, O_TEMPLE, O_WHIRL,
    },
    terrains::R_NEXUS,
    GameDefs, ItemId,
};
use crate::version::AtlVersion;

use super::{FactionLimits, Ruleset, VirtualMonster};

pub(super) struct Standard;
pub(super) static STANDARD: Standard = Standard;

static GAME_DEFS: GameDefs = GameDefs {
    ruleset_name: "Wyreth",
    ruleset_version: AtlVersion::new(2, 0, 0),
    max_speed: 8,
    phased_move_offset: 7,
    fleet_wind_boost: 2,
    fleet_crew_boost: 0,
    fleet_load_boost: 0,
    students_per_teacher: 10,
    maintenance_cost: 10,
    leader_cost: 20,
    maintenance_multiplier: 0,
    multiplier_use: 0,
    starve_percent: 33,
    skill_starvation: 0,
    start_money: 5020,
    work_fraction: 3,
    entertain_fraction: 20,
    entertain_income: 20,
    tax_base_income: 50,
    tax_bonus_weapon: 0,
    tax_bonus_armor: 0,
    tax_bonus_fort: 0,
    who_can_tax: 18,
    tax_pillage_month_long: 0,
    heals_per_man: 5,
    guard_regen: 20,
    city_guard: 40,
    guard_money: 50,
    city_pop: 20000,
    wmon_frequency: 10,
    lair_frequency: 10,
    faction_points: 5,
    times_reward: 50,
    towns_exist: 1,
    leaders_exist: 1,
    skill_limit_nonleaders: 1,
    mage_nonleaders: 0,
    races_exist: 1,
    gates_exist: 1,
    food_items_exist: 1,
    coastal_fish: 0,
    city_monsters_exist: 1,
    wandering_monsters_exist: 1,
    lair_monsters_exist: 1,
    weather_exists: 1,
    open_ended: 0,
    nexus_exists: 1,
    conquest_game: 0,
    random_economy: 1,
    variable_economy: 1,
    city_market_normal_amt: 50,
    city_market_advanced_amt: 20,
    city_market_trade_amt: 50,
    city_market_magic_amt: 20,
    more_profitable_trade_goods: 0,
    base_man_cost: 50,
    lastorders_maintained_by_scripts: 0,
    max_inactive_turns: 10,
    easier_underworld: 1,
    default_work_order: 1,
    faction_limit_type: 1,
    flight_over_water: 1,
    start_cities_exist: 1,
    safe_start_cities: 0,
    amt_start_city_guards: 500,
    start_city_guards_plate: 1,
    start_city_mages: 4,
    start_city_tactics: 4,
    apprentices_exist: 0,
    apprentice_name: "apprentice",
    world_name: "Wyreth",
    nexus_gate_out: 1,
    nexus_is_city: 0,
    battle_faction_info: 0,
    allow_withdraw: 1,
    city_rename_cost: 0,
    multi_hex_nexus: 0,
    icosahedral_world: 0,
    underworld_levels: 0,
    underdeep_levels: 0,
    abyss_level: 0,
    town_probability: 100,
    town_spread: 0,
    towns_not_adjacent: 0,
    less_arctic_towns: 0,
    ocean: 60,
    continent_size: 16,
    terrain_granularity: 4,
    lakes: 1,
    archipelago: 16,
    sever_land_bridges: 30,
    sea_limit: 6,
    lake_wage_effect: 0,
    lakeside_is_coastal: 0,
    odd_terrain: 0,
    improved_farsight: 0,
    gm_report: 1,
    decay: 0,
    limited_mages_per_building: 0,
    transit_report: 0,
    markets_show_advanced_items: 0,
    use_prepare_command: 0,
    monster_advance_min_percent: 15,
    monster_advance_hostile_percent: 0,
    have_email_special_commands: 1,
    start_cities_start_unlimited: 1,
    proportional_amts_usage: 0,
    use_weapon_armor_command: 1,
    monster_no_spoils: 0,
    monster_spoils_recovery: 0,
    max_assassin_free_attacks: 0,
    release_monsters: 0,
    check_monster_control_mid_turn: 0,
    detect_gate_numbers: 0,
    army_rout: 0,
    only_rout_once: 0,
    advanced_forts: 0,
    full_trueseeing_bonus: 0,
    improved_amts: 0,
    full_invis_on_self: 0,
    monster_battle_regen: 0,
    skill_practice_amount: 0,
    required_experience: 0,
    upkeep_minimum_food: 0,
    upkeep_maximum_food: -1,
    upkeep_food_value: 10,
    prevent_sail_through: 0,
    allow_trivial_portage: 0,
    gates_not_perennial: 0,
    start_gates_open: 0,
    show_closed_gates: 0,
    transport: 0,
    local_transport: 1,
    nonlocal_transport: 3,
    shipping_cost: 5,
    fractional_weight: 0,
    grow_races: 0,
    dynamic_population: 0,
    pop_growth: 100,
    delay_mortality: 3,
    delay_growth: 6,
    town_development: 100,
    tactics_needs_war: 0,
    allies_noaid: 0,
    harder_assassination: 0,
    disperse_gate_numbers: 0,
    undeath_contagion: 0,
    regions_economy: 0,
    advanced_tactics: 0,
    overwhelming: 0,
    strict_guard: 0,
    ocean_guard: 1,
    battle_log_level: BattleLogLevel::Normal,
    extended_fort_defence: 0,
    spoils_no_trade: 0,
    world_events: 0,
    faction_statistics: 0,
    build_no_trade: 0,
    transport_no_trade: 0,
    faction_activity: FactionActivityRules::Default,
    destroy_behavior: DestroyBehavior::Immediate,
    min_destroy_points: 200,
    max_destroy_percent: 34,
    half_riding_bonus: 0,
};

static FACTION_LIMITS: FactionLimits = FactionLimits {
    mages: &[0, 1, 2, 3, 5, 7],
    apprentices: &[0, 2, 4, 6, 10, 14],
    taxes: &[0, 10, 24, 40, 60, 100],
    trades: &[0, 10, 24, 40, 60, 100],
    quartermasters: &[0, 2, 4, 8, 12, 20],
    tacticians: &[0, 1, 2, 4, 6, 10],
    martial: &[0, 10, 25, 40, 60, 90],
};

static FACTION_TYPES: &[&str] = &["War", "Trade", "Magic"];

impl Ruleset for Standard {
    fn game_defs(&self) -> &'static GameDefs {
        &GAME_DEFS
    }

    fn faction_limits(&self) -> &'static FactionLimits {
        &FACTION_LIMITS
    }

    fn faction_types(&self) -> &'static [&'static str] {
        FACTION_TYPES
    }

    fn virtual_monster(&self) -> Option<VirtualMonster> {
        Some(VirtualMonster {
            object_type: O_BKEEP as i32,
            item: ItemId::Balrog,
            name: "Elder Demons",
            count: 200,
        })
    }

    fn apply_rule_table_modifications(&self, modifier: &mut TableModifier<'_>) {
        macro_rules! enable_item {
            ($item:ident) => {
                modifier.enable_item(ItemId::$item.index() as i32)
            };
        }

        let defs = self.game_defs();
        if defs.nexus_is_city != 0 && defs.towns_exist != 0 {
            modifier.clear_terrain_races(R_NEXUS as i32);
            modifier.modify_terrain_race(R_NEXUS as i32, 0, ItemId::HighElf.index() as i32);
            modifier.modify_terrain_race(R_NEXUS as i32, 1, ItemId::Viking.index() as i32);
            modifier.modify_terrain_race(R_NEXUS as i32, 2, ItemId::Plainsman.index() as i32);
            modifier.clear_terrain_items(R_NEXUS as i32);
            modifier.modify_terrain_items(R_NEXUS as i32, 0, ItemId::Iron.index() as i32, 100, 10);
            modifier.modify_terrain_items(R_NEXUS as i32, 1, ItemId::Wood.index() as i32, 100, 10);
            modifier.modify_terrain_items(R_NEXUS as i32, 2, ItemId::Stone.index() as i32, 100, 10);
            modifier.modify_terrain_economy(R_NEXUS as i32, 1000, 15, 50, 2);
        }

        enable_item!(Pick);
        enable_item!(Spear);
        enable_item!(Axe);
        enable_item!(Hammer);
        enable_item!(Mcrossbow);
        enable_item!(Mwagon);
        enable_item!(Glider);
        enable_item!(Net);
        enable_item!(Lasso);
        enable_item!(Bag);
        enable_item!(Spinning);
        enable_item!(LeatherArmor);
        enable_item!(ClothArmor);
        enable_item!(Boots);
        enable_item!(Baxe);
        enable_item!(Mbaxe);
        enable_item!(Imarm);
        enable_item!(Superbow);
        enable_item!(Lance);
        enable_item!(Javelin);
        enable_item!(Pike);

        modifier.enable_skill(modifier.skill_id("armorcraft"));
        modifier.enable_skill(modifier.skill_id("weaponcraft"));

        for object in [
            O_ROADN, O_ROADNE, O_ROADNW, O_ROADS, O_ROADSE, O_ROADSW, O_TEMPLE, O_MQUARRY, O_AMINE,
            O_PRESERVE, O_SACGROVE,
        ] {
            modifier.enable_object(object as i32);
        }
        for object in [O_ISLE, O_DERELICT, O_OCAVE, O_WHIRL] {
            modifier.enable_object(object as i32);
        }
        for item in [
            ItemId::Pirates,
            ItemId::Kraken,
            ItemId::Merfolk,
            ItemId::Elemental,
        ] {
            modifier.enable_item(item.index() as i32);
        }

        if defs.underdeep_levels > 0 || defs.underworld_levels > 1 {
            for item in [
                ItemId::Mushroom,
                ItemId::HealPotion,
                ItemId::Roughgem,
                ItemId::Gems,
            ] {
                modifier.enable_item(item.index() as i32);
            }
            modifier.enable_skill(modifier.skill_id("GEMC"));
        }

        modifier.modify_item_production_booster(
            ItemId::Axe.index() as i32,
            ItemId::Hammer.index() as i32,
            1,
        );
    }
}
