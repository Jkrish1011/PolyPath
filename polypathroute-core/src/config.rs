// Loads config.yaml    
use toml;
use std::{
    collections::HashMap,
    fs,
};
use serde::Deserialize;
use anyhow::Result;

#[derive(Deserialize, Debug, Clone)]
struct GlobalConfig {
    update_interval: u8,
    cache_ttl: u8,
    log_level: String
}

#[derive(Deserialize, Debug, Clone)]
pub struct Pair {
    pub source_chain: String,
    pub destination_chain: String,
    pub source_token_name: String,
    pub source_address: String,
    pub destination_address: String,
    pub destination_token_name: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct BridgeConfig {
    pub base_url: String,
    pub chains: Vec<String>,
    pub pairs: Option<Vec<Pair>>,
    pub extra: Option<HashMap<String, toml::Value>>
}

#[derive(Deserialize, Debug, Clone)]
pub struct ConfigManager {
    pub global: GlobalConfig,
    pub bridges: HashMap<String, BridgeConfig>
}

impl ConfigManager {
    pub fn new(config_path: &str) -> Self {
        let path = config_path; 
        let s = fs::read_to_string(path).unwrap();
        let cfg = toml::from_str::<ConfigManager>(&s).unwrap();
        cfg
    }
}