use std::fmt::Write as _;

use serde::Deserialize;

use super::Tables;

pub const ITEM_COUNT: usize = 229;
pub const NITEMS: usize = ITEM_COUNT;
// C++ NUM_ATTACK_TYPES excludes the non-resistable sentinel at numeric value 6.
pub const NUM_ATTACK_TYPES: usize = 6;
pub const NUM_WEAPON_CLASSES: usize = 8;
pub const MAX_WEAPON_BONUS_TARGETS: usize = 4;

macro_rules! item_id_enum {
    ($($variant:ident),+ $(,)?) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        #[repr(usize)]
        pub enum ItemId {
            $($variant),+
        }

        impl ItemId {
            pub const COUNT: usize = ITEM_COUNT;

            pub const fn index(self) -> usize {
                self as usize
            }

            pub fn from_index(index: usize) -> Option<Self> {
                const ALL: [ItemId; ITEM_COUNT] = [$(
                    ItemId::$variant
                ),+];
                ALL.get(index).copied()
            }
        }
    };
}

item_id_enum! {
    Leaders,
    Viking,
    Barbarian,
    Plainsman,
    Eskimo,
    Nomad,
    Tribesman,
    Darkman,
    WoodElf,
    SeaElf,
    HighElf,
    TribalElf,
    IceDwarf,
    HillDwarf,
    UnderDwarf,
    DesertDwarf,
    Orc,
    Gnome,
    CentaurMan,
    Silver,
    Grain,
    Livestock,
    Iron,
    Wood,
    Stone,
    Fur,
    Fish,
    Herbs,
    Horse,
    Longboat,
    Longship,
    Raft,
    Cog,
    Clipper,
    Galleon,
    Agalleon,
    Galley,
    Corsair,
    Balloon,
    Airship,
    Cloudship,
    Sword,
    Crossbow,
    Longbow,
    ChainArmor,
    PlateArmor,
    Wagon,
    Mithril,
    Ironwood,
    Whorse,
    Floater,
    Rootstone,
    Yew,
    Msword,
    Mplate,
    Doublebow,
    Ivory,
    Pearl,
    Jewelry,
    Figurines,
    Tarotcards,
    Caviar,
    Wine,
    Spices,
    Chocolate,
    Truffles,
    Vodka,
    Roses,
    Perfume,
    Silk,
    Velvet,
    Mink,
    Cashmere,
    Cotton,
    Dyes,
    Wool,
    Lion,
    Wolf,
    Gbear,
    Crocodile,
    Anaconda,
    Scorpion,
    Pbear,
    Rat,
    Spider,
    Lizard,
    Trent,
    Roc,
    Bthing,
    Kong,
    Sphinx,
    Iwurm,
    Dragon,
    Centaur,
    Kobold,
    Ogre,
    Lmen,
    Wmen,
    Sandling,
    Yeti,
    Goblin,
    Troll,
    Ettin,
    Skeleton,
    Undead,
    Lich,
    Imp,
    Demon,
    Balrog,
    Eagle,
    Amuletofi,
    Ringofi,
    Cloakofi,
    Staffoff,
    Staffofl,
    Amuletofts,
    Amuletofp,
    Runesword,
    Shieldstone,
    Mcarpet,
    Iwolf,
    Ieagle,
    Idragon,
    Iskeleton,
    Iundead,
    Ilich,
    Iimp,
    Idemon,
    Ibalrog,
    Portal,
    Peasant,
    Pick,
    Spear,
    Axe,
    Hammer,
    Mcrossbow,
    Mwagon,
    Glider,
    Net,
    Lasso,
    Bag,
    Spinning,
    LeatherArmor,
    ClothArmor,
    Boots,
    Pirates,
    Kraken,
    Merfolk,
    Elemental,
    Man,
    Fairy,
    Lizardman,
    Uruk,
    Goblinman,
    Hobbit,
    Gnoll,
    Drowman,
    Merc,
    Titan,
    Amazon,
    Ogreman,
    Highlander,
    Minotaur,
    Lance,
    Mushroom,
    Rrat,
    Noogle,
    Mutant,
    Baxe,
    Mbaxe,
    Admantium,
    Adsword,
    Adbaxe,
    Imarm,
    Adring,
    Adplate,
    Superbow,
    Camel,
    Greyelf,
    Drow,
    Hydra,
    StormGiant,
    CloudGiant,
    Illyrthid,
    Sorcerers,
    Magicians,
    Darkmage,
    Warriors,
    IceDragon,
    HealPotion,
    Roughgem,
    Gems,
    Javelin,
    Pike,
    Mwolf,
    Mspider,
    Mole,
    Bplate,
    Mchain,
    Fsword,
    Qstaff,
    Sabre,
    Mace,
    Mstar,
    Dagger,
    Pdagger,
    Bhammer,
    Bow,
    Shortbow,
    Heavycrossbow,
    Harp,
    Food,
    Mshield,
    Ishield,
    Wshield,
    Aegis,
    Windchime,
    GateCrystal,
    Staffofh,
    ScryingOrb,
    Cornucopia,
    BookOfExorcism,
    HolySymbol,
    Censer,
    RelicOfGrace,
    Turt,
    Catapult,
    SteelDefender,
    Devil,
}

