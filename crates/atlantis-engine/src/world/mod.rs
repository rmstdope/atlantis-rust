use crate::economy::SetupError;

pub mod classic;
mod names;
pub mod natural;
pub mod settlements;

pub use settlements::{make_starting_city, TownType};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldGenerationError {
    RegionArray(crate::model::RegionArrayError),
    InvalidSurfaceDimensions { width: i32, height: i32 },
    InvalidNaturalSurfaceDimensions { width: i32, height: i32 },
    NaturalMapDimensionsOverflow { width: i32, height: i32 },
    MissingNaturalBiome { x: i32, y: i32 },
    UnsupportedNaturalRuleset,
    RegionSetup(SetupError),
    MissingGeneratedName(i32),
    RegionNumberOverflow,
}

impl std::fmt::Display for WorldGenerationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RegionArray(error) => error.fmt(formatter),
            Self::InvalidSurfaceDimensions { width, height } => write!(
                formatter,
                "classic surface dimensions must be at least 8x8 (got {width}x{height})"
            ),
            Self::InvalidNaturalSurfaceDimensions { width, height } => write!(
                formatter,
                "natural surface dimensions must be positive multiples of 8 and at least 8x8 (got {width}x{height})"
            ),
            Self::NaturalMapDimensionsOverflow { width, height } => write!(
                formatter,
                "natural surface dimensions overflow the climate map (got {width}x{height})"
            ),
            Self::MissingNaturalBiome { x, y } => {
                write!(formatter, "natural map has no supported biome at ({x}, {y})")
            }
            Self::UnsupportedNaturalRuleset => {
                formatter.write_str("natural surface generation requires the NewOrigins ruleset")
            }
            Self::RegionSetup(error) => write!(formatter, "region setup failed: {error:?}"),
            Self::MissingGeneratedName(index) => {
                write!(formatter, "generated region name {index} does not exist")
            }
            Self::RegionNumberOverflow => {
                formatter.write_str("region number exceeds the supported range")
            }
        }
    }
}

impl std::error::Error for WorldGenerationError {}

impl From<crate::model::RegionArrayError> for WorldGenerationError {
    fn from(error: crate::model::RegionArrayError) -> Self {
        Self::RegionArray(error)
    }
}

impl From<SetupError> for WorldGenerationError {
    fn from(error: SetupError) -> Self {
        Self::RegionSetup(error)
    }
}
