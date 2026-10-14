use crate::economy::SetupError;

pub mod classic;
mod names;
pub mod settlements;

pub use settlements::{make_starting_city, TownType};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldGenerationError {
    RegionArray(crate::model::RegionArrayError),
    InvalidSurfaceDimensions { width: i32, height: i32 },
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
