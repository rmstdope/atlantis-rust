//! C++: `gamedata.cpp` skill-family tables and `skills.cpp` lookup functions.

use std::fmt::Write as _;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SkillDependency {
    pub skill: Option<&'static str>,
    pub level: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SkillDef {
    pub name: Option<&'static str>,
    pub abbreviation: Option<&'static str>,
    pub cost: i32,
    pub flags: i32,
    pub special: Option<&'static str>,
    pub range: Option<&'static str>,
    pub dependencies: [SkillDependency; 3],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HealingDef {
    pub number: i32,
    pub rate: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DefenseModifier {
    pub kind: i32,
    pub value: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpecialDamage {
    pub kind: i32,
    pub minimum: i32,
    pub value: i32,
    pub flags: i32,
    pub class: i32,
    pub effect: Option<&'static str>,
    pub hit_damage: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SpecialDef {
    pub key: Option<&'static str>,
    pub name: Option<&'static str>,
    pub target_flags: i32,
    pub buildings: [i32; 5],
    pub targets: [i32; 7],
    pub effects: [Option<&'static str>; 3],
    pub effect_flags: i32,
    pub shields: [i32; 4],
    pub defenses: [DefenseModifier; 4],
    pub shield_description: Option<&'static str>,
    pub damage: [SpecialDamage; 4],
    pub spell_description: Option<&'static str>,
    pub secondary_spell_description: Option<&'static str>,
    pub spell_target: Option<&'static str>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EffectDef {
    pub name: Option<&'static str>,
    pub attack_value: i32,
    pub defense_modifiers: [DefenseModifier; 4],
    pub cancel_effect: Option<&'static str>,
    pub flags: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RangeDef {
    pub key: Option<&'static str>,
    pub flags: i32,
    pub range_class: i32,
    pub range_multiplier: i32,
    pub cross_level_penalty: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AttributeModifier {
    pub flags: i32,
    pub identifier: Option<&'static str>,
    pub kind: i32,
    pub value: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AttributeModifierDef {
    pub key: Option<&'static str>,
    pub flags: i32,
    pub modifiers: [AttributeModifier; 5],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ShowDef {
    pub skill: i32,
    pub level: i32,
    pub description: Option<&'static str>,
}

/// C++: `gamedata.cpp` `sd` (`SkillDefs`) table order.
pub const SKILL_DEFS: &[SkillDef] = &[
    SkillDef {
        name: Some("mining"),
        abbreviation: Some("MINI"),
        cost: 10,
        flags: 0,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("lumberjack"),
        abbreviation: Some("LUMB"),
        cost: 10,
        flags: 0,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("quarrying"),
        abbreviation: Some("QUAR"),
        cost: 10,
        flags: 0,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("hunting"),
        abbreviation: Some("HUNT"),
        cost: 10,
        flags: 0,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("fishing"),
        abbreviation: Some("FISH"),
        cost: 10,
        flags: 0,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("herb lore"),
        abbreviation: Some("HERB"),
        cost: 10,
        flags: 0,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("horse training"),
        abbreviation: Some("HORS"),
        cost: 10,
        flags: 0,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("weaponsmith"),
        abbreviation: Some("WEAP"),
        cost: 10,
        flags: 0,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("armorer"),
        abbreviation: Some("ARMO"),
        cost: 10,
        flags: 0,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("carpenter"),
        abbreviation: Some("CARP"),
        cost: 10,
        flags: 0,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("building"),
        abbreviation: Some("BUIL"),
        cost: 10,
        flags: 0,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("shipbuilding"),
        abbreviation: Some("SHIP"),
        cost: 10,
        flags: 0,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("entertainment"),
        abbreviation: Some("ENTE"),
        cost: 10,
        flags: 0,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("tactics"),
        abbreviation: Some("TACT"),
        cost: 200,
        flags: 128,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("combat"),
        abbreviation: Some("COMB"),
        cost: 10,
        flags: 128,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("riding"),
        abbreviation: Some("RIDI"),
        cost: 10,
        flags: 128,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("crossbow"),
        abbreviation: Some("XBOW"),
        cost: 10,
        flags: 128,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("longbow"),
        abbreviation: Some("LBOW"),
        cost: 10,
        flags: 128,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("stealth"),
        abbreviation: Some("STEA"),
        cost: 50,
        flags: 0,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("observation"),
        abbreviation: Some("OBSE"),
        cost: 50,
        flags: 0,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("healing"),
        abbreviation: Some("HEAL"),
        cost: 10,
        flags: 0,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("sailing"),
        abbreviation: Some("SAIL"),
        cost: 10,
        flags: 0,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("farming"),
        abbreviation: Some("FARM"),
        cost: 10,
        flags: 0,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("ranching"),
        abbreviation: Some("RANC"),
        cost: 10,
        flags: 0,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("force"),
        abbreviation: Some("FORC"),
        cost: 100,
        flags: 9,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("pattern"),
        abbreviation: Some("PATT"),
        cost: 100,
        flags: 9,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("spirit"),
        abbreviation: Some("SPIR"),
        cost: 100,
        flags: 9,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("fire"),
        abbreviation: Some("FIRE"),
        cost: 100,
        flags: 515,
        special: Some("fireball"),
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("FORC"),
                level: 1,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("earthquake"),
        abbreviation: Some("EQUA"),
        cost: 100,
        flags: 515,
        special: Some("earthquake"),
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("FORC"),
                level: 1,
            },
            SkillDependency {
                skill: Some("PATT"),
                level: 1,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("force shield"),
        abbreviation: Some("FSHI"),
        cost: 100,
        flags: 2051,
        special: Some("force_shield"),
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("FORC"),
                level: 1,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("energy shield"),
        abbreviation: Some("ESHI"),
        cost: 100,
        flags: 2051,
        special: Some("energy_shield"),
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("FORC"),
                level: 1,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("spirit shield"),
        abbreviation: Some("SSHI"),
        cost: 100,
        flags: 2051,
        special: Some("shield_spirit"),
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("SPIR"),
                level: 1,
            },
            SkillDependency {
                skill: Some("FORC"),
                level: 1,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("magical healing"),
        abbreviation: Some("MHEA"),
        cost: 100,
        flags: 1,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("PATT"),
                level: 1,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("gate lore"),
        abbreviation: Some("GATE"),
        cost: 100,
        flags: 5,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("PATT"),
                level: 1,
            },
            SkillDependency {
                skill: Some("SPIR"),
                level: 1,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("farsight"),
        abbreviation: Some("FARS"),
        cost: 100,
        flags: 5,
        special: None,
        range: Some("rng_farsight"),
        dependencies: [
            SkillDependency {
                skill: Some("PATT"),
                level: 1,
            },
            SkillDependency {
                skill: Some("SPIR"),
                level: 1,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("teleportation"),
        abbreviation: Some("TELE"),
        cost: 100,
        flags: 5,
        special: None,
        range: Some("rng_teleport"),
        dependencies: [
            SkillDependency {
                skill: Some("GATE"),
                level: 1,
            },
            SkillDependency {
                skill: Some("FARS"),
                level: 2,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("portal lore"),
        abbreviation: Some("PORT"),
        cost: 100,
        flags: 5,
        special: None,
        range: Some("rng_portal"),
        dependencies: [
            SkillDependency {
                skill: Some("GATE"),
                level: 2,
            },
            SkillDependency {
                skill: Some("FARS"),
                level: 1,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("mind reading"),
        abbreviation: Some("MIND"),
        cost: 100,
        flags: 5,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("PATT"),
                level: 1,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("weather lore"),
        abbreviation: Some("WEAT"),
        cost: 100,
        flags: 261,
        special: None,
        range: Some("rng_weather"),
        dependencies: [
            SkillDependency {
                skill: Some("FORC"),
                level: 1,
            },
            SkillDependency {
                skill: Some("PATT"),
                level: 1,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("summon wind"),
        abbreviation: Some("SWIN"),
        cost: 100,
        flags: 5,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("WEAT"),
                level: 1,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("summon storm"),
        abbreviation: Some("SSTO"),
        cost: 100,
        flags: 1027,
        special: Some("storm"),
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("WEAT"),
                level: 1,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("summon tornado"),
        abbreviation: Some("STOR"),
        cost: 100,
        flags: 515,
        special: Some("tornado"),
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("WEAT"),
                level: 3,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("call lightning"),
        abbreviation: Some("CALL"),
        cost: 100,
        flags: 515,
        special: Some("lightning"),
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("WEAT"),
                level: 5,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("clear skies"),
        abbreviation: Some("CLEA"),
        cost: 100,
        flags: 2055,
        special: Some("clear_skies"),
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("WEAT"),
                level: 1,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("earth lore"),
        abbreviation: Some("EART"),
        cost: 100,
        flags: 261,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("PATT"),
                level: 1,
            },
            SkillDependency {
                skill: Some("FORC"),
                level: 1,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("wolf lore"),
        abbreviation: Some("WOLF"),
        cost: 100,
        flags: 5,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("EART"),
                level: 1,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("bird lore"),
        abbreviation: Some("BIRD"),
        cost: 100,
        flags: 5,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("EART"),
                level: 1,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("dragon lore"),
        abbreviation: Some("DRAG"),
        cost: 100,
        flags: 5,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("BIRD"),
                level: 3,
            },
            SkillDependency {
                skill: Some("WOLF"),
                level: 3,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("necromancy"),
        abbreviation: Some("NECR"),
        cost: 100,
        flags: 257,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("FORC"),
                level: 1,
            },
            SkillDependency {
                skill: Some("SPIR"),
                level: 1,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("summon skeletons"),
        abbreviation: Some("SUSK"),
        cost: 100,
        flags: 5,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("NECR"),
                level: 1,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("raise undead"),
        abbreviation: Some("RAIS"),
        cost: 100,
        flags: 5,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("NECR"),
                level: 3,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("summon lich"),
        abbreviation: Some("SULI"),
        cost: 100,
        flags: 5,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("NECR"),
                level: 5,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("create aura of fear"),
        abbreviation: Some("FEAR"),
        cost: 100,
        flags: 1027,
        special: Some("fear"),
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("NECR"),
                level: 1,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("summon black wind"),
        abbreviation: Some("SBLA"),
        cost: 100,
        flags: 515,
        special: Some("black_wind"),
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("NECR"),
                level: 5,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("banish undead"),
        abbreviation: Some("BUND"),
        cost: 100,
        flags: 3,
        special: Some("banish_undead"),
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("NECR"),
                level: 1,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("demon lore"),
        abbreviation: Some("DEMO"),
        cost: 100,
        flags: 257,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("FORC"),
                level: 1,
            },
            SkillDependency {
                skill: Some("SPIR"),
                level: 1,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("summon imps"),
        abbreviation: Some("SUIM"),
        cost: 100,
        flags: 5,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("DEMO"),
                level: 1,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("summon demon"),
        abbreviation: Some("SUDE"),
        cost: 100,
        flags: 5,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("SUIM"),
                level: 3,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("summon balrog"),
        abbreviation: Some("SUBA"),
        cost: 100,
        flags: 5,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("SUDE"),
                level: 3,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("banish demons"),
        abbreviation: Some("BDEM"),
        cost: 100,
        flags: 3,
        special: Some("banish_demon"),
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("DEMO"),
                level: 1,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("illusion"),
        abbreviation: Some("ILLU"),
        cost: 100,
        flags: 257,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("FORC"),
                level: 1,
            },
            SkillDependency {
                skill: Some("PATT"),
                level: 1,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("phantasmal entertainment"),
        abbreviation: Some("PHEN"),
        cost: 100,
        flags: 5,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("ILLU"),
                level: 1,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("create phantasmal beasts"),
        abbreviation: Some("PHBE"),
        cost: 100,
        flags: 5,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("ILLU"),
                level: 1,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("create phantasmal undead"),
        abbreviation: Some("PHUN"),
        cost: 100,
        flags: 37,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("ILLU"),
                level: 1,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("create phantasmal demons"),
        abbreviation: Some("PHDE"),
        cost: 100,
        flags: 5,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("ILLU"),
                level: 1,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("invisibility"),
        abbreviation: Some("INVI"),
        cost: 100,
        flags: 5,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("ILLU"),
                level: 3,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("true seeing"),
        abbreviation: Some("TRUE"),
        cost: 100,
        flags: 1,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("ILLU"),
                level: 3,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("dispel illusions"),
        abbreviation: Some("DISP"),
        cost: 100,
        flags: 3,
        special: Some("dispel_illusion"),
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("ILLU"),
                level: 1,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("artifact lore"),
        abbreviation: Some("ARTI"),
        cost: 100,
        flags: 257,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("FORC"),
                level: 1,
            },
            SkillDependency {
                skill: Some("PATT"),
                level: 1,
            },
            SkillDependency {
                skill: Some("SPIR"),
                level: 1,
            },
        ],
    },
    SkillDef {
        name: Some("create ring of invisibility"),
        abbreviation: Some("CRRI"),
        cost: 100,
        flags: 5,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("ARTI"),
                level: 2,
            },
            SkillDependency {
                skill: Some("INVI"),
                level: 3,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("create cloak of invulnerability"),
        abbreviation: Some("CRCL"),
        cost: 100,
        flags: 5,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("ARTI"),
                level: 4,
            },
            SkillDependency {
                skill: Some("FSHI"),
                level: 4,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("create staff of fire"),
        abbreviation: Some("CRSF"),
        cost: 100,
        flags: 5,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("ARTI"),
                level: 2,
            },
            SkillDependency {
                skill: Some("FIRE"),
                level: 3,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("create staff of lightning"),
        abbreviation: Some("CRSL"),
        cost: 100,
        flags: 5,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("ARTI"),
                level: 4,
            },
            SkillDependency {
                skill: Some("CALL"),
                level: 3,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("create amulet of true seeing"),
        abbreviation: Some("CRTA"),
        cost: 100,
        flags: 5,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("ARTI"),
                level: 2,
            },
            SkillDependency {
                skill: Some("TRUE"),
                level: 3,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("create amulet of protection"),
        abbreviation: Some("CRPA"),
        cost: 100,
        flags: 5,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("ARTI"),
                level: 1,
            },
            SkillDependency {
                skill: Some("SSHI"),
                level: 3,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("create runesword"),
        abbreviation: Some("CRRU"),
        cost: 100,
        flags: 5,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("ARTI"),
                level: 2,
            },
            SkillDependency {
                skill: Some("FEAR"),
                level: 3,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("create shieldstone"),
        abbreviation: Some("CRSS"),
        cost: 100,
        flags: 5,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("ARTI"),
                level: 1,
            },
            SkillDependency {
                skill: Some("ESHI"),
                level: 3,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("create magic carpet"),
        abbreviation: Some("CRMA"),
        cost: 100,
        flags: 5,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("ARTI"),
                level: 1,
            },
            SkillDependency {
                skill: Some("WEAT"),
                level: 3,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("engrave runes of warding"),
        abbreviation: Some("ENGR"),
        cost: 100,
        flags: 5,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("ARTI"),
                level: 2,
            },
            SkillDependency {
                skill: Some("ESHI"),
                level: 3,
            },
            SkillDependency {
                skill: Some("SSHI"),
                level: 3,
            },
        ],
    },
    SkillDef {
        name: Some("construct gate"),
        abbreviation: Some("CGAT"),
        cost: 100,
        flags: 5,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("ARTI"),
                level: 2,
            },
            SkillDependency {
                skill: Some("GATE"),
                level: 3,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("enchant swords"),
        abbreviation: Some("ESWO"),
        cost: 100,
        flags: 5,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("ARTI"),
                level: 2,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("enchant armor"),
        abbreviation: Some("EARM"),
        cost: 100,
        flags: 5,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("ARTI"),
                level: 2,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("enchant shields"),
        abbreviation: Some("ESHD"),
        cost: 100,
        flags: 37,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("ARTI"),
                level: 1,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("construct portal"),
        abbreviation: Some("CPOR"),
        cost: 100,
        flags: 5,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("ARTI"),
                level: 1,
            },
            SkillDependency {
                skill: Some("PORT"),
                level: 2,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("manipulation"),
        abbreviation: Some("MANI"),
        cost: 100,
        flags: 48,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("weaponcraft"),
        abbreviation: Some("WCRA"),
        cost: 10,
        flags: 32,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("WEAP"),
                level: 5,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("armorcraft"),
        abbreviation: Some("ACRA"),
        cost: 10,
        flags: 32,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("ARMO"),
                level: 5,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("camel training"),
        abbreviation: Some("CAME"),
        cost: 10,
        flags: 32,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("gemcutting"),
        abbreviation: Some("GCUT"),
        cost: 20,
        flags: 32,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("monster training"),
        abbreviation: Some("MTRA"),
        cost: 50,
        flags: 32,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("create flaming sword"),
        abbreviation: Some("CFSW"),
        cost: 100,
        flags: 37,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("ARTI"),
                level: 2,
            },
            SkillDependency {
                skill: Some("FIRE"),
                level: 3,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("cooking"),
        abbreviation: Some("COOK"),
        cost: 10,
        flags: 32,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("create food"),
        abbreviation: Some("CRFD"),
        cost: 100,
        flags: 37,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("EART"),
                level: 1,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("quartermaster"),
        abbreviation: Some("QUAM"),
        cost: 100,
        flags: 32,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("create aegis"),
        abbreviation: Some("CRAG"),
        cost: 100,
        flags: 37,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("ARTI"),
                level: 2,
            },
            SkillDependency {
                skill: Some("TRUE"),
                level: 5,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("create windchime"),
        abbreviation: Some("CRWC"),
        cost: 100,
        flags: 37,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("ARTI"),
                level: 1,
            },
            SkillDependency {
                skill: Some("SWIN"),
                level: 2,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("create gate crystal"),
        abbreviation: Some("CRGC"),
        cost: 100,
        flags: 37,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("ARTI"),
                level: 2,
            },
            SkillDependency {
                skill: Some("GATE"),
                level: 2,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("create staff of healing"),
        abbreviation: Some("CRSH"),
        cost: 100,
        flags: 37,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("ARTI"),
                level: 2,
            },
            SkillDependency {
                skill: Some("MHEA"),
                level: 2,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("create scrying orb"),
        abbreviation: Some("CRSO"),
        cost: 100,
        flags: 37,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("ARTI"),
                level: 2,
            },
            SkillDependency {
                skill: Some("FARS"),
                level: 2,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("create cornucopia"),
        abbreviation: Some("CRCO"),
        cost: 100,
        flags: 37,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("ARTI"),
                level: 1,
            },
            SkillDependency {
                skill: Some("EART"),
                level: 2,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("create book of exorcism"),
        abbreviation: Some("CRBX"),
        cost: 100,
        flags: 37,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("ARTI"),
                level: 2,
            },
            SkillDependency {
                skill: Some("BDEM"),
                level: 2,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("create holy symbol"),
        abbreviation: Some("CRHS"),
        cost: 100,
        flags: 37,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("ARTI"),
                level: 2,
            },
            SkillDependency {
                skill: Some("BUND"),
                level: 2,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("create censer of protection"),
        abbreviation: Some("CRCN"),
        cost: 100,
        flags: 37,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("ARTI"),
                level: 2,
            },
            SkillDependency {
                skill: Some("FSHI"),
                level: 3,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("transmutation"),
        abbreviation: Some("TRNS"),
        cost: 100,
        flags: 37,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("ARTI"),
                level: 1,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
    SkillDef {
        name: Some("blasphemous ritual"),
        abbreviation: Some("BRTL"),
        cost: 500000,
        flags: 37,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("PATT"),
                level: 4,
            },
            SkillDependency {
                skill: Some("FORC"),
                level: 4,
            },
            SkillDependency {
                skill: Some("SPIR"),
                level: 4,
            },
        ],
    },
    SkillDef {
        name: Some("endurance"),
        abbreviation: Some("ENDU"),
        cost: 50,
        flags: 160,
        special: None,
        range: None,
        dependencies: [
            SkillDependency {
                skill: Some("COMB"),
                level: 3,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
            SkillDependency {
                skill: None,
                level: 0,
            },
        ],
    },
];

/// C++: `gamedata.cpp` `hd` (`HealDefs`) table order.
pub const HEALING_DEFS: &[HealingDef] = &[
    HealingDef { number: 0, rate: 0 },
    HealingDef {
        number: 1,
        rate: 40,
    },
    HealingDef {
        number: 1,
        rate: 45,
    },
    HealingDef {
        number: 1,
        rate: 50,
    },
    HealingDef {
        number: 1,
        rate: 55,
    },
    HealingDef {
        number: 1,
        rate: 60,
    },
    HealingDef {
        number: 1,
        rate: 70,
    },
];

/// C++: `gamedata.cpp` `mhd` (`MagicHealDefs`) table order.
pub const MAGIC_HEALING_DEFS: &[HealingDef] = &[
    HealingDef { number: 0, rate: 0 },
    HealingDef {
        number: 5,
        rate: 70,
    },
    HealingDef {
        number: 20,
        rate: 75,
    },
    HealingDef {
        number: 40,
        rate: 80,
    },
    HealingDef {
        number: 80,
        rate: 85,
    },
    HealingDef {
        number: 160,
        rate: 90,
    },
];

/// C++: `gamedata.cpp` `spd` (`SpecialDefs`) table order.
pub const SPECIAL_DEFS: &[SpecialDef] = &[
    SpecialDef {
        key: None,
        name: Some("an unknown attack"),
        target_flags: 0,
        buildings: [-1, -1, -1, -1, -1],
        targets: [-1, -1, -1, -1, -1, -1, -1],
        effects: [None, None, None],
        effect_flags: 0,
        shields: [-1, -1, -1, -1],
        defenses: [
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
        ],
        shield_description: Some("a non-shield spell"),
        damage: [
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
        ],
        spell_description: Some("casts something that is not a spell"),
        secondary_spell_description: Some("annoying "),
        spell_target: Some(""),
    },
    SpecialDef {
        key: Some("fireball"),
        name: Some("a fireball"),
        target_flags: 0,
        buildings: [-1, -1, -1, -1, -1],
        targets: [-1, -1, -1, -1, -1, -1, -1],
        effects: [None, None, None],
        effect_flags: 6,
        shields: [-1, -1, -1, -1],
        defenses: [
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
        ],
        shield_description: Some("a non-shield spell"),
        damage: [
            SpecialDamage {
                kind: 1,
                minimum: 2,
                value: 5,
                flags: 130,
                class: 5,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
        ],
        spell_description: Some("shoots a Fireball"),
        secondary_spell_description: Some("killing "),
        spell_target: Some(""),
    },
    SpecialDef {
        key: Some("hellfire"),
        name: Some("hellfire"),
        target_flags: 0,
        buildings: [-1, -1, -1, -1, -1],
        targets: [-1, -1, -1, -1, -1, -1, -1],
        effects: [None, None, None],
        effect_flags: 6,
        shields: [-1, -1, -1, -1],
        defenses: [
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
        ],
        shield_description: Some("a non-shield spell"),
        damage: [
            SpecialDamage {
                kind: 1,
                minimum: 2,
                value: 25,
                flags: 130,
                class: 5,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
        ],
        spell_description: Some("blasts the enemy with Hellfire"),
        secondary_spell_description: Some("killing "),
        spell_target: Some(""),
    },
    SpecialDef {
        key: Some("fear"),
        name: Some("cause fear"),
        target_flags: 640,
        buildings: [-1, -1, -1, -1, -1],
        targets: [-1, -1, -1, -1, -1, -1, -1],
        effects: [Some("fear"), None, None],
        effect_flags: 6,
        shields: [-1, -1, -1, -1],
        defenses: [
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
        ],
        shield_description: Some("a non-shield spell"),
        damage: [
            SpecialDamage {
                kind: 2,
                minimum: 2,
                value: 25,
                flags: 130,
                class: 6,
                effect: Some("fear"),
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
        ],
        spell_description: Some("strikes fear into the enemy"),
        secondary_spell_description: Some("causing "),
        spell_target: Some(" men to cower in terror"),
    },
    SpecialDef {
        key: Some("lightning"),
        name: Some("a lightning strike"),
        target_flags: 0,
        buildings: [-1, -1, -1, -1, -1],
        targets: [-1, -1, -1, -1, -1, -1, -1],
        effects: [None, None, None],
        effect_flags: 6,
        shields: [-1, -1, -1, -1],
        defenses: [
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
        ],
        shield_description: Some("a non-shield spell"),
        damage: [
            SpecialDamage {
                kind: 3,
                minimum: 2,
                value: 25,
                flags: 130,
                class: 7,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: 1,
                minimum: 2,
                value: 25,
                flags: 130,
                class: 5,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
        ],
        spell_description: Some("unleashes a mighty lightning strike"),
        secondary_spell_description: Some("killing "),
        spell_target: Some(""),
    },
    SpecialDef {
        key: Some("mindblast"),
        name: Some("a mindblast"),
        target_flags: 0,
        buildings: [-1, -1, -1, -1, -1],
        targets: [-1, -1, -1, -1, -1, -1, -1],
        effects: [None, None, None],
        effect_flags: 2,
        shields: [-1, -1, -1, -1],
        defenses: [
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
        ],
        shield_description: Some("a non-shield spell"),
        damage: [
            SpecialDamage {
                kind: 2,
                minimum: 2,
                value: 125,
                flags: 130,
                class: 6,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
        ],
        spell_description: Some("blasts the minds of the enemy"),
        secondary_spell_description: Some("killing "),
        spell_target: Some(" troops"),
    },
    SpecialDef {
        key: Some("earthquake"),
        name: Some("an earthquake"),
        target_flags: 2,
        buildings: [24, 25, 26, 27, -1],
        targets: [-1, -1, -1, -1, -1, -1, -1],
        effects: [None, None, None],
        effect_flags: 22,
        shields: [-1, -1, -1, -1],
        defenses: [
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
        ],
        shield_description: Some("a non-shield spell"),
        damage: [
            SpecialDamage {
                kind: 3,
                minimum: 2,
                value: 30,
                flags: 130,
                class: 7,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
        ],
        spell_description: Some("invokes a mighty Earthquake"),
        secondary_spell_description: Some("killing "),
        spell_target: Some(""),
    },
    SpecialDef {
        key: Some("force_shield"),
        name: Some("a force shield"),
        target_flags: 0,
        buildings: [-1, -1, -1, -1, -1],
        targets: [-1, -1, -1, -1, -1, -1, -1],
        effects: [None, None, None],
        effect_flags: 13,
        shields: [5, -1, -1, -1],
        defenses: [
            DefenseModifier { kind: 0, value: 1 },
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
        ],
        shield_description: Some("Force Shield"),
        damage: [
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
        ],
        spell_description: Some("casts something that is not a spell"),
        secondary_spell_description: Some("annoying "),
        spell_target: Some(""),
    },
    SpecialDef {
        key: Some("energy_shield"),
        name: Some("an energy shield"),
        target_flags: 0,
        buildings: [-1, -1, -1, -1, -1],
        targets: [-1, -1, -1, -1, -1, -1, -1],
        effects: [None, None, None],
        effect_flags: 1,
        shields: [1, -1, -1, -1],
        defenses: [
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
        ],
        shield_description: Some("Energy Shield"),
        damage: [
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
        ],
        spell_description: Some("casts something that is not a spell"),
        secondary_spell_description: Some("annoying "),
        spell_target: Some(""),
    },
    SpecialDef {
        key: Some("shield_spirit"),
        name: Some("a spirit shield"),
        target_flags: 0,
        buildings: [-1, -1, -1, -1, -1],
        targets: [-1, -1, -1, -1, -1, -1, -1],
        effects: [None, None, None],
        effect_flags: 1,
        shields: [2, -1, -1, -1],
        defenses: [
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
        ],
        shield_description: Some("Spirit Shield"),
        damage: [
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
        ],
        spell_description: Some("casts something that is not a spell"),
        secondary_spell_description: Some("annoying "),
        spell_target: Some(""),
    },
    SpecialDef {
        key: Some("dispel_illusion"),
        name: Some("dispel illusion"),
        target_flags: 256,
        buildings: [-1, -1, -1, -1, -1],
        targets: [-1, -1, -1, -1, -1, -1, -1],
        effects: [None, None, None],
        effect_flags: 6,
        shields: [-1, -1, -1, -1],
        defenses: [
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
        ],
        shield_description: Some("a non-shield spell"),
        damage: [
            SpecialDamage {
                kind: 6,
                minimum: 2,
                value: 50,
                flags: 130,
                class: 5,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
        ],
        spell_description: Some("casts Dispel Illusions"),
        secondary_spell_description: Some("dispelling "),
        spell_target: Some(" illusions"),
    },
    SpecialDef {
        key: Some("storm"),
        name: Some("summon storm"),
        target_flags: 128,
        buildings: [-1, -1, -1, -1, -1],
        targets: [-1, -1, -1, -1, -1, -1, -1],
        effects: [Some("storm"), None, None],
        effect_flags: 6,
        shields: [-1, -1, -1, -1],
        defenses: [
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
        ],
        shield_description: Some("a non-shield spell"),
        damage: [
            SpecialDamage {
                kind: 3,
                minimum: 2,
                value: 50,
                flags: 130,
                class: 7,
                effect: Some("storm"),
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
        ],
        spell_description: Some("summons a terrible storm"),
        secondary_spell_description: Some("reducing the effectiveness of "),
        spell_target: Some(" troops"),
    },
    SpecialDef {
        key: Some("tornado"),
        name: Some("summon tornado"),
        target_flags: 0,
        buildings: [-1, -1, -1, -1, -1],
        targets: [-1, -1, -1, -1, -1, -1, -1],
        effects: [None, None, None],
        effect_flags: 6,
        shields: [-1, -1, -1, -1],
        defenses: [
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
        ],
        shield_description: Some("a non-shield spell"),
        damage: [
            SpecialDamage {
                kind: 3,
                minimum: 2,
                value: 25,
                flags: 130,
                class: 7,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
        ],
        spell_description: Some("summons a wild tornado"),
        secondary_spell_description: Some("killing "),
        spell_target: Some(""),
    },
    SpecialDef {
        key: Some("clear_skies"),
        name: Some("clear skies"),
        target_flags: 0,
        buildings: [-1, -1, -1, -1, -1],
        targets: [-1, -1, -1, -1, -1, -1, -1],
        effects: [None, None, None],
        effect_flags: 1,
        shields: [3, -1, -1, -1],
        defenses: [
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
        ],
        shield_description: Some("Clear Skies"),
        damage: [
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
        ],
        spell_description: Some("casts something that is not a spell"),
        secondary_spell_description: Some("annoying "),
        spell_target: Some(""),
    },
    SpecialDef {
        key: Some("black_wind"),
        name: Some("a black wind"),
        target_flags: 8,
        buildings: [-1, -1, -1, -1, -1],
        targets: [103, 104, 105, 106, 107, 108, 228],
        effects: [None, None, None],
        effect_flags: 6,
        shields: [-1, -1, -1, -1],
        defenses: [
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
        ],
        shield_description: Some("a non-shield spell"),
        damage: [
            SpecialDamage {
                kind: 2,
                minimum: 2,
                value: 65,
                flags: 130,
                class: 6,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
        ],
        spell_description: Some("summons the black wind"),
        secondary_spell_description: Some("killing "),
        spell_target: Some(""),
    },
    SpecialDef {
        key: Some("banish_undead"),
        name: Some("banish undead"),
        target_flags: 4,
        buildings: [-1, -1, -1, -1, -1],
        targets: [103, 104, 105, -1, -1, -1, -1],
        effects: [None, None, None],
        effect_flags: 6,
        shields: [-1, -1, -1, -1],
        defenses: [
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
        ],
        shield_description: Some("a non-shield spell"),
        damage: [
            SpecialDamage {
                kind: 6,
                minimum: 2,
                value: 25,
                flags: 130,
                class: 5,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
        ],
        spell_description: Some("casts Banish Undead"),
        secondary_spell_description: Some("banishing "),
        spell_target: Some(" undead"),
    },
    SpecialDef {
        key: Some("banish_demon"),
        name: Some("banish demons"),
        target_flags: 4,
        buildings: [-1, -1, -1, -1, -1],
        targets: [106, 107, 108, 228, -1, -1, -1],
        effects: [None, None, None],
        effect_flags: 6,
        shields: [-1, -1, -1, -1],
        defenses: [
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
        ],
        shield_description: Some("a non-shield spell"),
        damage: [
            SpecialDamage {
                kind: 6,
                minimum: 2,
                value: 25,
                flags: 130,
                class: 6,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
        ],
        spell_description: Some("casts Banish Demons"),
        secondary_spell_description: Some("banishing "),
        spell_target: Some(" demons"),
    },
    SpecialDef {
        key: Some("firebreath"),
        name: Some("fiery breath"),
        target_flags: 0,
        buildings: [-1, -1, -1, -1, -1],
        targets: [-1, -1, -1, -1, -1, -1, -1],
        effects: [None, None, None],
        effect_flags: 6,
        shields: [-1, -1, -1, -1],
        defenses: [
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
        ],
        shield_description: Some("a non-shield spell"),
        damage: [
            SpecialDamage {
                kind: 1,
                minimum: 2,
                value: 5,
                flags: 130,
                class: 5,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
        ],
        spell_description: Some("breathes Fire"),
        secondary_spell_description: Some("killing "),
        spell_target: Some(""),
    },
    SpecialDef {
        key: Some("icebreath"),
        name: Some("icy breath"),
        target_flags: 0,
        buildings: [-1, -1, -1, -1, -1],
        targets: [-1, -1, -1, -1, -1, -1, -1],
        effects: [None, None, None],
        effect_flags: 6,
        shields: [-1, -1, -1, -1],
        defenses: [
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
        ],
        shield_description: Some("a non-shield spell"),
        damage: [
            SpecialDamage {
                kind: 2,
                minimum: 2,
                value: 5,
                flags: 130,
                class: 6,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
        ],
        spell_description: Some("breathes Ice"),
        secondary_spell_description: Some("killing "),
        spell_target: Some(""),
    },
    SpecialDef {
        key: Some("spook_horses"),
        name: Some("terrify horses"),
        target_flags: 144,
        buildings: [-1, -1, -1, -1, -1],
        targets: [28, -1, -1, -1, -1, -1, -1],
        effects: [Some("spook"), None, None],
        effect_flags: 2,
        shields: [-1, -1, -1, -1],
        defenses: [
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
        ],
        shield_description: Some("a non-shield spell"),
        damage: [
            SpecialDamage {
                kind: 4,
                minimum: 1,
                value: 0,
                flags: 130,
                class: 6,
                effect: Some("spook"),
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
        ],
        spell_description: Some("strikes fear into enemy mounts"),
        secondary_spell_description: Some("causing "),
        spell_target: Some(" mounts to panic"),
    },
    SpecialDef {
        key: Some("invulnerable"),
        name: Some("invulnerability"),
        target_flags: 0,
        buildings: [-1, -1, -1, -1, -1],
        targets: [-1, -1, -1, -1, -1, -1, -1],
        effects: [None, None, None],
        effect_flags: 1,
        shields: [6, -1, -1, -1],
        defenses: [
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
        ],
        shield_description: Some("invulnerability"),
        damage: [
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
        ],
        spell_description: Some("casts something that is not a spell"),
        secondary_spell_description: Some("annoying "),
        spell_target: Some(""),
    },
    SpecialDef {
        key: Some("physical_shield"),
        name: Some("a physical shield"),
        target_flags: 0,
        buildings: [-1, -1, -1, -1, -1],
        targets: [-1, -1, -1, -1, -1, -1, -1],
        effects: [None, None, None],
        effect_flags: 1,
        shields: [5, -1, -1, -1],
        defenses: [
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
        ],
        shield_description: Some("Physical Shield"),
        damage: [
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
        ],
        spell_description: Some("casts something that is not a spell"),
        secondary_spell_description: Some("annoying "),
        spell_target: Some(""),
    },
    SpecialDef {
        key: Some("catapult strike"),
        name: Some("a catapult strike"),
        target_flags: 0,
        buildings: [-1, -1, -1, -1, -1],
        targets: [-1, -1, -1, -1, -1, -1, -1],
        effects: [None, None, None],
        effect_flags: 6,
        shields: [-1, -1, -1, -1],
        defenses: [
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
        ],
        shield_description: Some("a non-shield attack"),
        damage: [
            SpecialDamage {
                kind: 5,
                minimum: 1,
                value: 4,
                flags: 130,
                class: 4,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
            SpecialDamage {
                kind: -1,
                minimum: 0,
                value: 0,
                flags: 0,
                class: 0,
                effect: None,
                hit_damage: 1,
            },
        ],
        spell_description: Some("strikes"),
        secondary_spell_description: Some("killing "),
        spell_target: Some(""),
    },
];

/// C++: `gamedata.cpp` `efd` (`EffectDefs`) table order.
pub const EFFECT_DEFS: &[EffectDef] = &[
    EffectDef {
        name: Some("dazzle"),
        attack_value: -2,
        defense_modifiers: [
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
        ],
        cancel_effect: None,
        flags: 1,
    },
    EffectDef {
        name: Some("fear"),
        attack_value: -2,
        defense_modifiers: [
            DefenseModifier { kind: 0, value: -2 },
            DefenseModifier { kind: 4, value: -2 },
            DefenseModifier { kind: 5, value: -2 },
            DefenseModifier { kind: -1, value: 0 },
        ],
        cancel_effect: None,
        flags: 0,
    },
    EffectDef {
        name: Some("storm"),
        attack_value: -2,
        defense_modifiers: [
            DefenseModifier { kind: 0, value: -2 },
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
            DefenseModifier { kind: -1, value: 0 },
        ],
        cancel_effect: None,
        flags: 0,
    },
    EffectDef {
        name: Some("spook"),
        attack_value: -1,
        defense_modifiers: [
            DefenseModifier { kind: 0, value: -1 },
            DefenseModifier { kind: 4, value: -1 },
            DefenseModifier { kind: 5, value: -1 },
            DefenseModifier { kind: -1, value: 0 },
        ],
        cancel_effect: None,
        flags: 0,
    },
];

/// C++: `gamedata.cpp` `rtd` (`RangeDefs`) table order.
pub const RANGE_DEFS: &[RangeDef] = &[
    RangeDef {
        key: Some("rng_teleport"),
        flags: 0,
        range_class: 2,
        range_multiplier: 2,
        cross_level_penalty: 4,
    },
    RangeDef {
        key: Some("rng_portal"),
        flags: 0,
        range_class: 2,
        range_multiplier: 2,
        cross_level_penalty: 4,
    },
    RangeDef {
        key: Some("rng_farsight"),
        flags: 0,
        range_class: 2,
        range_multiplier: 4,
        cross_level_penalty: 4,
    },
    RangeDef {
        key: Some("rng_weather"),
        flags: 0,
        range_class: 2,
        range_multiplier: 4,
        cross_level_penalty: 4,
    },
    RangeDef {
        key: Some("rng_clearsky"),
        flags: 0,
        range_class: 2,
        range_multiplier: 2,
        cross_level_penalty: 4,
    },
    RangeDef {
        key: Some("rng_transport"),
        flags: 0,
        range_class: 0,
        range_multiplier: 1,
        cross_level_penalty: 10000000,
    },
];

/// C++: `gamedata.cpp` `atd` (`AttribDefs`) table order.
pub const ATTRIBUTE_MODIFIER_DEFS: &[AttributeModifierDef] = &[
    AttributeModifierDef {
        key: Some("tactics"),
        flags: 1,
        modifiers: [
            AttributeModifier {
                flags: 513,
                identifier: Some("TACT"),
                kind: 1,
                value: 1,
            },
            AttributeModifier {
                flags: 0,
                identifier: None,
                kind: 0,
                value: 0,
            },
            AttributeModifier {
                flags: 0,
                identifier: None,
                kind: 0,
                value: 0,
            },
            AttributeModifier {
                flags: 0,
                identifier: None,
                kind: 0,
                value: 0,
            },
            AttributeModifier {
                flags: 0,
                identifier: None,
                kind: 0,
                value: 0,
            },
        ],
    },
    AttributeModifierDef {
        key: Some("combat"),
        flags: 0,
        modifiers: [
            AttributeModifier {
                flags: 513,
                identifier: Some("COMB"),
                kind: 1,
                value: 1,
            },
            AttributeModifier {
                flags: 0,
                identifier: None,
                kind: 0,
                value: 0,
            },
            AttributeModifier {
                flags: 0,
                identifier: None,
                kind: 0,
                value: 0,
            },
            AttributeModifier {
                flags: 0,
                identifier: None,
                kind: 0,
                value: 0,
            },
            AttributeModifier {
                flags: 0,
                identifier: None,
                kind: 0,
                value: 0,
            },
        ],
    },
    AttributeModifierDef {
        key: Some("stealth"),
        flags: 3,
        modifiers: [
            AttributeModifier {
                flags: 513,
                identifier: Some("STEA"),
                kind: 1,
                value: 1,
            },
            AttributeModifier {
                flags: 1026,
                identifier: Some("RING"),
                kind: 0,
                value: 3,
            },
            AttributeModifier {
                flags: 4,
                identifier: Some("invis"),
                kind: 0,
                value: 3,
            },
            AttributeModifier {
                flags: 4,
                identifier: Some("guard"),
                kind: 3,
                value: 0,
            },
            AttributeModifier {
                flags: 0,
                identifier: None,
                kind: 0,
                value: 0,
            },
        ],
    },
    AttributeModifierDef {
        key: Some("observation"),
        flags: 1,
        modifiers: [
            AttributeModifier {
                flags: 513,
                identifier: Some("OBSE"),
                kind: 1,
                value: 1,
            },
            AttributeModifier {
                flags: 1,
                identifier: Some("TRUE"),
                kind: 2,
                value: 1,
            },
            AttributeModifier {
                flags: 2,
                identifier: Some("AMTS"),
                kind: 0,
                value: 2,
            },
            AttributeModifier {
                flags: 2,
                identifier: Some("AEGS"),
                kind: 0,
                value: 3,
            },
            AttributeModifier {
                flags: 0,
                identifier: None,
                kind: 0,
                value: 0,
            },
        ],
    },
    AttributeModifierDef {
        key: Some("wind"),
        flags: 0,
        modifiers: [
            AttributeModifier {
                flags: 1,
                identifier: Some("SWIN"),
                kind: 1,
                value: 1,
            },
            AttributeModifier {
                flags: 2,
                identifier: Some("WCHM"),
                kind: 0,
                value: 2,
            },
            AttributeModifier {
                flags: 0,
                identifier: None,
                kind: 0,
                value: 0,
            },
            AttributeModifier {
                flags: 0,
                identifier: None,
                kind: 0,
                value: 0,
            },
            AttributeModifier {
                flags: 0,
                identifier: None,
                kind: 0,
                value: 0,
            },
        ],
    },
    AttributeModifierDef {
        key: Some("entertainment"),
        flags: 0,
        modifiers: [
            AttributeModifier {
                flags: 1,
                identifier: Some("PHEN"),
                kind: 1,
                value: 5,
            },
            AttributeModifier {
                flags: 1,
                identifier: Some("ENTE"),
                kind: 1,
                value: 1,
            },
            AttributeModifier {
                flags: 0,
                identifier: None,
                kind: 0,
                value: 0,
            },
            AttributeModifier {
                flags: 0,
                identifier: None,
                kind: 0,
                value: 0,
            },
            AttributeModifier {
                flags: 0,
                identifier: None,
                kind: 0,
                value: 0,
            },
        ],
    },
    AttributeModifierDef {
        key: Some("toughness"),
        flags: 0,
        modifiers: [
            AttributeModifier {
                flags: 1,
                identifier: Some("ENDU"),
                kind: 2,
                value: 1,
            },
            AttributeModifier {
                flags: 0,
                identifier: None,
                kind: 0,
                value: 0,
            },
            AttributeModifier {
                flags: 0,
                identifier: None,
                kind: 0,
                value: 0,
            },
            AttributeModifier {
                flags: 0,
                identifier: None,
                kind: 0,
                value: 0,
            },
            AttributeModifier {
                flags: 0,
                identifier: None,
                kind: 0,
                value: 0,
            },
        ],
    },
    AttributeModifierDef {
        key: Some("susk"),
        flags: 0,
        modifiers: [
            AttributeModifier {
                flags: 1,
                identifier: Some("SUSK"),
                kind: 1,
                value: 1,
            },
            AttributeModifier {
                flags: 0,
                identifier: None,
                kind: 0,
                value: 0,
            },
            AttributeModifier {
                flags: 0,
                identifier: None,
                kind: 0,
                value: 0,
            },
            AttributeModifier {
                flags: 0,
                identifier: None,
                kind: 0,
                value: 0,
            },
            AttributeModifier {
                flags: 0,
                identifier: None,
                kind: 0,
                value: 0,
            },
        ],
    },
    AttributeModifierDef {
        key: Some("rais"),
        flags: 0,
        modifiers: [
            AttributeModifier {
                flags: 1,
                identifier: Some("RAIS"),
                kind: 1,
                value: 1,
            },
            AttributeModifier {
                flags: 0,
                identifier: None,
                kind: 0,
                value: 0,
            },
            AttributeModifier {
                flags: 0,
                identifier: None,
                kind: 0,
                value: 0,
            },
            AttributeModifier {
                flags: 0,
                identifier: None,
                kind: 0,
                value: 0,
            },
            AttributeModifier {
                flags: 0,
                identifier: None,
                kind: 0,
                value: 0,
            },
        ],
    },
];
/// C++ declares `ShowDefs` in `skills.h` but provides no definition or consumer.
pub const SHOW_DEFS: &[ShowDef] = &[];

pub mod skill_flags {
    pub const MAGIC: i32 = 0x0001;
    pub const COMBAT: i32 = 0x0002;
    pub const CAST: i32 = 0x0004;
    pub const FOUNDATION: i32 = 0x0008;
    pub const APPRENTICE: i32 = 0x0010;
    pub const DISABLED: i32 = 0x0020;
    pub const SLOW_STUDY: i32 = 0x0040;
    pub const BATTLE_REPORT: i32 = 0x0080;
    pub const NOTIFY: i32 = 0x0100;
    pub const DAMAGE: i32 = 0x0200;
    pub const FEAR: i32 = 0x0400;
    pub const MAGE_OTHER: i32 = 0x0800;
    pub const NO_STUDY: i32 = 0x1000;
    pub const NO_TEACH: i32 = 0x2000;
    pub const NO_EXPERIENCE: i32 = 0x4000;
}

pub mod special_target_flags {
    pub const HIT_BUILDING_IF: i32 = 0x001;
    pub const HIT_BUILDING_EXCEPT: i32 = 0x002;
    pub const HIT_SOLDIER_IF: i32 = 0x004;
    pub const HIT_SOLDIER_EXCEPT: i32 = 0x008;
    pub const HIT_MOUNT_IF: i32 = 0x010;
    pub const HIT_MOUNT_EXCEPT: i32 = 0x020;
    pub const HIT_EFFECT_IF: i32 = 0x040;
    pub const HIT_EFFECT_EXCEPT: i32 = 0x080;
    pub const HIT_ILLUSION: i32 = 0x100;
    pub const HIT_NO_MONSTER: i32 = 0x200;
}

pub mod special_effect_flags {
    pub const SHIELD: i32 = 0x01;
    pub const DAMAGE: i32 = 0x02;
    pub const USE_LEVEL: i32 = 0x04;
    pub const DEFENSE_BONUS: i32 = 0x08;
    pub const NO_BUILDING: i32 = 0x10;
    pub const DONT_COMBINE: i32 = 0x20;
}

pub mod effect_flags {
    pub const ONE_SHOT: i32 = 0x001;
    pub const NO_SET: i32 = 0x002;
}

pub mod range_flags {
    pub const NEXUS_TARGET: i32 = 0x0001;
    pub const NEXUS_SOURCE: i32 = 0x0002;
    pub const CROSS_LEVELS: i32 = 0x0004;
    pub const SURFACE_ONLY: i32 = 0x0008;
}

pub mod range_classes {
    pub const ABSOLUTE: i32 = 0;
    pub const LEVEL: i32 = 1;
    pub const LEVEL_SQUARED: i32 = 2;
    pub const LEVEL_CUBED: i32 = 3;
    pub const COUNT: i32 = 4;
}

pub mod attribute_flags {
    pub const CHECK_MONSTERS: i32 = 0x01;
    pub const USE_WORST: i32 = 0x02;
}

pub mod attribute_item_flags {
    pub const SKILL: i32 = 0x0001;
    pub const ITEM: i32 = 0x0002;
    pub const FLAGGED: i32 = 0x0004;
    pub const NOT: i32 = 0x0100;
    pub const CUMULATIVE: i32 = 0x0200;
    pub const PERMANENT: i32 = 0x0400;
}

pub mod attribute_modifier_types {
    pub const CONSTANT: i32 = 0;
    pub const UNIT_LEVEL: i32 = 1;
    pub const UNIT_LEVEL_HALF: i32 = 2;
    pub const FORCE_CONSTANT: i32 = 3;
    pub const COUNT: i32 = 4;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SkillTables {
    pub skills: Vec<SkillDef>,
    pub healing: Vec<HealingDef>,
    pub magic_healing: Vec<HealingDef>,
    pub specials: Vec<SpecialDef>,
    pub effects: Vec<EffectDef>,
    pub ranges: Vec<RangeDef>,
    pub attribute_modifiers: Vec<AttributeModifierDef>,
    pub shows: Vec<ShowDef>,
}

impl Default for SkillTables {
    fn default() -> Self {
        Self {
            skills: SKILL_DEFS.to_vec(),
            healing: HEALING_DEFS.to_vec(),
            magic_healing: MAGIC_HEALING_DEFS.to_vec(),
            specials: SPECIAL_DEFS.to_vec(),
            effects: EFFECT_DEFS.to_vec(),
            ranges: RANGE_DEFS.to_vec(),
            attribute_modifiers: ATTRIBUTE_MODIFIER_DEFS.to_vec(),
            shows: SHOW_DEFS.to_vec(),
        }
    }
}

/// C++: `skills.cpp` `FindSkill`; returns the first abbreviation match.
pub fn find_skill<'a>(defs: &'a [SkillDef], abbreviation: &str) -> Option<&'a SkillDef> {
    defs.iter().find(|skill| {
        skill
            .abbreviation
            .is_some_and(|value| crate::text::is_equal(value, abbreviation))
    })
}

/// C++: `skills.cpp` `LookupSkill`; disabled skills are still discoverable.
pub fn lookup_skill(defs: &[SkillDef], abbreviation: &str) -> Option<usize> {
    defs.iter().position(|skill| {
        skill
            .abbreviation
            .is_some_and(|value| crate::text::is_equal(value, abbreviation))
    })
}

/// C++: `skills.cpp` `ParseSkill`; names and abbreviations of disabled skills are rejected.
pub fn parse_skill(defs: &[SkillDef], name_or_abbreviation: &str) -> Option<usize> {
    defs.iter().position(|skill| {
        skill.flags & skill_flags::DISABLED == 0
            && (skill
                .name
                .is_some_and(|value| crate::text::is_equal(value, name_or_abbreviation))
                || skill
                    .abbreviation
                    .is_some_and(|value| crate::text::is_equal(value, name_or_abbreviation)))
    })
}

/// C++: `skills.cpp` `FindSpecial`; returns the matching table index.
pub fn find_special(defs: &[SpecialDef], key: &str) -> Option<usize> {
    defs.iter().position(|special| {
        special
            .key
            .is_some_and(|value| crate::text::is_equal(value, key))
    })
}

/// C++: `skills.cpp` `FindEffect`; returns the matching table index.
pub fn find_effect(defs: &[EffectDef], name: &str) -> Option<usize> {
    defs.iter().position(|effect| {
        effect
            .name
            .is_some_and(|value| crate::text::is_equal(value, name))
    })
}

/// C++: `skills.cpp` `FindRange`; returns the matching table index.
pub fn find_range(defs: &[RangeDef], key: &str) -> Option<usize> {
    defs.iter().position(|range| {
        range
            .key
            .is_some_and(|value| crate::text::is_equal(value, key))
    })
}

/// C++: `skills.cpp` `FindAttrib`; returns the matching table index.
pub fn find_attribute(defs: &[AttributeModifierDef], key: &str) -> Option<usize> {
    defs.iter().position(|attribute| {
        attribute
            .key
            .is_some_and(|value| crate::text::is_equal(value, key))
    })
}

fn append_number(output: &mut String, value: i32) {
    write!(output, "{value}").expect("writing to String cannot fail");
}

fn append_json_string(output: &mut String, value: Option<&str>) {
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
            character if character <= '\u{1f}' => {
                write!(output, "\\u{:04x}", character as u32)
                    .expect("writing to String cannot fail");
            }
            character => output.push(character),
        }
    }
    output.push('"');
}

fn append_count(output: &mut String, table: &str, count: usize) {
    output.push_str("{\"table\":\"counts\",\"name\":");
    append_json_string(output, Some(table));
    output.push_str(",\"value\":");
    write!(output, "{count}").expect("writing to String cannot fail");
    output.push_str("}\n");
}

fn begin_indexed_row(output: &mut String, table: &str, index: usize) {
    output.push_str("{\"table\":\"");
    output.push_str(table);
    output.push_str("\",\"index\":");
    write!(output, "{index}").expect("writing to String cannot fail");
}

fn append_field(output: &mut String, name: &str) {
    output.push_str(",\"");
    output.push_str(name);
    output.push_str("\":");
}

fn append_i32_array<const N: usize>(output: &mut String, values: &[i32; N]) {
    output.push('[');
    for (index, value) in values.iter().enumerate() {
        if index != 0 {
            output.push(',');
        }
        append_number(output, *value);
    }
    output.push(']');
}

fn append_string_array<const N: usize>(output: &mut String, values: &[Option<&'static str>; N]) {
    output.push('[');
    for (index, value) in values.iter().enumerate() {
        if index != 0 {
            output.push(',');
        }
        append_json_string(output, *value);
    }
    output.push(']');
}

fn append_defense_array<const N: usize>(output: &mut String, values: &[DefenseModifier; N]) {
    output.push('[');
    for (index, value) in values.iter().enumerate() {
        if index != 0 {
            output.push(',');
        }
        output.push_str("{\"type\":");
        append_number(output, value.kind);
        output.push_str(",\"value\":");
        append_number(output, value.value);
        output.push('}');
    }
    output.push(']');
}

fn append_damage_array(output: &mut String, values: &[SpecialDamage; 4]) {
    output.push('[');
    for (index, value) in values.iter().enumerate() {
        if index != 0 {
            output.push(',');
        }
        output.push_str("{\"type\":");
        append_number(output, value.kind);
        output.push_str(",\"minimum\":");
        append_number(output, value.minimum);
        output.push_str(",\"value\":");
        append_number(output, value.value);
        output.push_str(",\"flags\":");
        append_number(output, value.flags);
        output.push_str(",\"class\":");
        append_number(output, value.class);
        output.push_str(",\"effect\":");
        append_json_string(output, value.effect);
        output.push_str(",\"hit_damage\":");
        append_number(output, value.hit_damage);
        output.push('}');
    }
    output.push(']');
}

fn append_dependencies(output: &mut String, values: &[SkillDependency; 3]) {
    output.push('[');
    for (index, value) in values.iter().enumerate() {
        if index != 0 {
            output.push(',');
        }
        output.push_str("{\"skill\":");
        append_json_string(output, value.skill);
        output.push_str(",\"level\":");
        append_number(output, value.level);
        output.push('}');
    }
    output.push(']');
}

fn write_skills(output: &mut String, skills: &[SkillDef]) {
    append_count(output, "skills", skills.len());
    for (index, skill) in skills.iter().enumerate() {
        begin_indexed_row(output, "skills", index);
        append_field(output, "name");
        append_json_string(output, skill.name);
        append_field(output, "abbreviation");
        append_json_string(output, skill.abbreviation);
        append_field(output, "cost");
        append_number(output, skill.cost);
        append_field(output, "flags");
        append_number(output, skill.flags);
        append_field(output, "special");
        append_json_string(output, skill.special);
        append_field(output, "range");
        append_json_string(output, skill.range);
        append_field(output, "dependencies");
        append_dependencies(output, &skill.dependencies);
        output.push_str("}\n");
    }
}

fn write_healing(output: &mut String, table: &str, values: &[HealingDef]) {
    append_count(output, table, values.len());
    for (index, value) in values.iter().enumerate() {
        begin_indexed_row(output, table, index);
        append_field(output, "number");
        append_number(output, value.number);
        append_field(output, "rate");
        append_number(output, value.rate);
        output.push_str("}\n");
    }
}

fn write_specials(output: &mut String, values: &[SpecialDef]) {
    append_count(output, "specials", values.len());
    for (index, special) in values.iter().enumerate() {
        begin_indexed_row(output, "specials", index);
        append_field(output, "key");
        append_json_string(output, special.key);
        append_field(output, "name");
        append_json_string(output, special.name);
        append_field(output, "target_flags");
        append_number(output, special.target_flags);
        append_field(output, "buildings");
        append_i32_array(output, &special.buildings);
        append_field(output, "targets");
        append_i32_array(output, &special.targets);
        append_field(output, "effects");
        append_string_array(output, &special.effects);
        append_field(output, "effect_flags");
        append_number(output, special.effect_flags);
        append_field(output, "shields");
        append_i32_array(output, &special.shields);
        append_field(output, "defenses");
        append_defense_array(output, &special.defenses);
        append_field(output, "shield_description");
        append_json_string(output, special.shield_description);
        append_field(output, "damage");
        append_damage_array(output, &special.damage);
        append_field(output, "spell_description");
        append_json_string(output, special.spell_description);
        append_field(output, "secondary_spell_description");
        append_json_string(output, special.secondary_spell_description);
        append_field(output, "spell_target");
        append_json_string(output, special.spell_target);
        output.push_str("}\n");
    }
}

fn write_effects(output: &mut String, values: &[EffectDef]) {
    append_count(output, "effects", values.len());
    for (index, effect) in values.iter().enumerate() {
        begin_indexed_row(output, "effects", index);
        append_field(output, "name");
        append_json_string(output, effect.name);
        append_field(output, "attack_value");
        append_number(output, effect.attack_value);
        append_field(output, "defense_modifiers");
        append_defense_array(output, &effect.defense_modifiers);
        append_field(output, "cancel_effect");
        append_json_string(output, effect.cancel_effect);
        append_field(output, "flags");
        append_number(output, effect.flags);
        output.push_str("}\n");
    }
}

fn write_ranges(output: &mut String, values: &[RangeDef]) {
    append_count(output, "ranges", values.len());
    for (index, range) in values.iter().enumerate() {
        begin_indexed_row(output, "ranges", index);
        append_field(output, "key");
        append_json_string(output, range.key);
        append_field(output, "flags");
        append_number(output, range.flags);
        append_field(output, "range_class");
        append_number(output, range.range_class);
        append_field(output, "range_multiplier");
        append_number(output, range.range_multiplier);
        append_field(output, "cross_level_penalty");
        append_number(output, range.cross_level_penalty);
        output.push_str("}\n");
    }
}

fn write_attributes(output: &mut String, values: &[AttributeModifierDef]) {
    append_count(output, "attribute_modifiers", values.len());
    for (index, attribute) in values.iter().enumerate() {
        begin_indexed_row(output, "attribute_modifiers", index);
        append_field(output, "key");
        append_json_string(output, attribute.key);
        append_field(output, "flags");
        append_number(output, attribute.flags);
        append_field(output, "modifiers");
        output.push('[');
        for (modifier_index, modifier) in attribute.modifiers.iter().enumerate() {
            if modifier_index != 0 {
                output.push(',');
            }
            output.push_str("{\"flags\":");
            append_number(output, modifier.flags);
            output.push_str(",\"identifier\":");
            append_json_string(output, modifier.identifier);
            output.push_str(",\"type\":");
            append_number(output, modifier.kind);
            output.push_str(",\"value\":");
            append_number(output, modifier.value);
            output.push('}');
        }
        output.push(']');
        output.push_str("}\n");
    }
}

/// Writes ordered family records using the C++ reference dump's JSONL schema.
pub fn write_skill_family_jsonl(output: &mut String, tables: &SkillTables) {
    write_skills(output, &tables.skills);
    write_healing(output, "healing", &tables.healing);
    write_healing(output, "magic_healing", &tables.magic_healing);
    write_specials(output, &tables.specials);
    write_effects(output, &tables.effects);
    write_ranges(output, &tables.ranges);
    write_attributes(output, &tables.attribute_modifiers);
}

#[cfg(test)]
mod tests {
    use super::skill_flags;
    use super::{
        find_attribute, find_effect, find_range, find_skill, find_special, lookup_skill,
        parse_skill, write_skill_family_jsonl, SkillDef, SkillDependency, SkillTables,
    };

    #[test]
    fn base_skill_families_match_pre_modify_reference_standard() {
        let mut actual = String::new();
        write_skill_family_jsonl(&mut actual, &SkillTables::default());
        assert_eq!(
            actual,
            include_str!("../../../../fixtures/tables/pre-modify-skill-families/standard.jsonl")
        );
    }

    #[test]
    fn base_skill_families_match_pre_modify_reference_neworigins() {
        let mut actual = String::new();
        write_skill_family_jsonl(&mut actual, &SkillTables::default());
        assert_eq!(
            actual,
            include_str!("../../../../fixtures/tables/pre-modify-skill-families/neworigins.jsonl")
        );
    }

    #[test]
    fn skill_lookup_matches_abbreviations_and_disabled_filtering() {
        let tables = SkillTables::default();
        assert_eq!(
            find_skill(&tables.skills, "mini").and_then(|skill| skill.name),
            Some("mining")
        );
        assert_eq!(lookup_skill(&tables.skills, "mInI"), Some(0));
        assert_eq!(parse_skill(&tables.skills, "Horse_Training"), Some(6));
        assert_eq!(parse_skill(&tables.skills, "not a skill"), None);

        let disabled = [SkillDef {
            name: Some("disabled skill"),
            abbreviation: Some("DABL"),
            cost: 0,
            flags: skill_flags::DISABLED,
            special: None,
            range: None,
            dependencies: [SkillDependency {
                skill: None,
                level: 0,
            }; 3],
        }];
        assert_eq!(lookup_skill(&disabled, "dabl"), Some(0));
        assert_eq!(parse_skill(&disabled, "disabled_skill"), None);
    }

    #[test]
    fn skill_family_find_functions_return_ordered_indices() {
        let tables = SkillTables::default();
        assert_eq!(find_special(&tables.specials, "FIREBALL"), Some(1));
        assert_eq!(find_effect(&tables.effects, "Fear"), Some(1));
        assert_eq!(find_range(&tables.ranges, "RNG TELEPORT"), Some(0));
        assert_eq!(
            find_attribute(&tables.attribute_modifiers, "TACTICS"),
            Some(0)
        );
        assert_eq!(find_special(&tables.specials, "missing"), None);
        assert_eq!(find_effect(&tables.effects, "missing"), None);
        assert_eq!(find_range(&tables.ranges, "missing"), None);
        assert_eq!(find_attribute(&tables.attribute_modifiers, "missing"), None);
        assert!(tables.shows.is_empty());
    }
}
