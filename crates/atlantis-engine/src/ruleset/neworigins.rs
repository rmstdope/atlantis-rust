use crate::defs::{
    enums::{BattleLogLevel, DestroyBehavior, FactionActivityRules},
    modify::TableModifier,
    objects::{
        O_ALCHEMISTLAB, O_AMINE, O_BOG, O_DARKTOWER, O_DERELICT, O_FAERIERING, O_GEMAPPRAISER,
        O_GIANTCASTLE, O_HUT, O_ICECAVE, O_ILAIR, O_ISLE, O_MAGETOWER, O_MCASTLE, O_MCITADEL,
        O_MFORTRESS, O_MQUARRY, O_MSTABLE, O_MTOWER, O_OASIS, O_OCAVE, O_PALACE, O_PRESERVE,
        O_ROADN, O_ROADNE, O_ROADNW, O_ROADS, O_ROADSE, O_ROADSW, O_SACGROVE, O_STABLE, O_TEMPLE,
        O_TRAPPINGHUT, O_TRAPPINGLODGE, O_WHIRL,
    },
    terrains::{
        R_CAVERN, R_CHASM, R_DESERT, R_FOREST, R_JUNGLE, R_MOUNTAIN, R_NEXUS, R_PLAIN, R_SWAMP,
        R_TUNDRA, R_TUNNELS, R_UFOREST,
    },
    EscapeFlags, GameDefs, ItemId,
};
use crate::version::AtlVersion;

use super::{FactionLimits, Ruleset, StartingFactionSetup};

pub(super) struct NewOrigins;
pub(super) static NEW_ORIGINS: NewOrigins = NewOrigins;

