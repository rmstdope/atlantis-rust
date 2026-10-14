mod neworigins;
mod standard;

use crate::{
    defs::{
        modify::TableModifier,
        skills::{attribute_item_flags, attribute_modifier_types, range_flags},
        terrains::{
            TerrainId, R_DESERT, R_FOREST, R_JUNGLE, R_MOUNTAIN, R_OCEAN, R_PLAIN, R_SWAMP,
            R_TUNDRA,
        },
        GameDefs, ItemId, RulesetTables,
    },
    rng::Rng,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldNameStyle {
    Standard,
    NewOrigins,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FactionLimits {
    pub mages: &'static [i32],
    pub apprentices: &'static [i32],
    pub taxes: &'static [i32],
    pub trades: &'static [i32],
    pub quartermasters: &'static [i32],
    pub tacticians: &'static [i32],
    pub martial: &'static [i32],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StartingFactionSetup {
    pub unclaimed_per_turn: i32,
    pub studies: Vec<(&'static str, i32)>,
    pub set_flags: i32,
    pub clear_flags: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VirtualMonster {
    pub object_type: i32,
    pub item: ItemId,
    pub name: &'static str,
    pub count: i32,
}

pub trait Ruleset: Sync {
    fn game_defs(&self) -> &'static GameDefs;
    fn faction_limits(&self) -> &'static FactionLimits;
    fn faction_types(&self) -> &'static [&'static str];
    fn apply_rule_table_modifications(&self, _modifier: &mut TableModifier<'_>) {}

    fn starting_faction_setup(&self, turn_number: i32) -> StartingFactionSetup {
        let mut studies = vec![("PATT", 30), ("SPIR", 30), ("GATE", 30)];
        if turn_number >= 25 {
            studies.extend([("PATT", 60), ("SPIR", 60), ("FORC", 90), ("COMB", 30)]);
        }
        StartingFactionSetup {
            unclaimed_per_turn: 50,
            studies,
            set_flags: 0,
            clear_flags: 0,
        }
    }

    fn virtual_monster(&self) -> Option<VirtualMonster> {
        None
    }

    /// C++: `standard/world.cpp` and `neworigins/world.cpp` `GetRegType`.
    fn surface_terrain_from_roll(&self, latitude: i32, roll: i32) -> TerrainId {
        match latitude {
            0 if roll < 24 => R_TUNDRA,
            0 if roll < 32 => R_MOUNTAIN,
            0 if roll < 40 => R_FOREST,
            0 => R_PLAIN,
            1 if roll < 16 => R_PLAIN,
            1 if roll < 40 => R_FOREST,
            1 if roll < 48 => R_MOUNTAIN,
            1 => R_SWAMP,
            2 if roll < 20 => R_PLAIN,
            2 if roll < 28 => R_FOREST,
            2 if roll < 36 => R_MOUNTAIN,
            2 if roll < 44 => R_SWAMP,
            2 if roll < 52 => R_JUNGLE,
            2 => R_DESERT,
            3 if roll < 16 => R_PLAIN,
            3 if roll < 24 => R_MOUNTAIN,
            3 if roll < 36 => R_SWAMP,
            3 if roll < 48 => R_JUNGLE,
            3 => R_DESERT,
            _ => R_OCEAN,
        }
    }

    /// C++: `ARegionList::GetRegType`; the surface consumes one `getrandom(64)` draw.
    fn get_reg_type(&self, latitude: i32, rng: &mut Rng) -> TerrainId {
        self.surface_terrain_from_roll(latitude, rng.get_random(64))
    }

    fn world_name_style(&self) -> WorldNameStyle {
        WorldNameStyle::Standard
    }

    fn unnamed_region_name(&self) -> &'static str {
        "Unnamed"
    }

    /// C++: `ARegionList::GetLevelXScale`.
    fn level_x_scale(&self, level: i32) -> i32 {
        level_x_scale(self.game_defs(), level)
    }

    /// C++: `ARegionList::GetLevelYScale`.
    fn level_y_scale(&self, level: i32) -> i32 {
        level_y_scale(self.game_defs(), level)
    }

    fn name(&self) -> &'static str {
        self.game_defs().ruleset_name
    }
}

fn level_x_scale(defs: &GameDefs, level: i32) -> i32 {
    if level < 2 {
        return 1;
    }
    if defs.underworld_levels == 1 && defs.underdeep_levels == 0 {
        return 2;
    }
    if level >= 2 && level < defs.underworld_levels + 2 {
        return if level == 2 { 1 } else { 2 };
    }
    if level >= defs.underworld_levels + 2
        && level < defs.underworld_levels + defs.underdeep_levels + 2
    {
        return if level == defs.underworld_levels + 2 {
            2
        } else {
            4
        };
    }
    1
}

fn level_y_scale(defs: &GameDefs, level: i32) -> i32 {
    if level < 2 {
        return 1;
    }
    if defs.underworld_levels == 1 && defs.underdeep_levels == 0 {
        return 2;
    }
    if level >= 2 && level < defs.underworld_levels + 2 {
        if level == defs.underworld_levels + 1 {
            return 4;
        }
        return 2;
    }
    if level >= defs.underworld_levels + 2
        && level < defs.underworld_levels + defs.underdeep_levels + 2
    {
        return 4;
    }
    1
}

impl dyn Ruleset + '_ {
    pub fn tables(&self) -> RulesetTables {
        let mut tables = RulesetTables::default();
        {
            let mut modifier = TableModifier::new(&mut tables);
            apply_common_table_prefix(self.game_defs(), &mut modifier);
            self.apply_rule_table_modifications(&mut modifier);
            apply_common_table_suffix(self.game_defs(), &mut modifier);
        }
        tables
    }

    pub fn from_name(name: &str) -> Option<&'static Self> {
        if name.eq_ignore_ascii_case("standard") || name.eq_ignore_ascii_case("Wyreth") {
            Some(&standard::STANDARD)
        } else if name.eq_ignore_ascii_case("neworigins") || name.eq_ignore_ascii_case("NewOrigins")
        {
            Some(&neworigins::NEW_ORIGINS)
        } else {
            None
        }
    }
}

