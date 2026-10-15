use crate::{economy::SetupError, model::Game, rng::Rng, ruleset::Ruleset};

pub mod classic;
pub mod connections;
mod names;
pub mod natural;
pub mod settlements;

pub use settlements::{make_starting_city, TownType};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SurfaceGenerator {
    Classic,
    Natural,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldGenerationError {
    RegionArray(crate::model::RegionArrayError),
    MissingRegionArray(usize),
    MissingRegionLevel(crate::model::RegionLevelType),
    InvalidSurfaceDimensions { width: i32, height: i32 },
    InvalidNaturalSurfaceDimensions { width: i32, height: i32 },
    NaturalMapDimensionsOverflow { width: i32, height: i32 },
    MissingNaturalBiome { x: i32, y: i32 },
    UnsupportedNaturalRuleset,
    UnsupportedAbyssLevel,
    InvalidGateCount,
    RegionSetup(SetupError),
    MissingGeneratedName(i32),
    RegionNumberOverflow,
}

impl std::fmt::Display for WorldGenerationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RegionArray(error) => error.fmt(formatter),
            Self::MissingRegionArray(index) => {
                write!(formatter, "region array {index} does not exist")
            }
            Self::MissingRegionLevel(level) => {
                write!(formatter, "region array for {level:?} does not exist")
            }
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
            Self::UnsupportedAbyssLevel => {
                formatter.write_str("abyss level generation is not implemented")
            }
            Self::InvalidGateCount => {
                formatter.write_str("gate numbering has gates but no available numbers")
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

/// C++: `game.cpp` `Game::CreateWorld`; builds levels before world connections.
pub(crate) fn create_world(
    game: &mut Game,
    ruleset: &'static dyn Ruleset,
    rng: &mut Rng,
    seed: i32,
    width: i32,
    height: i32,
    nexus_size: i32,
    generator: SurfaceGenerator,
) -> Result<(), WorldGenerationError> {
    let definitions = ruleset.game_defs();
    if definitions.abyss_level != 0 {
        return Err(WorldGenerationError::UnsupportedAbyssLevel);
    }
    let nexus_height = if nexus_size == 1 { 1 } else { 2 };
    classic::create_nexus_level(game, ruleset, rng, nexus_size, nexus_height)?;
    match generator {
        SurfaceGenerator::Classic => {
            classic::create_classic_surface_level(game, ruleset, rng, width, height)?;
        }
        SurfaceGenerator::Natural => {
            natural::create_natural_surface_level(game, ruleset, rng, seed, width, height)?;
        }
    }

    for level in 2..definitions.underworld_levels + 2 {
        classic::create_underworld_level(
            game,
            ruleset,
            rng,
            width / ruleset.level_x_scale(level),
            height / ruleset.level_y_scale(level),
        )?;
    }
    for level in definitions.underworld_levels + 2
        ..definitions.underworld_levels + definitions.underdeep_levels + 2
    {
        classic::create_underdeep_level(
            game,
            ruleset,
            rng,
            width / ruleset.level_x_scale(level),
            height / ruleset.level_y_scale(level),
        )?;
    }
    connections::connect_world(game, ruleset, rng)
}
