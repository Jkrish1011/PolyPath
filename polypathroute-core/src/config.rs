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
struct BridgeConfig {
    pub base_url: String,
    pub chains: Vec<String>,
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