fn apply_common_table_prefix(defs: &GameDefs, modifier: &mut TableModifier<'_>) {
    if defs.apprentices_exist != 0 {
        let skill = modifier.skill_id("MANI");
        modifier.enable_skill(skill);
    }
    if defs.gates_exist == 0 {
        let skill = modifier.skill_id("GATE");
        modifier.disable_skill(skill);
    }
    if defs.full_trueseeing_bonus != 0 {
        modifier.modify_attribute_modifier(
            "observation",
            1,
            attribute_item_flags::SKILL,
            Some("TRUE"),
            attribute_modifier_types::UNIT_LEVEL,
            1,
        );
    }
    if defs.improved_amts != 0 {
        modifier.modify_attribute_modifier(
            "observation",
            2,
            attribute_item_flags::ITEM,
            Some("AMTS"),
            attribute_modifier_types::CONSTANT,
            3,
        );
    }
    if defs.full_invis_on_self != 0 {
        modifier.modify_attribute_modifier(
            "stealth",
            3,
            attribute_item_flags::SKILL,
            Some("INVI"),
            attribute_modifier_types::UNIT_LEVEL,
            1,
        );
    }
}

fn apply_common_table_suffix(defs: &GameDefs, modifier: &mut TableModifier<'_>) {
    if defs.easier_underworld != 0 {
        for range in [
            "rng_teleport",
            "rng_farsight",
            "rng_clearsky",
            "rng_weather",
        ] {
            modifier.modify_range_flags(range, range_flags::CROSS_LEVELS);
        }
    }

    if defs.transport & crate::defs::enums::flags::ALLOW_TRANSPORT != 0 {
        let skill = modifier.skill_id("quartermaster");
        modifier.enable_skill(skill);
        modifier.enable_object(crate::defs::objects::O_CARAVANSERAI as i32);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DownstreamRuleset;

    impl Ruleset for DownstreamRuleset {
        fn game_defs(&self) -> &'static GameDefs {
            <dyn Ruleset>::from_name("standard")
                .expect("standard ruleset")
                .game_defs()
        }

        fn faction_limits(&self) -> &'static FactionLimits {
            static LIMITS: FactionLimits = FactionLimits {
                mages: &[],
                apprentices: &[],
                taxes: &[],
                trades: &[],
                quartermasters: &[],
                tacticians: &[],
                martial: &[],
            };
            &LIMITS
        }

        fn faction_types(&self) -> &'static [&'static str] {
            &[]
        }
    }

    #[test]
    fn ruleset_implementations_can_omit_table_modification_hook() {
        let ruleset: &dyn Ruleset = &DownstreamRuleset;
        let tables = ruleset.tables();
        assert_eq!(tables.item_families.items.len(), crate::defs::ITEM_COUNT);
    }
}
