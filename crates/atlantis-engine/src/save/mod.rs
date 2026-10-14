mod json;
mod legacy;
mod legacy_write;

pub use json::{load_json, load_save, save_json};
pub use legacy::{load_legacy, LoadedGame, SaveError};
pub use legacy_write::save_legacy;