pub struct ItemTypeFlags;

impl ItemTypeFlags {
    pub const NORMAL: i32 = 0x00001;
    pub const ADVANCED: i32 = 0x00002;
    pub const TRADE: i32 = 0x00004;
    pub const MAN: i32 = 0x00008;
    pub const MONSTER: i32 = 0x00010;
    pub const MAGIC: i32 = 0x00020;
    pub const WEAPON: i32 = 0x00040;
    pub const ARMOR: i32 = 0x00080;
    pub const MOUNT: i32 = 0x00100;
    pub const BATTLE: i32 = 0x00200;
    pub const SPECIAL: i32 = 0x00400;
    pub const TOOL: i32 = 0x00800;
    pub const FOOD: i32 = 0x01000;
    pub const ILLUSION: i32 = 0x02000;
    pub const UNDEAD: i32 = 0x04000;
    pub const DEMON: i32 = 0x08000;
    pub const LEADER: i32 = 0x10000;
    pub const MONEY: i32 = 0x20000;
    pub const ANIMAL: i32 = 0x40000;
    pub const SHIP: i32 = 0x80000;
    pub const MAGE_ONLY: i32 = 0x100000;
    pub const ALWAYS_SPOIL: i32 = 0x200000;
    pub const NEVER_SPOIL: i32 = 0x400000;
}

pub struct ItemFlags;

impl ItemFlags {
    pub const CANTGIVE: i32 = 0x01;
    pub const DISABLED: i32 = 0x02;
    pub const NOMARKET: i32 = 0x04;
    pub const ORINPUTS: i32 = 0x08;
    pub const SKILLOUT: i32 = 0x10;
    pub const NOTRANSPORT: i32 = 0x20;
    pub const MANPRODUCE: i32 = 0x40;
    pub const SKILLOUT_HALF: i32 = 0x80;
}

pub struct EscapeFlags;

impl EscapeFlags {
    pub const LOSE_LINKED: i32 = 0x01;
    pub const HAS_SKILL: i32 = 0x02;
    pub const ESC_LEV_LINEAR: i32 = 0x04;
    pub const ESC_LEV_SQUARE: i32 = 0x08;
    pub const ESC_LEV_CUBE: i32 = 0x10;
    pub const ESC_LEV_QUAD: i32 = 0x20;
    pub const LOSS_CHANCE: i32 = 0x40;
    pub const ESC_NUM_SQUARE: i32 = 0x80;
}

pub struct WeaponFlags;

impl WeaponFlags {
    pub const NEEDSKILL: i32 = 0x001;
    pub const ALWAYSREADY: i32 = 0x002;
    pub const NODEFENSE: i32 = 0x004;
    pub const NOFOOT: i32 = 0x008;
    pub const NOMOUNT: i32 = 0x010;
    pub const SHORT: i32 = 0x020;
    pub const LONG: i32 = 0x040;
    pub const RANGED: i32 = 0x080;
    pub const NOATTACKERSKILL: i32 = 0x100;
    pub const RIDINGBONUS: i32 = 0x200;
    pub const RIDINGBONUSDEFENSE: i32 = 0x400;
}

