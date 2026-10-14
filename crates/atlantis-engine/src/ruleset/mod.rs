mod neworigins;
mod standard;

use crate::defs::GameDefs;

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

    fn name(&self) -> &'static str {
        self.game_defs().ruleset_name
    }
}

impl dyn Ruleset {
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