static GAME_DEFS: GameDefs = GameDefs {
    ruleset_name: "NewOrigins",
    ruleset_version: AtlVersion::new(3, 0, 0),
    max_speed: 8,
    phased_move_offset: 7,
    fleet_wind_boost: 2,
    fleet_crew_boost: 0,
    fleet_load_boost: 0,
    students_per_teacher: 10,
    maintenance_cost: 10,
    leader_cost: 50,
    maintenance_multiplier: 0,
    multiplier_use: 0,
    starve_percent: 33,
    skill_starvation: 0,
    start_money: 10000,
    work_fraction: 5,
    entertain_fraction: 20,
    entertain_income: 30,
    tax_base_income: 50,
    tax_bonus_weapon: 0,
    tax_bonus_armor: 0,
    tax_bonus_fort: 0,
    who_can_tax: 2578,
    tax_pillage_month_long: 1,
    heals_per_man: 5,
    guard_regen: 20,
    city_guard: 40,
    guard_money: 50,
    city_pop: 10000,
    wmon_frequency: 30,
    lair_frequency: 30,
    faction_points: 5,
    times_reward: 200,
    towns_exist: 1,
    leaders_exist: 1,
    skill_limit_nonleaders: 0,
    mage_nonleaders: 0,
    races_exist: 1,
    gates_exist: 1,
    food_items_exist: 1,
    coastal_fish: 0,
    city_monsters_exist: 1,
    wandering_monsters_exist: 1,
    lair_monsters_exist: 1,
    weather_exists: 0,
    open_ended: 0,
    nexus_exists: 1,
    conquest_game: 0,
    random_economy: 1,
    variable_economy: 1,
    city_market_normal_amt: 50,
    city_market_advanced_amt: 20,
    city_market_trade_amt: 50,
    city_market_magic_amt: 20,
    more_profitable_trade_goods: 1,
    base_man_cost: 50,
    lastorders_maintained_by_scripts: 1,
    max_inactive_turns: 10,
    easier_underworld: 1,
    default_work_order: 1,
    faction_limit_type: 1,
    flight_over_water: 1,
    start_cities_exist: 0,
    safe_start_cities: 0,
    amt_start_city_guards: 500,
    start_city_guards_plate: 1,
    start_city_mages: 4,
    start_city_tactics: 4,
    apprentices_exist: 1,
    apprentice_name: "apprentice",
    world_name: "Atlantis",
    nexus_gate_out: 1,
    nexus_is_city: 0,
    battle_faction_info: 1,
    allow_withdraw: 1,
    city_rename_cost: 10000,
    multi_hex_nexus: 0,
    icosahedral_world: 0,
    underworld_levels: 1,
    underdeep_levels: 0,
    abyss_level: 0,
    town_probability: 90,
    town_spread: 100,
    towns_not_adjacent: 100,
    less_arctic_towns: 0,
    ocean: 55,
    continent_size: 14,
    terrain_granularity: 4,
    lakes: 1,
    archipelago: 20,
    sever_land_bridges: 30,
    sea_limit: 6,
    lake_wage_effect: 0,
    lakeside_is_coastal: 0,
    odd_terrain: 70,
    improved_farsight: 1,
    gm_report: 1,
    decay: 0,
    limited_mages_per_building: 1,
    transit_report: 6591,
    markets_show_advanced_items: 0,
    use_prepare_command: 1,
    monster_advance_min_percent: 0,
    monster_advance_hostile_percent: 100,
    have_email_special_commands: 0,
    start_cities_start_unlimited: 1,
    proportional_amts_usage: 1,
    use_weapon_armor_command: 1,
    monster_no_spoils: 2,
    monster_spoils_recovery: 2,
    max_assassin_free_attacks: 0,
    release_monsters: 0,
    check_monster_control_mid_turn: 1,
    detect_gate_numbers: 1,
    army_rout: 0,
    only_rout_once: 1,
    advanced_forts: 0,
    full_trueseeing_bonus: 1,
    improved_amts: 0,
    full_invis_on_self: 1,
    monster_battle_regen: 0,
    skill_practice_amount: 5,
    required_experience: 0,
    upkeep_minimum_food: 0,
    upkeep_maximum_food: -1,
    upkeep_food_value: 50,
    prevent_sail_through: 1,
    allow_trivial_portage: 1,
    gates_not_perennial: 0,
    start_gates_open: 0,
    show_closed_gates: 0,
    transport: 7,
    local_transport: 2,
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
    allies_noaid: 1,
    harder_assassination: 0,
    disperse_gate_numbers: 1,
    undeath_contagion: 33,
    regions_economy: 1,
    advanced_tactics: 1,
    overwhelming: 0,
    strict_guard: 1,
    ocean_guard: 0,
    battle_log_level: BattleLogLevel::Verbose,
    extended_fort_defence: 1,
    spoils_no_trade: 1,
    world_events: 1,
    faction_statistics: 1,
    build_no_trade: 1,
    transport_no_trade: 1,
    faction_activity: FactionActivityRules::MartialMerged,
    destroy_behavior: DestroyBehavior::PerSkill,
    min_destroy_points: 200,
    max_destroy_percent: 34,
    half_riding_bonus: 1,
};

static FACTION_LIMITS: FactionLimits = FactionLimits {
    mages: &[1, 2, 3, 4, 5, 6],
    apprentices: &[1, 3, 5, 7, 10, 15],
    taxes: &[0, 15, 30, 50, 75, 100],
    trades: &[0, 15, 30, 50, 75, 100],
    quartermasters: &[0, 2, 5, 9, 14, 25],
    tacticians: &[0, 1, 2, 4, 6, 10],
    martial: &[0, 10, 25, 40, 60, 90],
};

static FACTION_TYPES: &[&str] = &["Martial", "Magic"];