pub struct ArmorFlags;

impl ArmorFlags {
    pub const USEINASSASSINATE: i32 = 0x01;
}

pub struct BattleItemFlags;

impl BattleItemFlags {
    pub const MAGEONLY: i32 = 0x01;
    pub const SPECIAL: i32 = 0x02;
    pub const SHIELD: i32 = 0x04;
    pub const EXCLUSIVE: i32 = 0x08;
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(try_from = "i32")]
#[repr(i32)]
pub enum AttackType {
    Combat = 0,
    Energy = 1,
    Spirit = 2,
    Weather = 3,
    Riding = 4,
    Ranged = 5,
    NonResistable = 6,
}

impl TryFrom<i32> for AttackType {
    type Error = &'static str;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Combat),
            1 => Ok(Self::Energy),
            2 => Ok(Self::Spirit),
            3 => Ok(Self::Weather),
            4 => Ok(Self::Riding),
            5 => Ok(Self::Ranged),
            6 => Ok(Self::NonResistable),
            _ => Err("invalid attack type"),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(try_from = "i32")]
#[repr(i32)]
pub enum Ethnicity {
    None = 0,
    Viking = 1,
    Barbarian = 2,
    Man = 3,
    Eskimo = 4,
    Nomad = 5,
    Tribesman = 6,
    Highelf = 7,
    Elf = 8,
    Dwarf = 9,
    Orc = 10,
    Lizardman = 11,
    Drow = 12,
    Titan = 13,
    Hero = 14,
}

impl TryFrom<i32> for Ethnicity {
    type Error = &'static str;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::None),
            1 => Ok(Self::Viking),
            2 => Ok(Self::Barbarian),
            3 => Ok(Self::Man),
            4 => Ok(Self::Eskimo),
            5 => Ok(Self::Nomad),
            6 => Ok(Self::Tribesman),
            7 => Ok(Self::Highelf),
            8 => Ok(Self::Elf),
            9 => Ok(Self::Dwarf),
            10 => Ok(Self::Orc),
            11 => Ok(Self::Lizardman),
            12 => Ok(Self::Drow),
            13 => Ok(Self::Titan),
            14 => Ok(Self::Hero),
            _ => Err("invalid ethnicity"),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(try_from = "i32")]
#[repr(i32)]
pub enum WeaponClass {
    Slashing = 0,
    Piercing = 1,
    Crushing = 2,
    Cleaving = 3,
    ArmorPiercing = 4,
    MagicEnergy = 5,
    MagicSpirit = 6,
    MagicWeather = 7,
}

