#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum BattleLogLevel {
    Normal = 0,
    Detailed = 1,
    Verbose = 2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum FactionActivityRules {
    Default = 0,
    Martial = 1,
    MartialMerged = 2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum DestroyBehavior {
    Immediate = 0,
    PerFigure = 1,
    PerSkill = 2,
}

pub mod flags {
    pub const MULT_NONE: i32 = 0;
    pub const MULT_MAGES: i32 = 1;
    pub const MULT_LEADERS: i32 = 2;
    pub const MULT_ALL: i32 = 3;

    pub const STARVE_NONE: i32 = 0;
    pub const STARVE_MAGES: i32 = 1;
    pub const STARVE_LEADERS: i32 = 2;
    pub const STARVE_ALL: i32 = 3;

    pub const TAX_ANYONE: i32 = 0x00001;
    pub const TAX_COMBAT_SKILL: i32 = 0x00002;
    pub const TAX_BOW_SKILL: i32 = 0x00004;
    pub const TAX_RIDING_SKILL: i32 = 0x00008;
    pub const TAX_USABLE_WEAPON: i32 = 0x00010;
    pub const TAX_ANY_WEAPON: i32 = 0x00020;
    pub const TAX_HORSE: i32 = 0x00040;
    pub const TAX_MELEE_WEAPON_AND_MATCHING_SKILL: i32 = 0x00080;
    pub const TAX_BOW_SKILL_AND_MATCHING_WEAPON: i32 = 0x00100;
    pub const TAX_HORSE_AND_RIDING_SKILL: i32 = 0x00200;
    pub const TAX_STEALTH_SKILL: i32 = 0x00400;
    pub const TAX_MAGE_DAMAGE: i32 = 0x00800;
    pub const TAX_MAGE_FEAR: i32 = 0x01000;
    pub const TAX_MAGE_OTHER: i32 = 0x02000;
    pub const TAX_ANY_MAGE: i32 = 0x04000;
    pub const TAX_MAGE_COMBAT_SPELL: i32 = 0x08000;
    pub const TAX_BATTLE_ITEM: i32 = 0x10000;
    pub const TAX_USABLE_BATTLE_ITEM: i32 = 0x20000;
    pub const TAX_CREATURES: i32 = 0x40000;
    pub const TAX_ILLUSIONS: i32 = 0x80000;
    pub const TAX_NORMAL: i32 = TAX_COMBAT_SKILL | TAX_USABLE_WEAPON;

    pub const FACLIM_MAGE_COUNT: i32 = 0;
    pub const FACLIM_FACTION_TYPES: i32 = 1;
    pub const FACLIM_UNLIMITED: i32 = 2;

    pub const WFLIGHT_NONE: i32 = 0;
    pub const WFLIGHT_MUST_LAND: i32 = 1;
    pub const WFLIGHT_UNLIMITED: i32 = 2;

    pub const NO_EFFECT: i32 = 0x00;
    pub const ALL: i32 = 0x01;
    pub const TOWNS: i32 = 0x02;
    pub const NONPLAINS: i32 = 0x04;
    pub const NONPLAINS_TOWNS_ONLY: i32 = 0x08;
    pub const DESERT_ONLY: i32 = 0x10;

    pub const REPORT_NOTHING: i32 = 0x0000;
    pub const REPORT_SHOW_PEASANTS: i32 = 0x0001;
    pub const REPORT_SHOW_REGION_MONEY: i32 = 0x0002;
    pub const REPORT_SHOW_WAGES: i32 = 0x0004;
    pub const REPORT_SHOW_MARKETS: i32 = 0x0008;
    pub const REPORT_SHOW_RESOURCES: i32 = 0x0010;
    pub const REPORT_SHOW_ENTERTAINMENT: i32 = 0x0020;
    pub const REPORT_SHOW_ECONOMY: i32 = REPORT_SHOW_PEASANTS
        | REPORT_SHOW_REGION_MONEY
        | REPORT_SHOW_WAGES
        | REPORT_SHOW_MARKETS
        | REPORT_SHOW_RESOURCES
        | REPORT_SHOW_ENTERTAINMENT;
    pub const REPORT_SHOW_USED_EXITS: i32 = 0x0040;
    pub const REPORT_SHOW_ALL_EXITS: i32 = 0x0080;
    pub const REPORT_SHOW_GUARDS: i32 = 0x0100;
    pub const REPORT_SHOW_INDOOR_UNITS: i32 = 0x0200;
    pub const REPORT_SHOW_OUTDOOR_UNITS: i32 = 0x0400;
    pub const REPORT_SHOW_UNITS: i32 =
        REPORT_SHOW_GUARDS | REPORT_SHOW_INDOOR_UNITS | REPORT_SHOW_OUTDOOR_UNITS;
    pub const REPORT_SHOW_BUILDINGS: i32 = 0x0800;
    pub const REPORT_SHOW_ROADS: i32 = 0x1000;
    pub const REPORT_SHOW_SHIPS: i32 = 0x2000;
    pub const REPORT_SHOW_STRUCTURES: i32 =
        REPORT_SHOW_BUILDINGS | REPORT_SHOW_ROADS | REPORT_SHOW_SHIPS;
    pub const REPORT_USE_UNIT_SKILLS: i32 = 0x8000;
    pub const REPORT_SHOW_REGION: i32 = REPORT_SHOW_ECONOMY | REPORT_SHOW_ALL_EXITS;
    pub const REPORT_SHOW_EVERYTHING: i32 =
        REPORT_SHOW_REGION | REPORT_SHOW_UNITS | REPORT_SHOW_STRUCTURES;

    pub const PREPARE_NONE: i32 = 0;
    pub const PREPARE_NORMAL: i32 = 1;
    pub const PREPARE_STRICT: i32 = 2;

    pub const ARMY_ROUT_FIGURES: i32 = 0;
    pub const ARMY_ROUT_HITS_INDIVIDUAL: i32 = 1;
    pub const ARMY_ROUT_HITS_FIGURE: i32 = 2;

    pub const ALLOW_TRANSPORT: i32 = 0x01;
    pub const QM_AFFECT_COST: i32 = 0x02;
    pub const QM_AFFECT_DIST: i32 = 0x04;
}
