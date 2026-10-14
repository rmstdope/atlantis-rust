//! C++: `modify.cpp` `Game::Modify*` table mutation helpers.

use super::{
    items::{
        AttackType, ItemDef, ItemFlags, ItemTypeFlags, WeaponClass, NUM_ATTACK_TYPES,
        NUM_WEAPON_CLASSES,
    },
    objects::{object_flags, ObjectDef},
    skills::{
        attribute_modifier_types, find_attribute, find_effect, find_range, find_special,
        range_classes, skill_flags, SpecialDamage,
    },
    terrains::{TerrainDef, TerrainProduct},
    RulesetTables,
};
use crate::text::is_equal;

const WOOD_OR_STONE: i32 = -2;

pub struct TableModifier<'a> {
    tables: &'a mut RulesetTables,
}

impl<'a> TableModifier<'a> {
    pub fn new(tables: &'a mut RulesetTables) -> Self {
        Self { tables }
    }

    fn skill_index(&self, key: &str) -> Option<usize> {
        super::skills::lookup_skill(&self.tables.skills.skills, key)
    }

    pub fn skill_id(&self, key: &str) -> i32 {
        self.tables
            .skills
            .skills
            .iter()
            .position(|skill| {
                skill.name.is_some_and(|value| is_equal(value, key))
                    || skill.abbreviation.is_some_and(|value| is_equal(value, key))
            })
            .unwrap_or_else(|| panic!("ruleset modifier references missing skill {key:?}"))
            as i32
    }

    fn race_index(&self, key: &str) -> Option<usize> {
        self.tables.item_families.races.iter().position(|race| {
            race.abbreviation
                .as_deref()
                .is_some_and(|value| is_equal(value, key))
        })
    }

    fn monster_index(&self, key: &str) -> Option<usize> {
        self.tables
            .item_families
            .monsters
            .iter()
            .position(|monster| {
                monster
                    .abbreviation
                    .as_deref()
                    .is_some_and(|value| is_equal(value, key))
            })
    }

    fn weapon_index(&self, key: &str) -> Option<usize> {
        self.tables.item_families.weapons.iter().position(|weapon| {
            weapon
                .abbreviation
                .as_deref()
                .is_some_and(|value| is_equal(value, key))
        })
    }

    fn armor_index(&self, key: &str) -> Option<usize> {
        self.tables.item_families.armor.iter().position(|armor| {
            armor
                .abbreviation
                .as_deref()
                .is_some_and(|value| is_equal(value, key))
        })
    }

    fn mount_index(&self, key: &str) -> Option<usize> {
        self.tables.item_families.mounts.iter().position(|mount| {
            mount
                .abbreviation
                .as_deref()
                .is_some_and(|value| is_equal(value, key))
        })
    }

    fn battle_item_index(&self, key: &str) -> Option<usize> {
        self.tables
            .item_families
            .battle_items
            .iter()
            .position(|item| {
                item.abbreviation
                    .as_deref()
                    .is_some_and(|value| is_equal(value, key))
            })
    }

    fn object(&mut self, id: i32) -> Option<&mut ObjectDef> {
        checked_index(id, self.tables.objects.len())
            .and_then(|index| self.tables.objects.get_mut(index))
    }

    fn item(&mut self, id: i32) -> Option<&mut ItemDef> {
        checked_index(id, self.tables.item_families.items.len())
            .and_then(|index| self.tables.item_families.items.get_mut(index))
    }

    fn terrain(&mut self, id: i32) -> Option<&mut TerrainDef> {
        checked_index(id, self.tables.terrains.len())
            .and_then(|index| self.tables.terrains.get_mut(index))
    }

    /// C++: `modify.cpp` `Game::EnableSkill`.
    pub fn enable_skill(&mut self, skill: i32) {
        if let Some(skill) = checked_index(skill, self.tables.skills.skills.len())
            .and_then(|index| self.tables.skills.skills.get_mut(index))
        {
            skill.flags &= !skill_flags::DISABLED;
        }
    }

    /// C++: `modify.cpp` `Game::DisableSkill`.
    pub fn disable_skill(&mut self, skill: i32) {
        if let Some(skill) = checked_index(skill, self.tables.skills.skills.len())
            .and_then(|index| self.tables.skills.skills.get_mut(index))
        {
            skill.flags |= skill_flags::DISABLED;
        }
    }

    /// C++: `modify.cpp` `Game::ModifySkillDependancy`.
    pub fn modify_skill_dependency(
        &mut self,
        skill: i32,
        slot: i32,
        dependency: Option<&str>,
        level: i32,
    ) {
        let Some(skill_index) = checked_index(skill, self.tables.skills.skills.len()) else {
            return;
        };
        let Some(slot) = checked_index(
            slot,
            self.tables.skills.skills[skill_index].dependencies.len(),
        ) else {
            return;
        };
        let dependency = match dependency {
            Some(key) => match self.skill_index(key) {
                Some(index) => Some(index),
                None => return,
            },
            None => None,
        };
        if level < 0 {
            return;
        }
        let row = &mut self.tables.skills.skills[skill_index].dependencies[slot];
        row.skill = dependency;
        row.level = level;
    }

    /// C++: `modify.cpp` `Game::ModifySkillFlags`.
    pub fn modify_skill_flags(&mut self, skill: i32, flags: i32) {
        if let Some(skill) = checked_index(skill, self.tables.skills.skills.len())
            .and_then(|index| self.tables.skills.skills.get_mut(index))
        {
            skill.flags = flags;
        }
    }

    /// C++: `modify.cpp` `Game::ModifySkillCost`.
    pub fn modify_skill_cost(&mut self, skill: i32, cost: i32) {
        if cost < 0 {
            return;
        }
        if let Some(skill) = checked_index(skill, self.tables.skills.skills.len())
            .and_then(|index| self.tables.skills.skills.get_mut(index))
        {
            skill.cost = cost;
        }
    }

    /// C++: `modify.cpp` `Game::ModifySkillSpecial`.
    pub fn modify_skill_special(&mut self, skill: i32, special: Option<&str>) {
        let Some(skill_index) = checked_index(skill, self.tables.skills.skills.len()) else {
            return;
        };
        let special = match special {
            Some(key) => match find_special(&self.tables.skills.specials, key) {
                Some(index) => Some(index),
                None => return,
            },
            None => None,
        };
        self.tables.skills.skills[skill_index].special = special;
    }

