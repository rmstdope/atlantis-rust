mod setup;

pub(crate) use setup::region_wages;
pub use setup::{
    lair_check, manual_setup, setup_city_market, setup_economy, setup_habitat, setup_pop,
    setup_prods, setup_region, SetupError,
};
