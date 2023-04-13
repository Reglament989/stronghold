use std::{net::SocketAddr, path::Path};

use config::{Config as MasterConfig, File};
use serde::{Deserialize, Serialize};

lazy_static::lazy_static! {
    pub static ref SETTINGS: Config = Config::default();
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub database: Database,
    pub api: Api,
    pub federation: Federation,
}

#[derive(Serialize, Deserialize)]
pub struct Api {
    pub addr: SocketAddr,
    pub signing_key: String, // For federation
}

#[derive(Serialize, Deserialize)]
pub struct Federation {
    pub enabled: bool,
    pub forwarder: bool,
    pub addr: SocketAddr,
    pub trusted_servers: Vec<String>, // Public keys of trusted servers includs self
    pub trust_iherit: bool,           // Trust all servers recived from trusted_servers
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