    /// C++: `modify.cpp` `Game::ModifySkillRange`.
    pub fn modify_skill_range(&mut self, skill: i32, range: Option<&str>) {
        let Some(skill_index) = checked_index(skill, self.tables.skills.skills.len()) else {
            return;
        };
        let range = match range {
            Some(key) => match find_range(&self.tables.skills.ranges, key) {
                Some(index) => Some(index),
                None => return,
            },
            None => None,
        };
        self.tables.skills.skills[skill_index].range = range;
    }

    /// C++: `modify.cpp` `Game::EnableItem`.
    pub fn enable_item(&mut self, item: i32) {
        if let Some(item) = self.item(item) {
            item.flags &= !ItemFlags::DISABLED;
        }
    }

    /// C++: `modify.cpp` `Game::DisableItem`.
    pub fn disable_item(&mut self, item: i32) {
        if let Some(item) = self.item(item) {
            item.flags |= ItemFlags::DISABLED;
        }
    }

    /// C++: `modify.cpp` `Game::ModifyItemName`.
    pub fn modify_item_name(&mut self, item: i32, name: Option<&str>, plural: Option<&str>) {
        if let Some(item) = self.item(item) {
            item.name = name.map(str::to_owned);
            item.plural = plural.map(str::to_owned);
        }
    }

    /// C++: `modify.cpp` `Game::ModifyItemFlags`.
    pub fn modify_item_flags(&mut self, item: i32, flags: i32) {
        if let Some(item) = self.item(item) {
            item.flags = flags;
        }
    }

    /// C++: `modify.cpp` `Game::ModifyItemType`.
    pub fn modify_item_type(&mut self, item: i32, item_type: i32) {
        if let Some(item) = self.item(item) {
            item.item_type = item_type;
        }
    }

    /// C++: `modify.cpp` `Game::ModifyItemWeight`.
    pub fn modify_item_weight(&mut self, item: i32, weight: i32) {
        if let Some(item) = self.item(item) {
            item.weight = weight.max(0);
        }
    }

    /// C++: `modify.cpp` `Game::ModifyItemBasePrice`.
    pub fn modify_item_base_price(&mut self, item: i32, price: i32) {
        if let Some(item) = self.item(item) {
            item.base_price = price.max(0);
        }
    }

    /// C++: `modify.cpp` `Game::ModifyItemCapacities`.
    pub fn modify_item_capacities(&mut self, item: i32, walk: i32, ride: i32, fly: i32, swim: i32) {
        if let Some(item) = self.item(item) {
            item.walk = walk.max(0);
            item.ride = ride.max(0);
            item.fly = fly.max(0);
            item.swim = swim.max(0);
        }
    }

    /// C++: `modify.cpp` `Game::ModifyItemSpeed`.
    pub fn modify_item_speed(&mut self, item: i32, speed: i32) {
        if let Some(item) = self.item(item) {
            item.speed = speed.max(0);
        }
    }

    /// C++: `modify.cpp` `Game::ModifyItemProductionBooster`.
    pub fn modify_item_production_booster(&mut self, item: i32, multiplier_item: i32, bonus: i32) {
        if multiplier_item < -1 || multiplier_item >= self.tables.item_families.items.len() as i32 {
            return;
        }
        if let Some(item) = self.item(item) {
            item.mult_item = multiplier_item;
            item.mult_value = bonus;
        }
    }

    /// C++: `modify.cpp` `Game::ModifyItemHitch`.
    pub fn modify_item_hitch(&mut self, item: i32, hitch_item: i32, capacity: i32) {
        if hitch_item < -1
            || hitch_item >= self.tables.item_families.items.len() as i32
            || capacity < 0
        {
            return;
        }
        if let Some(item) = self.item(item) {
            item.hitch_item = hitch_item;
            item.hitch_walk = capacity;
        }
    }

    /// C++: `modify.cpp` `Game::ModifyItemProductionSkill`.
    pub fn modify_item_production_skill(&mut self, item: i32, skill: Option<&str>, level: i32) {
        if skill.is_some_and(|key| self.skill_index(key).is_none()) {
            return;
        }
        if let Some(item) = self.item(item) {
            item.production_skill = skill.map(str::to_owned);
            item.production_level = level;
        }
    }

    /// C++: `modify.cpp` `Game::ModifyItemProductionOutput`.
    pub fn modify_item_production_output(&mut self, item: i32, months: i32, count: i32) {
        if let Some(item) = self.item(item) {
            item.production_months = months.max(0);
            item.production_output = count.max(0);
        }
    }

    /// C++: `modify.cpp` `Game::ModifyItemProductionInput`.
    pub fn modify_item_production_input(&mut self, item: i32, slot: i32, input: i32, amount: i32) {
        let Some(item_index) = checked_index(item, self.tables.item_families.items.len()) else {
            return;
        };
        let Some(slot) = checked_index(
            slot,
            self.tables.item_families.items[item_index]
                .production_inputs
                .len(),
        ) else {
            return;
        };
        if input < -1 || input >= self.tables.item_families.items.len() as i32 {
            return;
        }
        let material = &mut self.tables.item_families.items[item_index].production_inputs[slot];
        material.item = input;
        material.amount = amount.max(0);
    }

    /// C++: `modify.cpp` `Game::ModifyItemMagicSkill`.
    pub fn modify_item_magic_skill(&mut self, item: i32, skill: Option<&str>, level: i32) {
        if skill.is_some_and(|key| self.skill_index(key).is_none()) {
            return;
        }
        if let Some(item) = self.item(item) {
            item.magic_skill = skill.map(str::to_owned);
            item.magic_level = level;
        }
    }

    /// C++: `modify.cpp` `Game::ModifyItemMagicOutput`.
    pub fn modify_item_magic_output(&mut self, item: i32, count: i32) {
        if let Some(item) = self.item(item) {
            item.magic_output = count.max(0);
        }
    }

    /// C++: `modify.cpp` `Game::ModifyItemMagicInput`.
    pub fn modify_item_magic_input(&mut self, item: i32, slot: i32, input: i32, amount: i32) {
        let Some(item_index) = checked_index(item, self.tables.item_families.items.len()) else {
            return;
        };
        let Some(slot) = checked_index(
            slot,
            self.tables.item_families.items[item_index]
                .magic_inputs
                .len(),
        ) else {
            return;
        };
        if input < -1 || input >= self.tables.item_families.items.len() as i32 {
            return;
        }
        let material = &mut self.tables.item_families.items[item_index].magic_inputs[slot];
        material.item = input;
        material.amount = amount.max(0);
    }

    /// C++: `modify.cpp` `Game::ModifyItemEscape`.
    pub fn modify_item_escape(&mut self, item: i32, escape: i32, skill: Option<&str>, value: i32) {
        if let Some(item) = self.item(item) {
            item.escape = escape;
            item.escape_skill = skill.map(str::to_owned);
            item.escape_value = value;
        }
    }

