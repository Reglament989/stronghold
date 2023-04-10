use std::path::Path;

use config::{Config as MasterConfig, File};
use serde::{Deserialize, Serialize};

lazy_static::lazy_static! {
    pub static ref SETTINGS: Config = Config::default();
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub database: Database,
}

#[derive(Serialize, Deserialize)]
pub struct Database {
    pub uri: String,
}

impl Default for Config {
    fn default() -> Self {
        MasterConfig::builder()
            .add_source(config::Environment::default().prefix("APP"))
            .add_source(File::from(Path::new("config.toml")))
            .build()
            .unwrap()
            .try_deserialize()
            .unwrap()
    }
}
