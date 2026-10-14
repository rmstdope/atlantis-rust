mod legacy;
mod legacy_write;

pub use legacy::{load_legacy, LoadedGame, SaveError};
pub use legacy_write::save_legacy;