    /// C++: `modify.cpp` `Game::ModifyRaceSkillLevels`.
    pub fn modify_race_skill_levels(&mut self, race: &str, special: i32, default: i32) {
        let Some(index) = self.race_index(race) else {
            return;
        };
        let row = &mut self.tables.item_families.races[index];
        row.special_level = special.max(0);
        row.default_level = default.max(0);
    }

    /// C++: `modify.cpp` `Game::ModifyRaceSkills`.
    pub fn modify_race_skills(&mut self, race: &str, slot: i32, skill: Option<&str>) {
        let Some(index) = self.race_index(race) else {
            return;
        };
        let Some(slot) = checked_index(slot, self.tables.item_families.races[index].skills.len())
        else {
            return;
        };
        if skill.is_some_and(|key| self.skill_index(key).is_none()) {
            return;
        }
        self.tables.item_families.races[index].skills[slot] = skill.map(str::to_owned);
    }

    /// C++: `modify.cpp` `Game::ModifyMonsterAttackLevel`.
    pub fn modify_monster_attack_level(&mut self, monster: &str, level: i32) {
        let Some(index) = self.monster_index(monster) else {
            return;
        };
        if level >= 0 {
            self.tables.item_families.monsters[index].attack_level = level;
        }
    }

    /// C++: `modify.cpp` `Game::ModifyMonsterDefense`.
    pub fn modify_monster_defense(&mut self, monster: &str, defense_type: i32, level: i32) {
        let Some(index) = self.monster_index(monster) else {
            return;
        };
        if let Some(defense_type) = checked_index(defense_type, NUM_ATTACK_TYPES) {
            self.tables.item_families.monsters[index].defense[defense_type] = level;
        }
    }

    /// C++: `modify.cpp` `Game::ModifyMonsterAttacksAndHits`.
    pub fn modify_monster_attacks_and_hits(
        &mut self,
        monster: &str,
        attacks: i32,
        hits: i32,
        regeneration: i32,
        hit_damage: i32,
    ) {
        let Some(index) = self.monster_index(monster) else {
            return;
        };
        if attacks < 0 || hits < 0 || regeneration < 0 || hit_damage < 0 {
            return;
        }
        let row = &mut self.tables.item_families.monsters[index];
        row.number_of_attacks = attacks;
        row.hits = hits;
        row.regeneration = regeneration;
        row.hit_damage = hit_damage;
    }

    /// C++: `modify.cpp` `Game::ModifyMonsterSkills`.
    pub fn modify_monster_skills(
        &mut self,
        monster: &str,
        tactics: i32,
        stealth: i32,
        observation: i32,
    ) {
        let Some(index) = self.monster_index(monster) else {
            return;
        };
        if tactics < 0 || stealth < 0 || observation < 0 {
            return;
        }
        let row = &mut self.tables.item_families.monsters[index];
        row.tactics = tactics;
        row.stealth = stealth;
        row.observation = observation;
    }

    /// C++: `modify.cpp` `Game::ModifyMonsterSpecial`.
    pub fn modify_monster_special(&mut self, monster: &str, special: Option<&str>, level: i32) {
        let Some(index) = self.monster_index(monster) else {
            return;
        };
        if special.is_some_and(|key| find_special(&self.tables.skills.specials, key).is_none())
            || level < 0
        {
            return;
        }
        self.tables.item_families.monsters[index].special = special.map(str::to_owned);
        self.tables.item_families.monsters[index].special_level = level;
    }

    /// C++: `modify.cpp` `Game::ModifyMonsterSpoils`.
    pub fn modify_monster_spoils(&mut self, monster: &str, silver: i32, spoil_type: i32) {
        let Some(index) = self.monster_index(monster) else {
            return;
        };
        if silver < 0 || spoil_type < -1 {
            return;
        }
        let row = &mut self.tables.item_families.monsters[index];
        row.silver = silver;
        row.spoil_type = spoil_type;
    }

    /// C++: `modify.cpp` `Game::ModifyMonsterThreat`.
    pub fn modify_monster_threat(&mut self, monster: &str, number: i32, hostile_chance: i32) {
        let Some(index) = self.monster_index(monster) else {
            return;
        };
        if number < 0 || !(0..=100).contains(&hostile_chance) {
            return;
        }
        let row = &mut self.tables.item_families.monsters[index];
        row.hostile = hostile_chance;
        row.number = number;
    }

    /// C++: `modify.cpp` `Game::ModifyWeaponSkills`.
    pub fn modify_weapon_skills(
        &mut self,
        weapon: &str,
        base_skill: Option<&str>,
        alternate_skill: Option<&str>,
    ) {
        let Some(index) = self.weapon_index(weapon) else {
            return;
        };
        if base_skill.is_some_and(|key| self.skill_index(key).is_none())
            || alternate_skill.is_some_and(|key| self.skill_index(key).is_none())
        {
            return;
        }
        let row = &mut self.tables.item_families.weapons[index];
        row.base_skill = base_skill.map(str::to_owned);
        row.or_skill = alternate_skill.map(str::to_owned);
    }

    /// C++: `modify.cpp` `Game::ModifyWeaponFlags`.
    pub fn modify_weapon_flags(&mut self, weapon: &str, flags: i32) {
        if let Some(index) = self.weapon_index(weapon) {
            self.tables.item_families.weapons[index].flags = flags;
        }
    }

    /// C++: `modify.cpp` `Game::ModifyWeaponAttack`.
    pub fn modify_weapon_attack(
        &mut self,
        weapon: &str,
        weapon_class: i32,
        attack_type: i32,
        number_of_attacks: i32,
        hit_damage: i32,
    ) {
        let Some(index) = self.weapon_index(weapon) else {
            return;
        };
        if !(0..NUM_ATTACK_TYPES as i32).contains(&attack_type) {
            return;
        }
        let (Ok(weapon_class), Ok(attack_type)) = (
            WeaponClass::try_from(weapon_class),
            AttackType::try_from(attack_type),
        ) else {
            return;
        };
        let row = &mut self.tables.item_families.weapons[index];
        row.weapon_class = weapon_class;
        row.attack_type = attack_type;
        row.number_of_attacks = number_of_attacks;
        row.hit_damage = hit_damage;
    }

    /// C++: `modify.cpp` `Game::ModifyWeaponBonuses`.
    pub fn modify_weapon_bonuses(
        &mut self,
        weapon: &str,
        attack: i32,
        defense: i32,
        vs_mount: i32,
    ) {
        if let Some(index) = self.weapon_index(weapon) {
            let row = &mut self.tables.item_families.weapons[index];
            row.attack_bonus = attack;
            row.defense_bonus = defense;
            row.mount_bonus = vs_mount;
        }
    }