impl Ruleset for NewOrigins {
    fn game_defs(&self) -> &'static GameDefs {
        &GAME_DEFS
    }

    fn faction_limits(&self) -> &'static FactionLimits {
        &FACTION_LIMITS
    }

    fn faction_types(&self) -> &'static [&'static str] {
        FACTION_TYPES
    }

    fn starting_faction_setup(&self, _turn_number: i32) -> StartingFactionSetup {
        StartingFactionSetup {
            unclaimed_per_turn: 300,
            studies: vec![
                ("OBSE", 30),
                ("FORC", 30),
                ("PATT", 30),
                ("SPIR", 30),
                ("GATE", 30),
                ("FIRE", 30),
                ("COMB", 180),
            ],
            set_flags: 0x0001,
            clear_flags: 0x0002 | 0x0008 | 0x0010,
        }
    }

    fn apply_rule_table_modifications(&self, modifier: &mut TableModifier<'_>) {
        let defs = self.game_defs();
        if defs.nexus_is_city != 0 && defs.towns_exist != 0 {
            modifier.clear_terrain_races(R_NEXUS as i32);
            modifier.modify_terrain_race(R_NEXUS as i32, 0, id(ItemId::HighElf));
            modifier.modify_terrain_race(R_NEXUS as i32, 1, id(ItemId::Man));
            modifier.modify_terrain_race(R_NEXUS as i32, 2, id(ItemId::HillDwarf));
            modifier.clear_terrain_items(R_NEXUS as i32);
            modifier.modify_terrain_items(R_NEXUS as i32, 0, id(ItemId::Iron), 100, 10);
            modifier.modify_terrain_items(R_NEXUS as i32, 1, id(ItemId::Wood), 100, 10);
            modifier.modify_terrain_items(R_NEXUS as i32, 2, id(ItemId::Stone), 100, 10);
            modifier.modify_terrain_economy(R_NEXUS as i32, 1000, 15, 50, 2);
        }

        for item in [
            ItemId::Camel,
            ItemId::Mcrossbow,
            ItemId::Mwagon,
            ItemId::Glider,
            ItemId::LeatherArmor,
            ItemId::Spear,
            ItemId::Javelin,
            ItemId::Mshield,
            ItemId::Ishield,
            ItemId::Wshield,
            ItemId::Aegis,
            ItemId::Windchime,
            ItemId::GateCrystal,
            ItemId::Staffofh,
            ItemId::ScryingOrb,
            ItemId::Cornucopia,
            ItemId::BookOfExorcism,
            ItemId::HolySymbol,
            ItemId::Censer,
            ItemId::Fsword,
            ItemId::Mushroom,
            ItemId::HealPotion,
            ItemId::Gems,
            ItemId::Pike,
            ItemId::Baxe,
            ItemId::Pick,
            ItemId::Axe,
            ItemId::Hammer,
            ItemId::Net,
            ItemId::Lasso,
            ItemId::Bag,
            ItemId::Spinning,
            ItemId::Catapult,
            ItemId::SteelDefender,
        ] {
            modifier.enable_item(id(item));
        }

        for item in [
            ItemId::Admantium,
            ItemId::Adsword,
            ItemId::Adring,
            ItemId::Adplate,
        ] {
            modifier.enable_item(id(item));
        }
        modifier.modify_item_production_skill(id(ItemId::Admantium), Some("MINI"), 5);
        modifier.modify_item_production_skill(id(ItemId::Adsword), Some("WEAP"), 5);
        modifier.modify_item_production_skill(id(ItemId::Adbaxe), Some("WEAP"), 5);
        modifier.modify_item_production_skill(id(ItemId::Adring), Some("ARMO"), 5);
        modifier.modify_item_production_skill(id(ItemId::Adplate), Some("ARMO"), 5);

        modifier.disable_item(id(ItemId::RelicOfGrace));
        for item in [
            ItemId::Superbow,
            ItemId::Boots,
            ItemId::ClothArmor,
            ItemId::Mbaxe,
            ItemId::Adbaxe,
            ItemId::Roughgem,
        ] {
            modifier.disable_item(id(item));
        }
        modifier.disable_skill(modifier.skill_id("create staff of lightning"));
        modifier.disable_item(id(ItemId::Staffofl));

        for skill in [
            "enchant shields",
            "create aegis",
            "create windchime",
            "create gate crystal",
            "create staff of healing",
            "create scrying orb",
            "create cornucopia",
            "create book of exorcism",
            "create holy symbol",
            "create censer of protection",
            "create flaming sword",
            "transmutation",
        ] {
            modifier.enable_skill(modifier.skill_id(skill));
        }
        modifier.disable_skill(modifier.skill_id("camel training"));
        modifier.disable_skill(modifier.skill_id("ranching"));
        modifier.disable_skill(modifier.skill_id("endurance"));
        modifier.disable_skill(modifier.skill_id("gemcutting"));
        modifier.enable_skill(modifier.skill_id("cooking"));
        modifier.enable_item(id(ItemId::Food));

        modifier.modify_skill_dependency(modifier.skill_id("RAIS"), 0, Some("SUSK"), 3);
        modifier.modify_skill_dependency(modifier.skill_id("summon lich"), 0, Some("RAIS"), 3);
        modifier.modify_skill_dependency(modifier.skill_id("dragon lore"), 1, Some("WOLF"), 3);

        for (item, output) in [
            (ItemId::Skeleton, 200),
            (ItemId::Wolf, 200),
            (ItemId::Imp, 200),
            (ItemId::Undead, 100),
            (ItemId::Eagle, 100),
            (ItemId::Demon, 100),
            (ItemId::Dragon, 20),
            (ItemId::Lich, 30),
        ] {
            modifier.modify_item_magic_output(id(item), output);
        }
        let escape = EscapeFlags::ESC_LEV_LINEAR | EscapeFlags::LOSE_LINKED;
        modifier.modify_item_escape(id(ItemId::Imp), escape, Some("SUIM"), 20);
        modifier.modify_item_escape(id(ItemId::Demon), escape, Some("SUDE"), 20);
        modifier.modify_item_escape(id(ItemId::Balrog), escape, Some("SUBA"), 20);

        for object in [O_ROADN, O_ROADNE, O_ROADNW, O_ROADS, O_ROADSE, O_ROADSW] {
            modifier.enable_object(object as i32);
            modifier.modify_object_construction(
                object as i32,
                id(ItemId::Stone),
                30,
                Some("BUIL"),
                2,
            );
        }
        for object in [
            O_TEMPLE,
            O_MQUARRY,
            O_AMINE,
            O_PRESERVE,
            O_SACGROVE,
            O_MTOWER,
            O_MFORTRESS,
            O_MCITADEL,
            O_STABLE,
            O_MSTABLE,
            O_HUT,
            O_TRAPPINGLODGE,
            O_FAERIERING,
            O_ALCHEMISTLAB,
            O_OASIS,
            O_TRAPPINGHUT,
        ] {
            modifier.enable_object(object as i32);
        }
        modifier.disable_object(O_GEMAPPRAISER as i32);
        modifier.disable_object(O_PALACE as i32);
        modifier.modify_object_name(O_MFORTRESS as i32, "Magical Fortress");
        modifier.modify_object_name(O_MCASTLE as i32, "Magical Castle");
        for object in [O_ISLE, O_DERELICT, O_OCAVE, O_WHIRL] {
            modifier.enable_object(object as i32);
        }

        for item in [
            ItemId::Pirates,
            ItemId::Kraken,
            ItemId::Merfolk,
            ItemId::Elemental,
            ItemId::Hydra,
        ] {
            modifier.enable_item(id(item));
        }
        modifier.enable_item(O_BOG as i32);
        modifier.enable_item(ItemId::IceDragon.index() as i32);
        modifier.enable_item(O_ICECAVE as i32);
        modifier.enable_item(ItemId::Illyrthid.index() as i32);
        modifier.enable_item(O_ILAIR as i32);
        modifier.enable_item(ItemId::Devil.index() as i32);
        modifier.enable_item(ItemId::StormGiant.index() as i32);
        modifier.enable_item(ItemId::CloudGiant.index() as i32);
        modifier.enable_item(O_GIANTCASTLE as i32);
        modifier.enable_item(ItemId::Warriors.index() as i32);
        modifier.enable_item(ItemId::Darkmage.index() as i32);
        modifier.enable_item(O_DARKTOWER as i32);
        modifier.enable_item(ItemId::Magicians.index() as i32);
        modifier.enable_item(O_MAGETOWER as i32);

        for item in [
            ItemId::Eskimo,
            ItemId::Tribesman,
            ItemId::Nomad,
            ItemId::TribalElf,
            ItemId::Viking,
            ItemId::Barbarian,
            ItemId::Darkman,
            ItemId::DesertDwarf,
            ItemId::Plainsman,
            ItemId::SeaElf,
            ItemId::Greyelf,
            ItemId::Minotaur,
            ItemId::Ogreman,
            ItemId::Hobbit,
        ] {
            modifier.disable_item(id(item));
        }

        modifier.modify_item_base_price(id(ItemId::Leaders), 700);
        configure_race(
            modifier,
            ItemId::Man,
            "HUMN",
            40,
            4,
            2,
            &["BUIL", "RIDI", "COMB", "MINI", "FARM", "COOK"],
        );
        configure_race(
            modifier,
            ItemId::HillDwarf,
            "HDWA",
            40,
            5,
            2,
            &["ARMO", "WEAP", "QUAR", "MINI", "BUIL"],
        );
        configure_race(
            modifier,
            ItemId::IceDwarf,
            "IDWA",
            40,
            5,
            2,
            &["COMB", "WEAP", "MINI", "FISH", "ARMO"],
        );
        configure_race(
            modifier,
            ItemId::HighElf,
            "HELF",
            40,
            5,
            2,
            &["HORS", "FISH", "LBOW", "SHIP", "SAIL"],
        );
        configure_race(
            modifier,
            ItemId::WoodElf,
            "WELF",
            40,
            5,
            2,
            &["LUMB", "LBOW", "ENTE", "CARP", "FISH", "COOK"],
        );
        configure_race(
            modifier,
            ItemId::Gnome,
            "GNOM",
            30,
            5,
            2,
            &["HERB", "QUAR", "ENTE", "XBOW", "HEAL"],
        );
        modifier.modify_item_capacities(id(ItemId::Gnome), 7, 0, 0, 0);
        modifier.modify_item_weight(id(ItemId::Gnome), 5);
        configure_race(
            modifier,
            ItemId::CentaurMan,
            "CTAU",
            70,
            5,
            2,
            &["LUMB", "HORS", "RIDI", "HEAL", "FARM"],
        );
        configure_race(
            modifier,
            ItemId::Lizardman,
            "LIZA",
            40,
            5,
            2,
            &["HUNT", "HERB", "CARP", "SAIL", "HEAL"],
        );
        configure_race(
            modifier,
            ItemId::Goblinman,
            "GBLN",
            30,
            5,
            2,
            &["QUAR", "XBOW", "SHIP", "WEAP", "ENTE"],
        );
        modifier.modify_item_capacities(id(ItemId::Goblinman), 7, 0, 0, 0);
        modifier.modify_item_weight(id(ItemId::Goblinman), 5);
        configure_race(
            modifier,
            ItemId::Gnoll,
            "GNOL",
            40,
            5,
            2,
            &["HORS", "HUNT", "COMB", "ARMO", "CARP", "COOK"],
        );
        configure_race(
            modifier,
            ItemId::Orc,
            "ORC",
            40,
            5,
            2,
            &["MINI", "LUMB", "COMB", "BUIL", "SHIP"],
        );
        configure_race(
            modifier,
            ItemId::Drowman,
            "DRLF",
            40,
            5,
            2,
            &["COMB", "HUNT", "LBOW", "LUMB", "COOK"],
        );
        configure_race(
            modifier,
            ItemId::UnderDwarf,
            "UDWA",
            40,
            5,
            2,
            &["WEAP", "ARMO", "QUAR", "MINI", "BUIL"],
        );

        modifier.clear_terrain_races(R_PLAIN as i32);
        set_terrain_races(
            modifier,
            R_PLAIN,
            &[
                ItemId::HighElf,
                ItemId::CentaurMan,
                ItemId::Gnoll,
                ItemId::Man,
            ],
            &[ItemId::HighElf, ItemId::CentaurMan, ItemId::Man],
        );
        modifier.modify_terrain_economy(R_PLAIN as i32, 800, 12, 40, 1);

        modifier.clear_terrain_races(R_FOREST as i32);
        set_terrain_races(
            modifier,
            R_FOREST,
            &[ItemId::WoodElf, ItemId::CentaurMan, ItemId::HighElf],
            &[ItemId::WoodElf, ItemId::CentaurMan, ItemId::HighElf],
        );
        modifier.modify_terrain_economy(R_FOREST as i32, 600, 12, 20, 2);

        modifier.clear_terrain_races(R_MOUNTAIN as i32);
        set_terrain_races(
            modifier,
            R_MOUNTAIN,
            &[ItemId::HillDwarf, ItemId::Orc, ItemId::Man],
            &[ItemId::HillDwarf, ItemId::Orc, ItemId::Man],
        );
        modifier.modify_terrain_economy(R_MOUNTAIN as i32, 400, 11, 20, 2);

        modifier.clear_terrain_races(R_SWAMP as i32);
        set_terrain_races(
            modifier,
            R_SWAMP,
            &[
                ItemId::Lizardman,
                ItemId::Goblinman,
                ItemId::Gnoll,
                ItemId::Orc,
            ],
            &[ItemId::Lizardman, ItemId::Goblinman, ItemId::Gnoll],
        );
        modifier.modify_terrain_economy(R_SWAMP as i32, 500, 11, 10, 2);

        modifier.clear_terrain_races(R_JUNGLE as i32);
        set_terrain_races(
            modifier,
            R_JUNGLE,
            &[
                ItemId::Orc,
                ItemId::WoodElf,
                ItemId::Lizardman,
                ItemId::Gnome,
            ],
            &[ItemId::Orc, ItemId::WoodElf, ItemId::Lizardman],
        );
        modifier.modify_terrain_economy(R_JUNGLE as i32, 500, 11, 20, 2);

        modifier.clear_terrain_races(R_DESERT as i32);
        set_terrain_races(
            modifier,
            R_DESERT,
            &[ItemId::Gnoll, ItemId::Goblinman, ItemId::Man],
            &[ItemId::Gnoll, ItemId::Goblinman, ItemId::Man],
        );
        modifier.modify_terrain_economy(R_DESERT as i32, 400, 11, 10, 1);

        modifier.clear_terrain_races(R_TUNDRA as i32);
        set_terrain_races(
            modifier,
            R_TUNDRA,
            &[ItemId::IceDwarf, ItemId::Gnome, ItemId::Gnoll],
            &[ItemId::IceDwarf, ItemId::Gnome, ItemId::Gnoll],
        );
        modifier.modify_terrain_economy(R_TUNDRA as i32, 400, 11, 10, 2);

        modifier.clear_terrain_races(R_CAVERN as i32);
        set_terrain_races(
            modifier,
            R_CAVERN,
            &[ItemId::Drowman, ItemId::UnderDwarf, ItemId::Orc],
            &[ItemId::UnderDwarf, ItemId::Goblinman, ItemId::Orc],
        );
        modifier.modify_terrain_economy(R_CAVERN as i32, 300, 11, 10, 2);

        modifier.clear_terrain_races(R_UFOREST as i32);
        set_terrain_races(
            modifier,
            R_UFOREST,
            &[ItemId::Drowman, ItemId::Gnome, ItemId::Goblinman],
            &[ItemId::Drowman, ItemId::Gnome, ItemId::Goblinman],
        );
        modifier.modify_terrain_economy(R_UFOREST as i32, 400, 11, 10, 2);

        modifier.clear_terrain_races(R_CHASM as i32);
        set_terrain_races(
            modifier,
            R_CHASM,
            &[ItemId::Drowman, ItemId::Gnome, ItemId::Goblinman],
            &[ItemId::UnderDwarf, ItemId::Drowman, ItemId::Goblinman],
        );
        modifier.modify_terrain_economy(R_CHASM as i32, 200, 11, 10, 4);

        modifier.modify_terrain_economy(R_TUNNELS as i32, 0, 0, 0, 2);
    }
}

fn id(item: ItemId) -> i32 {
    item.index() as i32
}

fn configure_race(
    modifier: &mut TableModifier<'_>,
    item: ItemId,
    abbreviation: &str,
    price: i32,
    special_level: i32,
    default_level: i32,
    skills: &[&str],
) {
    modifier.enable_item(id(item));
    modifier.modify_item_base_price(id(item), price);
    modifier.modify_race_skill_levels(abbreviation, special_level, default_level);
    for (slot, skill) in skills.iter().enumerate() {
        modifier.modify_race_skills(abbreviation, slot as i32, Some(skill));
    }
}

fn set_terrain_races(
    modifier: &mut TableModifier<'_>,
    terrain: usize,
    races: &[ItemId],
    coastal_races: &[ItemId],
) {
    for (slot, race) in races.iter().copied().enumerate() {
        modifier.modify_terrain_race(terrain as i32, slot as i32, id(race));
    }
    for (slot, race) in coastal_races.iter().copied().enumerate() {
        modifier.modify_terrain_coast_race(terrain as i32, slot as i32, id(race));
    }
}
