use std::fmt;

use crate::output::EngineOutput;

/// Packed Atlantis version compatible with the original engine's `ATL_VER`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AtlVersion(u32);

impl AtlVersion {
    /// C++: helper.h `MAKE_ATL_VER`.
    pub const fn new(major: u8, minor: u8, patch: u8) -> Self {
        Self(((major as u32) << 16) + ((minor as u32) << 8) + patch as u32)
    }

    /// C++: helper.h `ATL_VER_MAJOR`.
    pub const fn major(self) -> u8 {
        ((self.0 >> 16) & 0xff) as u8
    }

    /// C++: helper.h `ATL_VER_MINOR`.
    pub const fn minor(self) -> u8 {
        ((self.0 >> 8) & 0xff) as u8
    }

    /// C++: helper.h `ATL_VER_PATCH`.
    pub const fn patch(self) -> u8 {
        (self.0 & 0xff) as u8
    }

    /// Returns the packed representation used by the C++ engine.
    pub const fn packed(self) -> u32 {
        self.0
    }
}

impl fmt::Display for AtlVersion {
    /// C++: helper.h `ATL_VER_STRING`; even minor versions carry the beta suffix.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}.{}.{}",
            self.major(),
            self.minor(),
            self.patch()
        )?;
        if self.minor().is_multiple_of(2) {
            formatter.write_str(" (beta)")?;
        }
        Ok(())
    }
}

/// C++: game.h `CURRENT_ATL_VER`.
pub const CURRENT_ATL_VER: AtlVersion = AtlVersion::new(5, 2, 5);

pub const STANDARD_RULESET_NAME: &str = "Wyreth";
pub const STANDARD_RULESET_VERSION: AtlVersion = AtlVersion::new(2, 0, 0);
pub const NEW_ORIGINS_RULESET_NAME: &str = "NewOrigins";
pub const NEW_ORIGINS_RULESET_VERSION: AtlVersion = AtlVersion::new(3, 0, 0);

/// C++: main.cpp's three startup `Awrite` calls.
pub fn write_startup_banner(
    output: &mut EngineOutput,
    ruleset_name: &str,
    ruleset_version: AtlVersion,
) {
    output.write_line(&format!("Atlantis Engine Version: {CURRENT_ATL_VER}"));
    output.write_line(&format!("{ruleset_name}, Version: {ruleset_version}"));
    output.write_line("");
}