    /// C++: `modify.cpp` `Game::ModifyWeaponBonusMalus`.
    pub fn modify_weapon_bonus_malus(
        &mut self,
        weapon: &str,
        slot: i32,
        target_abbreviation: Option<&str>,
        attack_modifier: i32,
        defense_modifier: i32,
    ) {
        let Some(index) = self.weapon_index(weapon) else {
            return;
        };
        let Some(slot) = checked_index(
            slot,
            self.tables.item_families.weapons[index].bonus_malus.len(),
        ) else {
            return;
        };
        let modifier = &mut self.tables.item_families.weapons[index].bonus_malus[slot];
        modifier.weapon_abbreviation = target_abbreviation.map(str::to_owned);
        modifier.attack_modifier = attack_modifier;
        modifier.defense_modifier = defense_modifier;
    }

    /// C++: `modify.cpp` `Game::ModifyArmorFlags`.
    pub fn modify_armor_flags(&mut self, armor: &str, flags: i32) {
        if let Some(index) = self.armor_index(armor) {
            self.tables.item_families.armor[index].flags = flags;
        }
    }

    /// C++: `modify.cpp` `Game::ModifyArmorSaveFrom`.
    pub fn modify_armor_save_from(&mut self, armor: &str, from: i32) {
        let Some(index) = self.armor_index(armor) else {
            return;
        };
        if from >= 0 {
            self.tables.item_families.armor[index].from_value = from;
        }
    }

    /// C++: `modify.cpp` `Game::ModifyArmorSaveValue`.
    pub fn modify_armor_save_value(&mut self, armor: &str, weapon_class: i32, value: i32) {
        let Some(index) = self.armor_index(armor) else {
            return;
        };
        let Some(weapon_class) = checked_index(weapon_class, NUM_WEAPON_CLASSES) else {
            return;
        };
        let row = &mut self.tables.item_families.armor[index];
        if value >= 0 && value <= row.from_value {
            row.saves[weapon_class] = value;
        }
    }

    /// C++: `modify.cpp` `Game::ModifyMountSkill`.
    pub fn modify_mount_skill(&mut self, mount: &str, skill: Option<&str>) {
        let Some(index) = self.mount_index(mount) else {
            return;
        };
        if skill.is_some_and(|key| self.skill_index(key).is_none()) {
            return;
        }
        self.tables.item_families.mounts[index].skill = skill.map(str::to_owned);
    }

    /// C++: `modify.cpp` `Game::ModifyMountBonuses`.
    pub fn modify_mount_bonuses(&mut self, mount: &str, minimum: i32, maximum: i32, hampered: i32) {
        let Some(index) = self.mount_index(mount) else {
            return;
        };
        if minimum < 0 || maximum < 0 || hampered < minimum {
            return;
        }
        let row = &mut self.tables.item_families.mounts[index];
        row.minimum_bonus = minimum;
        row.maximum_bonus = maximum;
        row.maximum_hampered_bonus = hampered;
    }

    /// C++: `modify.cpp` `Game::ModifyMountSpecial`.
    pub fn modify_mount_special(&mut self, mount: &str, special: Option<&str>, level: i32) {
        let Some(index) = self.mount_index(mount) else {
            return;
        };
        if special.is_some_and(|key| find_special(&self.tables.skills.specials, key).is_none())
            || level < 0
        {
            return;
        }
        let row = &mut self.tables.item_families.mounts[index];
        row.special = special.map(str::to_owned);
        row.special_level = level;
    }

    /// C++: `modify.cpp` `Game::EnableObject`.
    pub fn enable_object(&mut self, object: i32) {
        if let Some(object) = self.object(object) {
            object.flags &= !object_flags::DISABLED;
        }
    }

    /// C++: `modify.cpp` `Game::DisableObject`.
    pub fn disable_object(&mut self, object: i32) {
        if let Some(object) = self.object(object) {
            object.flags |= object_flags::DISABLED;
        }
    }

    /// C++: `modify.cpp` `Game::ModifyObjectFlags`.
    pub fn modify_object_flags(&mut self, object: i32, flags: i32) {
        if let Some(object) = self.object(object) {
            object.flags = flags;
        }
    }

    /// C++: `modify.cpp` `Game::ModifyObjectDecay`.
    pub fn modify_object_decay(
        &mut self,
        object: i32,
        maximum_maintenance: i32,
        maximum_monthly_decay: i32,
        maintenance_factor: i32,
    ) {
        if maximum_monthly_decay > maximum_maintenance
            || maximum_maintenance < 0
            || maximum_monthly_decay < 0
            || maintenance_factor < 0
        {
            return;
        }
        if let Some(object) = self.object(object) {
            object.maximum_maintenance = maximum_maintenance;
            object.maximum_monthly_decay = maximum_monthly_decay;
            object.maintenance_factor = maintenance_factor;
        }
    }

    /// C++: `modify.cpp` `Game::ModifyObjectProduction`.
    pub fn modify_object_production(&mut self, object: i32, item: i32) {
        if item < -1 || item >= self.tables.item_families.items.len() as i32 {
            return;
        }
        if let Some(object) = self.object(object) {
            object.production_aided = item;
        }
    }

    /// C++: `modify.cpp` `Game::ModifyObjectMonster`.
    pub fn modify_object_monster(&mut self, object: i32, monster: i32) {
        if monster < -1 || monster >= self.tables.item_families.items.len() as i32 {
            return;
        }
        if let Some(object) = self.object(object) {
            object.monster = monster;
        }
    }

    /// C++: `modify.cpp` `Game::ModifyObjectConstruction`.
    pub fn modify_object_construction(
        &mut self,
        object: i32,
        item: i32,
        count: i32,
        skill: Option<&'static str>,
        level: i32,
    ) {
        if (item < -1 && item != WOOD_OR_STONE)
            || item >= self.tables.item_families.items.len() as i32
            || count < 0
            || skill.is_some_and(|key| self.skill_index(key).is_none())
            || level < 0
        {
            return;
        }
        if let Some(object) = self.object(object) {
            object.item = item;
            object.cost = count;
            object.skill = skill;
            object.level = level;
        }
    }

    /// C++: `modify.cpp` `Game::ModifyObjectManpower`.
    pub fn modify_object_manpower(
        &mut self,
        object: i32,
        protection: i32,
        capacity: i32,
        sailors: i32,
        maximum_mages: i32,
    ) {
        if protection < 0 || capacity < 0 || sailors < 0 || maximum_mages < 0 {
            return;
        }
        if let Some(object) = self.object(object) {
            object.protection = protection;
            object.capacity = capacity;
            object.sailors = sailors;
            object.maximum_mages = maximum_mages;
        }
    }

