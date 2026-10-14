mod neworigins;
mod standard;

use crate::defs::{
    modify::TableModifier,
    skills::{attribute_item_flags, attribute_modifier_types, range_flags},
    GameDefs, RulesetTables,
};

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

pub trait Ruleset: Sync {
    fn game_defs(&self) -> &'static GameDefs;
    fn faction_limits(&self) -> &'static FactionLimits;
    fn faction_types(&self) -> &'static [&'static str];
    fn apply_rule_table_modifications(&self, _modifier: &mut TableModifier<'_>) {}

    fn name(&self) -> &'static str {
        self.game_defs().ruleset_name
    }
}

impl dyn Ruleset {
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
