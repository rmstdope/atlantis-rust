pub mod enums;
pub mod gamedefs;
pub mod objects;
pub mod skills;
pub mod strings;
pub mod terrains;

pub use gamedefs::GameDefs;
pub use objects::{
    lookup_object, parse_object, BuildTarget, ObjectDef, ObjectId, ObjectItemLookup, NOBJECTS,
    OBJECT_DEFS,
};
pub use terrains::{TerrainDef, TerrainId, TerrainProduct, R_NUM, TERRAIN_DEFS};
mod items;

pub use items::{
    ArmorDef, ArmorFlags, AttackType, BattleItemDef, BattleItemFlags, EscapeFlags, Ethnicity,
    ItemDef, ItemFlags, ItemId, ItemTypeFlags, ManDef, Materials, MonDef, MountDef,
    WeaponBonusMalus, WeaponClass, WeaponDef, WeaponFlags, ITEM_COUNT, MAX_WEAPON_BONUS_TARGETS,
    NITEMS, NUM_ATTACK_TYPES, NUM_WEAPON_CLASSES,
};

use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tables {
    pub items: Vec<ItemDef>,
    pub races: Vec<ManDef>,
    pub monsters: Vec<MonDef>,
    pub weapons: Vec<WeaponDef>,
    pub armor: Vec<ArmorDef>,
    pub mounts: Vec<MountDef>,
    pub battle_items: Vec<BattleItemDef>,
}

impl Default for Tables {
    fn default() -> Self {
        Self::from_item_family_jsonl(include_str!("item-families-default.jsonl"))
            .expect("embedded C++ item-family defaults are valid")
    }
}

#[derive(Deserialize)]
struct CountRow {
    name: String,
    value: usize,
}

#[derive(Deserialize)]
struct IndexedRow<T> {
    index: usize,
    #[serde(flatten)]
    value: T,
}

impl Tables {
    fn from_item_family_jsonl(input: &str) -> Result<Self, String> {
        let mut tables = Self {
            items: Vec::new(),
            races: Vec::new(),
            monsters: Vec::new(),
            weapons: Vec::new(),
            armor: Vec::new(),
            mounts: Vec::new(),
            battle_items: Vec::new(),
        };
        let mut counts = [None; 7];

        for (line_index, line) in input.lines().enumerate() {
            let value: serde_json::Value = serde_json::from_str(line)
                .map_err(|error| format!("line {}: {error}", line_index + 1))?;
            let table = value
                .get("table")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| format!("line {} has no table name", line_index + 1))?;

            match table {
                "counts" => {
                    let count: CountRow = serde_json::from_value(value)
                        .map_err(|error| format!("line {}: {error}", line_index + 1))?;
                    let slot = count_slot(&count.name)
                        .ok_or_else(|| format!("unknown item-family table: {}", count.name))?;
                    if counts[slot].replace(count.value).is_some() {
                        return Err(format!("duplicate count for {}", count.name));
                    }
                }
                "items" => push_indexed::<ItemDef>(&mut tables.items, value, line_index)?,
                "races" => push_indexed::<ManDef>(&mut tables.races, value, line_index)?,
                "monsters" => push_indexed::<MonDef>(&mut tables.monsters, value, line_index)?,
                "weapons" => push_indexed::<WeaponDef>(&mut tables.weapons, value, line_index)?,
                "armor" => push_indexed::<ArmorDef>(&mut tables.armor, value, line_index)?,
                "mounts" => push_indexed::<MountDef>(&mut tables.mounts, value, line_index)?,
                "battle_items" => {
                    push_indexed::<BattleItemDef>(&mut tables.battle_items, value, line_index)?
                }
                other => return Err(format!("unexpected item-family row: {other}")),
            }
        }

        for (slot, name) in [
            "items",
            "races",
            "monsters",
            "weapons",
            "armor",
            "mounts",
            "battle_items",
        ]
        .into_iter()
        .enumerate()
        {
            let expected = counts[slot].ok_or_else(|| format!("missing count for {name}"))?;
            let actual = tables.table_len(slot);
            if expected != actual {
                return Err(format!(
                    "{name} count is {actual}, but the snapshot declares {expected}"
                ));
            }
        }

        Ok(tables)
    }

    fn table_len(&self, slot: usize) -> usize {
        match slot {
            0 => self.items.len(),
            1 => self.races.len(),
            2 => self.monsters.len(),
            3 => self.weapons.len(),
            4 => self.armor.len(),
            5 => self.mounts.len(),
            6 => self.battle_items.len(),
            _ => unreachable!("all table slots are enumerated"),
        }
    }

    /// Return the canonical source-ordered JSONL for these seven item-family tables.
    pub fn item_families_jsonl(&self) -> String {
        items::item_families_jsonl(self)
    }
}

fn count_slot(name: &str) -> Option<usize> {
    match name {
        "items" => Some(0),
        "races" => Some(1),
        "monsters" => Some(2),
        "weapons" => Some(3),
        "armor" => Some(4),
        "mounts" => Some(5),
        "battle_items" => Some(6),
        _ => None,
    }
}

fn push_indexed<T>(
    rows: &mut Vec<T>,
    value: serde_json::Value,
    line_index: usize,
) -> Result<(), String>
where
    T: for<'de> Deserialize<'de>,
{
    let row: IndexedRow<T> = serde_json::from_value(value)
        .map_err(|error| format!("line {}: {error}", line_index + 1))?;
    if row.index != rows.len() {
        return Err(format!(
            "line {} has index {}, expected {}",
            line_index + 1,
            row.index,
            rows.len()
        ));
    }
    rows.push(row.value);
    Ok(())
}