    /// C++: `modify.cpp` `Game::ModifyObjectDefence`.
    #[allow(clippy::too_many_arguments)]
    pub fn modify_object_defence(
        &mut self,
        object: i32,
        combat: i32,
        energy: i32,
        spirit: i32,
        weather: i32,
        riding: i32,
        ranged: i32,
    ) {
        if let Some(object) = self.object(object) {
            object.defense = [combat, energy, spirit, weather, riding, ranged];
        }
    }

    /// C++: `modify.cpp` `Game::ModifyObjectName`.
    pub fn modify_object_name(&mut self, object: i32, name: &'static str) {
        if let Some(object) = self.object(object) {
            object.name = name;
        }
    }

    /// C++: `modify.cpp` `Game::ClearTerrainRaces`.
    pub fn clear_terrain_races(&mut self, terrain: i32) {
        if let Some(terrain) = self.terrain(terrain) {
            terrain.races.fill(-1);
            terrain.coastal_races.fill(-1);
        }
    }

    /// C++: `modify.cpp` `Game::ModifyTerrainRace`.
    pub fn modify_terrain_race(&mut self, terrain: i32, slot: i32, race: i32) {
        let Some(terrain_index) = checked_index(terrain, self.tables.terrains.len()) else {
            return;
        };
        let Some(slot) = checked_index(slot, self.tables.terrains[terrain_index].races.len())
        else {
            return;
        };
        self.tables.terrains[terrain_index].races[slot] =
            valid_race_item(&self.tables.item_families.items, race);
    }

    /// C++: `modify.cpp` `Game::ModifyTerrainCoastRace`.
    pub fn modify_terrain_coast_race(&mut self, terrain: i32, slot: i32, race: i32) {
        let Some(terrain_index) = checked_index(terrain, self.tables.terrains.len()) else {
            return;
        };
        let Some(slot) = checked_index(
            slot,
            self.tables.terrains[terrain_index].coastal_races.len(),
        ) else {
            return;
        };
        self.tables.terrains[terrain_index].coastal_races[slot] =
            valid_race_item(&self.tables.item_families.items, race);
    }

    /// C++: `modify.cpp` `Game::ClearTerrainItems`.
    pub fn clear_terrain_items(&mut self, terrain: i32) {
        if let Some(terrain) = self.terrain(terrain) {
            terrain.products.fill(TerrainProduct {
                item: -1,
                chance: 0,
                amount: 0,
            });
        }
    }

    /// C++: `modify.cpp` `Game::ModifyTerrainItems`.
    pub fn modify_terrain_items(
        &mut self,
        terrain: i32,
        slot: i32,
        item: i32,
        chance: i32,
        amount: i32,
    ) {
        let Some(terrain_index) = checked_index(terrain, self.tables.terrains.len()) else {
            return;
        };
        let Some(slot) = checked_index(slot, self.tables.terrains[terrain_index].products.len())
        else {
            return;
        };
        let product = &mut self.tables.terrains[terrain_index].products[slot];
        product.item = valid_item_or_none(&self.tables.item_families.items, item);
        product.chance = if (0..=100).contains(&chance) {
            chance
        } else {
            0
        };
        product.amount = amount.max(0);
    }

    /// C++: `modify.cpp` `Game::ModifyTerrainWMons`.
    pub fn modify_terrain_wandering_monsters(
        &mut self,
        terrain: i32,
        frequency: i32,
        small: i32,
        big: i32,
        humanoid: i32,
    ) {
        let small = valid_item_or_none(&self.tables.item_families.items, small);
        let big = valid_item_or_none(&self.tables.item_families.items, big);
        let humanoid = valid_item_or_none(&self.tables.item_families.items, humanoid);
        if let Some(terrain) = self.terrain(terrain) {
            terrain.wandering_monster_frequency = frequency.max(0);
            terrain.small_monster = small;
            terrain.big_monster = big;
            terrain.humanoid = humanoid;
        }
    }

    /// C++: `modify.cpp` `Game::ModifyTerrainLairChance`.
    pub fn modify_terrain_lair_chance(&mut self, terrain: i32, chance: i32) {
        if let Some(terrain) = self.terrain(terrain) {
            terrain.lair_chance = if (0..=100).contains(&chance) {
                chance
            } else {
                0
            };
        }
    }

    /// C++: `modify.cpp` `Game::ModifyTerrainLair`.
    pub fn modify_terrain_lair(&mut self, terrain: i32, slot: i32, lair: i32) {
        let Some(terrain_index) = checked_index(terrain, self.tables.terrains.len()) else {
            return;
        };
        let Some(slot) = checked_index(slot, self.tables.terrains[terrain_index].lairs.len())
        else {
            return;
        };
        let lair = if lair < -1 || lair >= self.tables.objects.len() as i32 {
            -1
        } else {
            lair
        };
        self.tables.terrains[terrain_index].lairs[slot] = lair;
    }

    /// C++: `modify.cpp` `Game::ModifyTerrainEconomy`.
    pub fn modify_terrain_economy(
        &mut self,
        terrain: i32,
        population: i32,
        wages: i32,
        economy: i32,
        movement_points: i32,
    ) {
        if let Some(terrain) = self.terrain(terrain) {
            terrain.population = population.max(0);
            terrain.wages = wages.max(0);
            terrain.economy = economy.max(0);
            terrain.movement_points = movement_points.max(1);
        }
    }

    /// C++: `modify.cpp` `Game::ModifyTerrainFlags`.
    pub fn modify_terrain_flags(&mut self, terrain: i32, flags: i32) {
        if let Some(terrain) = self.terrain(terrain) {
            terrain.flags = flags;
        }
    }

    /// C++: `modify.cpp` `Game::ModifyBattleItemFlags`.
    pub fn modify_battle_item_flags(&mut self, item: &str, flags: i32) {
        if let Some(index) = self.battle_item_index(item) {
            self.tables.item_families.battle_items[index].flags = flags;
        }
    }

    /// C++: `modify.cpp` `Game::ModifyBattleItemSpecial`.
    pub fn modify_battle_item_special(&mut self, item: &str, special: Option<&str>, level: i32) {
        let Some(index) = self.battle_item_index(item) else {
            return;
        };
        if special.is_some_and(|key| find_special(&self.tables.skills.specials, key).is_none())
            || level < 0
        {
            return;
        }
        let row = &mut self.tables.item_families.battle_items[index];
        row.special = special.map(str::to_owned);
        row.skill_level = level;
    }