impl TryFrom<i32> for WeaponClass {
    type Error = &'static str;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Slashing),
            1 => Ok(Self::Piercing),
            2 => Ok(Self::Crushing),
            3 => Ok(Self::Cleaving),
            4 => Ok(Self::ArmorPiercing),
            5 => Ok(Self::MagicEnergy),
            6 => Ok(Self::MagicSpirit),
            7 => Ok(Self::MagicWeather),
            _ => Err("invalid weapon class"),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct Materials {
    pub item: i32,
    pub amount: i32,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct ItemDef {
    pub name: Option<String>,
    pub plural: Option<String>,
    pub abbreviation: Option<String>,
    pub flags: i32,
    pub production_skill: Option<String>,
    pub production_level: i32,
    pub production_months: i32,
    pub production_output: i32,
    pub production_inputs: [Materials; 4],
    pub magic_skill: Option<String>,
    pub magic_level: i32,
    pub magic_output: i32,
    pub magic_inputs: [Materials; 4],
    pub weight: i32,
    #[serde(rename = "type")]
    pub item_type: i32,
    pub base_price: i32,
    pub combat: i32,
    pub walk: i32,
    pub ride: i32,
    pub fly: i32,
    pub swim: i32,
    pub speed: i32,
    pub hitch_item: i32,
    pub hitch_walk: i32,
    pub mult_item: i32,
    pub mult_value: i32,
    pub max_inventory: i32,
    pub escape: i32,
    pub escape_skill: Option<String>,
    pub escape_value: i32,
    pub grant_skill: Option<String>,
    pub grant_from: [Option<String>; 4],
    pub min_grant: i32,
    pub max_grant: i32,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct ManDef {
    pub abbreviation: Option<String>,
    pub terrain: i32,
    pub special_level: i32,
    pub default_level: i32,
    pub skills: [Option<String>; 6],
    pub ethnicity: Ethnicity,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct MonDef {
    pub attack_level: i32,
    pub defense: [i32; NUM_ATTACK_TYPES],
    pub number_of_attacks: i32,
    pub hits: i32,
    pub regeneration: i32,
    pub tactics: i32,
    pub stealth: i32,
    pub observation: i32,
    pub special: Option<String>,
    pub special_level: i32,
    pub silver: i32,
    pub spoil_type: i32,
    pub hostile: i32,
    pub number: i32,
    pub name: Option<String>,
    pub abbreviation: Option<String>,
    pub hit_damage: i32,
    pub preferred_terrain: Vec<i32>,
    pub forbidden_terrain: Vec<i32>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct WeaponBonusMalus {
    #[serde(rename = "weapon")]
    pub weapon_abbreviation: Option<String>,
    pub attack_modifier: i32,
    pub defense_modifier: i32,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct WeaponDef {
    pub abbreviation: Option<String>,
    pub flags: i32,
    pub base_skill: Option<String>,
    pub or_skill: Option<String>,
    pub weapon_class: WeaponClass,
    pub attack_type: AttackType,
    pub number_of_attacks: i32,
    pub attack_bonus: i32,
    pub defense_bonus: i32,
    pub mount_bonus: i32,
    pub hit_damage: i32,
    pub bonus_malus: [WeaponBonusMalus; MAX_WEAPON_BONUS_TARGETS],
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct ArmorDef {
    pub abbreviation: Option<String>,
    pub flags: i32,
    #[serde(rename = "from")]
    pub from_value: i32,
    pub saves: [i32; NUM_WEAPON_CLASSES],
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct MountDef {
    pub abbreviation: Option<String>,
    pub skill: Option<String>,
    pub minimum_bonus: i32,
    pub maximum_bonus: i32,
    pub maximum_hampered_bonus: i32,
    pub special: Option<String>,
    pub special_level: i32,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct BattleItemDef {
    pub abbreviation: Option<String>,
    pub flags: i32,
    pub special: Option<String>,
    pub skill_level: i32,
    pub hit_damage: i32,
}

impl Tables {
    pub fn lookup_item(&self, token: &str) -> Option<usize> {
        self.items.iter().position(|item| {
            item.abbreviation.as_deref().is_some_and(|abbreviation| {
                item_alias_matches(token, abbreviation, is_illusion(item))
            })
        })
    }

    pub fn parse_all_items(&self, token: &str) -> Option<usize> {
        self.items
            .iter()
            .position(|item| item_names_match(item, token))
    }

    pub fn parse_item(&self, token: &str) -> Option<usize> {
        self.parse_enabled_item(token)
    }

    pub fn parse_enabled_item(&self, token: &str) -> Option<usize> {
        self.items
            .iter()
            .position(|item| item.flags & ItemFlags::DISABLED == 0 && item_names_match(item, token))
    }

    pub fn parse_giveable_item(&self, token: &str) -> Option<usize> {
        self.items.iter().position(|item| {
            item.flags & (ItemFlags::DISABLED | ItemFlags::CANTGIVE) == 0
                && item_names_match(item, token)
        })
    }

    pub fn parse_transportable_item(&self, token: &str) -> Option<usize> {
        self.items.iter().position(|item| {
            item.flags & (ItemFlags::DISABLED | ItemFlags::CANTGIVE | ItemFlags::NOTRANSPORT) == 0
                && item_names_match(item, token)
        })
    }

    pub fn find_item_index(&self, abbreviation: &str) -> Option<usize> {
        self.items.iter().position(|item| {
            item.abbreviation
                .as_deref()
                .is_some_and(|value| crate::text::is_equal(abbreviation, value))
        })
    }

    pub fn find_item(&self, abbreviation: &str) -> Option<&ItemDef> {
        self.find_item_index(abbreviation)
            .and_then(|index| self.items.get(index))
    }

    pub fn find_race(&self, abbreviation: &str) -> Option<&ManDef> {
        self.races.iter().find(|race| {
            race.abbreviation
                .as_deref()
                .is_some_and(|value| crate::text::is_equal(abbreviation, value))
        })
    }

    pub fn find_monster(&self, abbreviation: &str, illusion: bool) -> Option<&MonDef> {
        let mut tag = String::with_capacity(abbreviation.len() + if illusion { 1 } else { 0 });
        if illusion {
            tag.push('i');
        }
        tag.push_str(abbreviation);
        self.monsters.iter().find(|monster| {
            monster
                .abbreviation
                .as_deref()
                .is_some_and(|value| crate::text::is_equal(&tag, value))
        })
    }

    pub fn find_weapon(&self, abbreviation: &str) -> Option<&WeaponDef> {
        self.weapons.iter().find(|weapon| {
            weapon
                .abbreviation
                .as_deref()
                .is_some_and(|value| crate::text::is_equal(abbreviation, value))
        })
    }

    pub fn find_armor(&self, abbreviation: &str) -> Option<&ArmorDef> {
        self.armor.iter().find(|armor| {
            armor
                .abbreviation
                .as_deref()
                .is_some_and(|value| crate::text::is_equal(abbreviation, value))
        })
    }

    pub fn find_mount(&self, abbreviation: &str) -> Option<&MountDef> {
        self.mounts.iter().find(|mount| {
            mount
                .abbreviation
                .as_deref()
                .is_some_and(|value| crate::text::is_equal(abbreviation, value))
        })
    }

    pub fn find_battle_item(&self, abbreviation: &str) -> Option<&BattleItemDef> {
        self.battle_items.iter().find(|item| {
            item.abbreviation
                .as_deref()
                .is_some_and(|value| crate::text::is_equal(abbreviation, value))
        })
    }

    pub fn find_race_item_index(&self, race_index: usize) -> Option<usize> {
        let race = self.races.get(race_index)?;
        let abbreviation = race.abbreviation.as_deref()?;
        self.items.iter().position(|item| {
            item.item_type & ItemTypeFlags::MAN != 0
                && item
                    .abbreviation
                    .as_deref()
                    .is_some_and(|value| crate::text::is_equal(value, abbreviation))
        })
    }
}

fn is_illusion(item: &ItemDef) -> bool {
    item.item_type & ItemTypeFlags::ILLUSION != 0
}

fn item_alias_matches(token: &str, alias: &str, illusion: bool) -> bool {
    if illusion {
        let mut prefixed = String::with_capacity(alias.len() + 1);
        prefixed.push('i');
        prefixed.push_str(alias);
        crate::text::is_equal(token, &prefixed)
    } else {
        crate::text::is_equal(token, alias)
    }
}

fn item_names_match(item: &ItemDef, token: &str) -> bool {
    let illusion = is_illusion(item);
    [
        item.name.as_deref(),
        item.plural.as_deref(),
        item.abbreviation.as_deref(),
    ]
    .into_iter()
    .flatten()
    .any(|alias| item_alias_matches(token, alias, illusion))
}

pub(super) fn item_families_jsonl(tables: &Tables) -> String {
    let mut output = String::new();

    append_count(&mut output, "items", tables.items.len());
    for (index, item) in tables.items.iter().enumerate() {
        let mut row = JsonLine::new("items", Some(index));
        row.string("name", item.name.as_deref());
        row.string("plural", item.plural.as_deref());
        row.string("abbreviation", item.abbreviation.as_deref());
        row.number("flags", item.flags);
        row.string("production_skill", item.production_skill.as_deref());
        row.number("production_level", item.production_level);
        row.number("production_months", item.production_months);
        row.number("production_output", item.production_output);
        row.array(
            "production_inputs",
            item.production_inputs.len(),
            |out, input| {
                let material = &item.production_inputs[input];
                write!(
                    out,
                    "{{\"item\":{},\"amount\":{}}}",
                    material.item, material.amount
                )
                .expect("writing to String cannot fail");
            },
        );
        row.string("magic_skill", item.magic_skill.as_deref());
        row.number("magic_level", item.magic_level);
        row.number("magic_output", item.magic_output);
        row.array("magic_inputs", item.magic_inputs.len(), |out, input| {
            let material = &item.magic_inputs[input];
            write!(
                out,
                "{{\"item\":{},\"amount\":{}}}",
                material.item, material.amount
            )
            .expect("writing to String cannot fail");
        });
        row.number("weight", item.weight);
        row.number("type", item.item_type);
        row.number("base_price", item.base_price);
        row.number("combat", item.combat);
        row.number("walk", item.walk);
        row.number("ride", item.ride);
        row.number("fly", item.fly);
        row.number("swim", item.swim);
        row.number("speed", item.speed);
        row.number("hitch_item", item.hitch_item);
        row.number("hitch_walk", item.hitch_walk);
        row.number("mult_item", item.mult_item);
        row.number("mult_value", item.mult_value);
        row.number("max_inventory", item.max_inventory);
        row.number("escape", item.escape);
        row.string("escape_skill", item.escape_skill.as_deref());
        row.number("escape_value", item.escape_value);
        row.string("grant_skill", item.grant_skill.as_deref());
        row.array("grant_from", item.grant_from.len(), |out, skill| {
            push_json_string(out, item.grant_from[skill].as_deref());
        });
        row.number("min_grant", item.min_grant);
        row.number("max_grant", item.max_grant);
        output.push_str(&row.finish());
    }

    append_count(&mut output, "races", tables.races.len());
    for (index, race) in tables.races.iter().enumerate() {
        let mut row = JsonLine::new("races", Some(index));
        row.string("abbreviation", race.abbreviation.as_deref());
        row.number("terrain", race.terrain);
        row.number("special_level", race.special_level);
        row.number("default_level", race.default_level);
        row.array("skills", race.skills.len(), |out, skill| {
            push_json_string(out, race.skills[skill].as_deref());
        });
        row.number("ethnicity", race.ethnicity as i32);
        output.push_str(&row.finish());
    }

    append_count(&mut output, "monsters", tables.monsters.len());
    for (index, monster) in tables.monsters.iter().enumerate() {
        let mut row = JsonLine::new("monsters", Some(index));
        row.number("attack_level", monster.attack_level);
        row.array("defense", monster.defense.len(), |out, attack| {
            write!(out, "{}", monster.defense[attack]).expect("writing to String cannot fail");
        });
        row.number("number_of_attacks", monster.number_of_attacks);
        row.number("hits", monster.hits);
        row.number("regeneration", monster.regeneration);
        row.number("tactics", monster.tactics);
        row.number("stealth", monster.stealth);
        row.number("observation", monster.observation);
        row.string("special", monster.special.as_deref());
        row.number("special_level", monster.special_level);
        row.number("silver", monster.silver);
        row.number("spoil_type", monster.spoil_type);
        row.number("hostile", monster.hostile);
        row.number("number", monster.number);
        row.string("name", monster.name.as_deref());
        row.string("abbreviation", monster.abbreviation.as_deref());
        row.number("hit_damage", monster.hit_damage);
        row.array(
            "preferred_terrain",
            monster.preferred_terrain.len(),
            |out, terrain| {
                write!(out, "{}", monster.preferred_terrain[terrain])
                    .expect("writing to String cannot fail");
            },
        );
        row.array(
            "forbidden_terrain",
            monster.forbidden_terrain.len(),
            |out, terrain| {
                write!(out, "{}", monster.forbidden_terrain[terrain])
                    .expect("writing to String cannot fail");
            },
        );
        output.push_str(&row.finish());
    }

    append_count(&mut output, "weapons", tables.weapons.len());
    for (index, weapon) in tables.weapons.iter().enumerate() {
        let mut row = JsonLine::new("weapons", Some(index));
        row.string("abbreviation", weapon.abbreviation.as_deref());
        row.number("flags", weapon.flags);
        row.string("base_skill", weapon.base_skill.as_deref());
        row.string("or_skill", weapon.or_skill.as_deref());
        row.number("weapon_class", weapon.weapon_class as i32);
        row.number("attack_type", weapon.attack_type as i32);
        row.number("number_of_attacks", weapon.number_of_attacks);
        row.number("attack_bonus", weapon.attack_bonus);
        row.number("defense_bonus", weapon.defense_bonus);
        row.number("mount_bonus", weapon.mount_bonus);
        row.number("hit_damage", weapon.hit_damage);
        row.array("bonus_malus", weapon.bonus_malus.len(), |out, modifier| {
            let value = &weapon.bonus_malus[modifier];
            out.push_str("{\"weapon\":");
            push_json_string(out, value.weapon_abbreviation.as_deref());
            write!(
                out,
                ",\"attack_modifier\":{},\"defense_modifier\":{}}}",
                value.attack_modifier, value.defense_modifier
            )
            .expect("writing to String cannot fail");
        });
        output.push_str(&row.finish());
    }

    append_count(&mut output, "armor", tables.armor.len());
    for (index, armor) in tables.armor.iter().enumerate() {
        let mut row = JsonLine::new("armor", Some(index));
        row.string("abbreviation", armor.abbreviation.as_deref());
        row.number("flags", armor.flags);
        row.number("from", armor.from_value);
        row.array("saves", armor.saves.len(), |out, save| {
            write!(out, "{}", armor.saves[save]).expect("writing to String cannot fail");
        });
        output.push_str(&row.finish());
    }

    append_count(&mut output, "mounts", tables.mounts.len());
    for (index, mount) in tables.mounts.iter().enumerate() {
        let mut row = JsonLine::new("mounts", Some(index));
        row.string("abbreviation", mount.abbreviation.as_deref());
        row.string("skill", mount.skill.as_deref());
        row.number("minimum_bonus", mount.minimum_bonus);
        row.number("maximum_bonus", mount.maximum_bonus);
        row.number("maximum_hampered_bonus", mount.maximum_hampered_bonus);
        row.string("special", mount.special.as_deref());
        row.number("special_level", mount.special_level);
        output.push_str(&row.finish());
    }

    append_count(&mut output, "battle_items", tables.battle_items.len());
    for (index, item) in tables.battle_items.iter().enumerate() {
        let mut row = JsonLine::new("battle_items", Some(index));
        row.string("abbreviation", item.abbreviation.as_deref());
        row.number("flags", item.flags);
        row.string("special", item.special.as_deref());
        row.number("skill_level", item.skill_level);
        row.number("hit_damage", item.hit_damage);
        output.push_str(&row.finish());
    }

    output
}

fn append_count(output: &mut String, table: &str, count: usize) {
    let mut row = JsonLine::new("counts", None);
    row.string("name", Some(table));
    row.number("value", count);
    output.push_str(&row.finish());
}

struct JsonLine {
    output: String,
}

impl JsonLine {
    fn new(table: &str, index: Option<usize>) -> Self {
        let mut output = String::from("{\"table\":");
        push_json_string(&mut output, Some(table));
        if let Some(index) = index {
            write!(output, ",\"index\":{index}").expect("writing to String cannot fail");
        }
        Self { output }
    }

    fn string(&mut self, key: &str, value: Option<&str>) {
        self.field(key);
        push_json_string(&mut self.output, value);
    }

    fn number(&mut self, key: &str, value: impl std::fmt::Display) {
        self.field(key);
        write!(self.output, "{value}").expect("writing to String cannot fail");
    }

    fn array(
        &mut self,
        key: &str,
        count: usize,
        mut write_element: impl FnMut(&mut String, usize),
    ) {
        self.field(key);
        self.output.push('[');
        for index in 0..count {
            if index != 0 {
                self.output.push(',');
            }
            write_element(&mut self.output, index);
        }
        self.output.push(']');
    }

    fn field(&mut self, key: &str) {
        self.output.push(',');
        push_json_string(&mut self.output, Some(key));
        self.output.push(':');
    }

    fn finish(mut self) -> String {
        self.output.push_str("}\n");
        self.output
    }
}

fn push_json_string(output: &mut String, value: Option<&str>) {
    let Some(value) = value else {
        output.push_str("null");
        return;
    };

    output.push('"');
    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\u{08}' => output.push_str("\\b"),
            '\u{0c}' => output.push_str("\\f"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            character if character < '\u{20}' => {
                write!(output, "\\u00{:02x}", character as u32)
                    .expect("writing to String cannot fail");
            }
            character => output.push(character),
        }
    }
    output.push('"');
}