    /// C++: `modify.cpp` `Game::ModifySpecialTargetFlags`.
    pub fn modify_special_target_flags(&mut self, special: &str, flags: i32) {
        if let Some(index) = find_special(&self.tables.skills.specials, special) {
            self.tables.skills.specials[index].target_flags = flags;
        }
    }

    /// C++: `modify.cpp` `Game::ModifySpecialTargetObjects`.
    pub fn modify_special_target_objects(&mut self, special: &str, slot: i32, object: i32) {
        let Some(index) = find_special(&self.tables.skills.specials, special) else {
            return;
        };
        if !(0..=3).contains(&slot)
            || (object != -1 && object < 1)
            || object >= self.tables.objects.len() as i32
        {
            return;
        }
        if let Some(target) = self.tables.skills.specials[index]
            .buildings
            .get_mut(slot as usize)
        {
            *target = object;
        }
    }

    /// C++: `modify.cpp` `Game::ModifySpecialTargetItems`.
    pub fn modify_special_target_items(&mut self, special: &str, slot: i32, item: i32) {
        let Some(index) = find_special(&self.tables.skills.specials, special) else {
            return;
        };
        if !(0..=7).contains(&slot)
            || item < -1
            || item >= self.tables.item_families.items.len() as i32
        {
            return;
        }
        if let Some(target) = self.tables.skills.specials[index]
            .targets
            .get_mut(slot as usize)
        {
            *target = item;
        }
    }

    /// C++: `modify.cpp` `Game::ModifySpecialTargetEffects`.
    pub fn modify_special_target_effects(
        &mut self,
        special: &str,
        slot: i32,
        effect: Option<&str>,
    ) {
        let Some(index) = find_special(&self.tables.skills.specials, special) else {
            return;
        };
        let effect_index = match effect {
            Some(key) => match find_effect(&self.tables.skills.effects, key) {
                Some(index) => Some(index),
                None => return,
            },
            None => None,
        };
        if let Some(target) = checked_index(slot, self.tables.skills.specials[index].effects.len())
            .and_then(|slot| self.tables.skills.specials[index].effects.get_mut(slot))
        {
            *target = effect_index;
        }
    }

    /// C++: `modify.cpp` `Game::ModifySpecialEffectFlags`.
    pub fn modify_special_effect_flags(&mut self, special: &str, flags: i32) {
        if let Some(index) = find_special(&self.tables.skills.specials, special) {
            self.tables.skills.specials[index].effect_flags = flags;
        }
    }

    /// C++: `modify.cpp` `Game::ModifySpecialShields`.
    pub fn modify_special_shields(&mut self, special: &str, slot: i32, kind: i32) {
        let Some(index) = find_special(&self.tables.skills.specials, special) else {
            return;
        };
        if !(-1..=NUM_ATTACK_TYPES as i32).contains(&kind) {
            return;
        }
        if let Some(shield) = checked_index(slot, self.tables.skills.specials[index].shields.len())
            .and_then(|slot| self.tables.skills.specials[index].shields.get_mut(slot))
        {
            *shield = kind;
        }
    }

    /// C++: `modify.cpp` `Game::ModifySpecialDefenseMods`.
    pub fn modify_special_defense_mods(&mut self, special: &str, slot: i32, kind: i32, value: i32) {
        let Some(index) = find_special(&self.tables.skills.specials, special) else {
            return;
        };
        if !(-1..=NUM_ATTACK_TYPES as i32).contains(&kind) {
            return;
        }
        if let Some(defense) =
            checked_index(slot, self.tables.skills.specials[index].defenses.len())
                .and_then(|slot| self.tables.skills.specials[index].defenses.get_mut(slot))
        {
            defense.kind = kind;
            defense.value = value;
        }
    }

    /// C++: `modify.cpp` `Game::ModifySpecialDamage`.
    #[allow(clippy::too_many_arguments)]
    pub fn modify_special_damage(
        &mut self,
        special: &str,
        slot: i32,
        kind: i32,
        minimum: i32,
        value: i32,
        flags: i32,
        class: i32,
        effect: Option<&str>,
        hit_damage: i32,
    ) {
        let Some(index) = find_special(&self.tables.skills.specials, special) else {
            return;
        };
        if !(-1..=NUM_ATTACK_TYPES as i32).contains(&kind)
            || !(-1..NUM_WEAPON_CLASSES as i32).contains(&class)
            || minimum < 0
        {
            return;
        }
        let effect = match effect {
            Some(key) => match find_effect(&self.tables.skills.effects, key) {
                Some(index) => Some(index),
                None => return,
            },
            None => None,
        };
        let Some(damage) = checked_index(slot, self.tables.skills.specials[index].damage.len())
            .and_then(|slot| self.tables.skills.specials[index].damage.get_mut(slot))
        else {
            return;
        };
        *damage = SpecialDamage {
            kind,
            minimum,
            value,
            flags,
            class,
            effect,
            hit_damage,
        };
    }

    /// C++: `modify.cpp` `Game::ModifyEffectFlags`.
    pub fn modify_effect_flags(&mut self, effect: &str, flags: i32) {
        if let Some(index) = find_effect(&self.tables.skills.effects, effect) {
            self.tables.skills.effects[index].flags = flags;
        }
    }

    /// C++: `modify.cpp` `Game::ModifyEffectAttackMod`.
    pub fn modify_effect_attack_mod(&mut self, effect: &str, value: i32) {
        if let Some(index) = find_effect(&self.tables.skills.effects, effect) {
            self.tables.skills.effects[index].attack_value = value;
        }
    }

    /// C++: `modify.cpp` `Game::ModifyEffectDefenseMod`.
    pub fn modify_effect_defense_mod(&mut self, effect: &str, slot: i32, kind: i32, value: i32) {
        let Some(index) = find_effect(&self.tables.skills.effects, effect) else {
            return;
        };
        if !(0..=NUM_ATTACK_TYPES as i32).contains(&kind) {
            return;
        }
        if let Some(defense) = checked_index(
            slot,
            self.tables.skills.effects[index].defense_modifiers.len(),
        )
        .and_then(|slot| {
            self.tables.skills.effects[index]
                .defense_modifiers
                .get_mut(slot)
        }) {
            defense.kind = kind;
            defense.value = value;
        }
    }

    /// C++: `modify.cpp` `Game::ModifyEffectCancelEffect`.
    pub fn modify_effect_cancel_effect(&mut self, effect: &str, cancel: Option<&str>) {
        let Some(index) = find_effect(&self.tables.skills.effects, effect) else {
            return;
        };
        let cancel = match cancel {
            Some(key) => match find_effect(&self.tables.skills.effects, key) {
                Some(index) => Some(index),
                None => return,
            },
            None => None,
        };
        self.tables.skills.effects[index].cancel_effect = cancel;
    }

    /// C++: `modify.cpp` `Game::ModifyRangeFlags`.
    pub fn modify_range_flags(&mut self, range: &str, flags: i32) {
        if let Some(index) = find_range(&self.tables.skills.ranges, range) {
            self.tables.skills.ranges[index].flags = flags;
        }
    }

    /// C++: `modify.cpp` `Game::ModifyRangeClass`.
    pub fn modify_range_class(&mut self, range: &str, range_class: i32) {
        if !(0..range_classes::COUNT).contains(&range_class) {
            return;
        }
        if let Some(index) = find_range(&self.tables.skills.ranges, range) {
            self.tables.skills.ranges[index].range_class = range_class;
        }
    }

    /// C++: `modify.cpp` `Game::ModifyRangeMultiplier`.
    pub fn modify_range_multiplier(&mut self, range: &str, multiplier: i32) {
        if multiplier < 1 {
            return;
        }
        if let Some(index) = find_range(&self.tables.skills.ranges, range) {
            self.tables.skills.ranges[index].range_multiplier = multiplier;
        }
    }

    /// C++: `modify.cpp` `Game::ModifyRangeLevelPenalty`.
    pub fn modify_range_level_penalty(&mut self, range: &str, penalty: i32) {
        if penalty < 0 {
            return;
        }
        if let Some(index) = find_range(&self.tables.skills.ranges, range) {
            self.tables.skills.ranges[index].cross_level_penalty = penalty;
        }
    }

    /// C++: `modify.cpp` `Game::ModifyAttribMod`.
    pub fn modify_attribute_modifier(
        &mut self,
        attribute: &str,
        slot: i32,
        flags: i32,
        identifier: Option<&'static str>,
        kind: i32,
        value: i32,
    ) {
        let Some(index) = find_attribute(&self.tables.skills.attribute_modifiers, attribute) else {
            return;
        };
        if !(0..=5).contains(&slot)
            || identifier.is_none()
            || !(0..attribute_modifier_types::COUNT).contains(&kind)
        {
            return;
        }
        if let Some(modifier) = checked_index(
            slot,
            self.tables.skills.attribute_modifiers[index]
                .modifiers
                .len(),
        )
        .and_then(|slot| {
            self.tables.skills.attribute_modifiers[index]
                .modifiers
                .get_mut(slot)
        }) {
            modifier.flags = flags;
            modifier.identifier = identifier;
            modifier.kind = kind;
            modifier.value = value;
        }
    }

    /// C++: `modify.cpp` `Game::ModifyHealing`.
    pub fn modify_healing(&mut self, level: i32, patients: i32, success: i32) {
        if !(1..=5).contains(&level) {
            return;
        }
        self.tables.skills.healing[level as usize].number = patients;
        self.tables.skills.healing[level as usize].rate = success;
    }
}

fn checked_index(index: i32, len: usize) -> Option<usize> {
    usize::try_from(index).ok().filter(|&index| index < len)
}

fn valid_item_or_none(items: &[ItemDef], item: i32) -> i32 {
    if item < -1 || item >= items.len() as i32 {
        -1
    } else {
        item
    }
}

fn valid_race_item(items: &[ItemDef], item: i32) -> i32 {
    let item = valid_item_or_none(items, item);
    if item >= 0 && items[item as usize].item_type & ItemTypeFlags::MAN == 0 {
        -1
    } else {
        item
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::defs::{objects::O_ROADN, terrains::R_PLAIN};

    #[test]
    fn item_and_skill_modifiers_clamp_values_and_reject_invalid_references() {
        let mut tables = RulesetTables::default();
        let item = tables
            .item_families
            .find_item_index("PICK")
            .expect("pick item");
        let original_item = tables.item_families.items[item].clone();
        let original_skill = tables.skills.skills[0];

        {
            let mut modifier = TableModifier::new(&mut tables);
            modifier.modify_item_weight(item as i32, -5);
            modifier.modify_item_base_price(item as i32, -5);
            modifier.modify_item_production_output(item as i32, -1, -2);
            modifier.modify_item_production_input(item as i32, 0, -1, -5);
            modifier.modify_item_production_input(item as i32, 1, -2, 4);
            modifier.modify_item_production_input(item as i32, 4, item as i32, 4);
            modifier.modify_item_production_skill(item as i32, Some("missing-skill"), 4);
            modifier.modify_skill_cost(0, -1);
            modifier.modify_skill_dependency(0, 0, Some("missing-skill"), 4);
            modifier.modify_skill_dependency(0, 3, None, 4);
        }

        let item = &tables.item_families.items[item];
        assert_eq!(item.weight, 0);
        assert_eq!(item.base_price, 0);
        assert_eq!(item.production_months, 0);
        assert_eq!(item.production_output, 0);
        assert_eq!(item.production_inputs[0].item, -1);
        assert_eq!(item.production_inputs[0].amount, 0);
        assert_eq!(
            item.production_inputs[1],
            original_item.production_inputs[1]
        );
        assert_eq!(
            item.production_inputs[3],
            original_item.production_inputs[3]
        );
        assert_eq!(item.production_skill, original_item.production_skill);
        assert_eq!(tables.skills.skills[0].cost, original_skill.cost);
        assert_eq!(
            tables.skills.skills[0].dependencies[0],
            original_skill.dependencies[0]
        );
    }

    #[test]
    fn creature_and_equipment_modifiers_validate_indexes_and_enum_bounds() {
        let mut tables = RulesetTables::default();
        let race_index = tables
            .item_families
            .races
            .iter()
            .position(|race| race.abbreviation.is_some())
            .expect("named race");
        let race = tables.item_families.races[race_index]
            .abbreviation
            .as_deref()
            .expect("race abbreviation")
            .to_owned();
        let monster_index = tables
            .item_families
            .monsters
            .iter()
            .position(|monster| monster.abbreviation.is_some())
            .expect("named monster");
        let monster = tables.item_families.monsters[monster_index]
            .abbreviation
            .as_deref()
            .expect("monster abbreviation")
            .to_owned();
        let weapon_index = tables
            .item_families
            .weapons
            .iter()
            .position(|weapon| weapon.abbreviation.is_some())
            .expect("named weapon");
        let weapon = tables.item_families.weapons[weapon_index]
            .abbreviation
            .as_deref()
            .expect("weapon abbreviation")
            .to_owned();
        let armor_index = tables
            .item_families
            .armor
            .iter()
            .position(|armor| armor.abbreviation.is_some())
            .expect("named armor");
        let armor = tables.item_families.armor[armor_index]
            .abbreviation
            .as_deref()
            .expect("armor abbreviation")
            .to_owned();
        let mount_index = tables
            .item_families
            .mounts
            .iter()
            .position(|mount| mount.abbreviation.is_some())
            .expect("named mount");
        let mount = tables.item_families.mounts[mount_index]
            .abbreviation
            .as_deref()
            .expect("mount abbreviation")
            .to_owned();
        let original_monster = tables.item_families.monsters[monster_index].clone();
        let original_weapon = tables.item_families.weapons[weapon_index].clone();
        let original_armor = tables.item_families.armor[armor_index].clone();
        let original_mount = tables.item_families.mounts[mount_index].clone();

        {
            let mut modifier = TableModifier::new(&mut tables);
            modifier.modify_race_skill_levels(&race, -1, -2);
            modifier.modify_monster_attacks_and_hits(&monster, -1, 2, 3, 4);
            modifier.modify_weapon_attack(&weapon, 0, 6, 2, 10);
            modifier.modify_armor_save_value(&armor, 8, 4);
            modifier.modify_mount_bonuses(&mount, -1, 2, 3);
        }

        assert_eq!(tables.item_families.races[race_index].special_level, 0);
        assert_eq!(tables.item_families.races[race_index].default_level, 0);
        assert_eq!(
            tables.item_families.monsters[monster_index],
            original_monster
        );
        assert_eq!(tables.item_families.weapons[weapon_index], original_weapon);
        assert_eq!(tables.item_families.armor[armor_index], original_armor);
        assert_eq!(tables.item_families.mounts[mount_index], original_mount);

        {
            let mut modifier = TableModifier::new(&mut tables);
            modifier.modify_weapon_attack(&weapon, 0, 5, 2, 10);
        }
        assert_eq!(
            tables.item_families.weapons[weapon_index].attack_type,
            AttackType::Ranged
        );
        assert_eq!(
            tables.item_families.weapons[weapon_index].number_of_attacks,
            2
        );
    }

    #[test]
    fn object_and_terrain_modifiers_guard_bounds_and_clamp_values() {
        let mut tables = RulesetTables::default();
        let original_object = tables.objects[O_ROADN];
        {
            let mut modifier = TableModifier::new(&mut tables);
            modifier.modify_object_decay(O_ROADN as i32, 1, 2, 3);
            modifier.modify_object_construction(O_ROADN as i32, -3, 4, Some("missing"), 1);
            modifier.modify_object_flags(-1, 0);
        }
        assert_eq!(tables.objects[O_ROADN], original_object);

        {
            let mut modifier = TableModifier::new(&mut tables);
            modifier.modify_terrain_economy(R_PLAIN as i32, -1, -2, -3, 0);
            modifier.modify_terrain_items(R_PLAIN as i32, 0, i32::MAX, 101, -1);
            modifier.modify_terrain_race(
                R_PLAIN as i32,
                0,
                crate::defs::ItemId::Axe.index() as i32,
            );
        }

        let plain = &tables.terrains[R_PLAIN];
        assert_eq!(plain.population, 0);
        assert_eq!(plain.wages, 0);
        assert_eq!(plain.economy, 0);
        assert_eq!(plain.movement_points, 1);
        assert_eq!(
            plain.products[0],
            TerrainProduct {
                item: -1,
                chance: 0,
                amount: 0
            }
        );
        assert_eq!(plain.races[0], -1);
    }

    #[test]
    fn magic_range_attribute_and_healing_modifiers_reject_invalid_slots() {
        let mut tables = RulesetTables::default();
        let special_index = tables
            .skills
            .specials
            .iter()
            .position(|special| special.key.is_some())
            .expect("named special");
        let special = tables.skills.specials[special_index]
            .key
            .expect("special key")
            .to_owned();
        let effect_index = tables
            .skills
            .effects
            .iter()
            .position(|effect| effect.name.is_some())
            .expect("named effect");
        let effect = tables.skills.effects[effect_index]
            .name
            .expect("effect name")
            .to_owned();
        let range_index = tables
            .skills
            .ranges
            .iter()
            .position(|range| range.key.is_some())
            .expect("named range");
        let range = tables.skills.ranges[range_index]
            .key
            .expect("range key")
            .to_owned();
        let attribute_index = tables
            .skills
            .attribute_modifiers
            .iter()
            .position(|attribute| attribute.key.is_some())
            .expect("named attribute");
        let attribute = tables.skills.attribute_modifiers[attribute_index]
            .key
            .expect("attribute key")
            .to_owned();
        let original_special = tables.skills.specials[special_index];
        let original_range = tables.skills.ranges[range_index];
        let original_attribute = tables.skills.attribute_modifiers[attribute_index].modifiers[0];
        let original_healing = tables.skills.healing[1];

        {
            let mut modifier = TableModifier::new(&mut tables);
            modifier.modify_special_target_objects(&special, -1, 0);
            modifier.modify_special_target_effects(&special, 0, Some("missing-effect"));
            modifier.modify_range_class(&range, range_classes::COUNT);
            modifier.modify_attribute_modifier(
                &attribute,
                0,
                1,
                Some("missing"),
                attribute_modifier_types::COUNT,
                1,
            );
            modifier.modify_healing(0, 1, 1);
        }
        assert_eq!(tables.skills.specials[special_index], original_special);
        assert_eq!(tables.skills.ranges[range_index], original_range);
        assert_eq!(
            tables.skills.attribute_modifiers[attribute_index].modifiers[0],
            original_attribute
        );
        assert_eq!(tables.skills.healing[1], original_healing);

        {
            let mut modifier = TableModifier::new(&mut tables);
            modifier.modify_effect_attack_mod(&effect, 7);
            modifier.modify_range_class(&range, range_classes::LEVEL_SQUARED);
            modifier.modify_attribute_modifier(
                &attribute,
                0,
                1,
                Some("valid"),
                attribute_modifier_types::CONSTANT,
                2,
            );
            modifier.modify_healing(1, 8, 50);
        }
        assert_eq!(tables.skills.effects[effect_index].attack_value, 7);
        assert_eq!(
            tables.skills.ranges[range_index].range_class,
            range_classes::LEVEL_SQUARED
        );
        assert_eq!(
            tables.skills.attribute_modifiers[attribute_index].modifiers[0].value,
            2
        );
        assert_eq!(tables.skills.healing[1].number, 8);
        assert_eq!(tables.skills.healing[1].rate, 50);
    }
}
