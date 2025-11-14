// Provides async TTL cache API

use std::{collections::HashMap, hash::Hash};
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct CacheManager {
    dict: HashMap<String, String>
}

const DEFAULT_TTL: u64 = 3600;

impl CacheManager {

    pub fn new() -> Self {
        Self {
            dict: HashMap::new()
        }
    }

    pub fn set(&mut self, key: String, value: String, ttl: Option<u64>) -> Result<bool> {
        self.dict.insert(key, value);
        Ok(true)
    }

    pub fn get(&self, key: String) -> Result<&String> {
        Ok(self.dict.get(&key).unwrap())
    }

    pub fn remove(&mut self, key: String) -> Result<bool> {
        self.dict.remove_entry(&key);
        Ok(true)
    }

    pub fn clear(&mut self) -> Result<bool> {
        self.dict.clear();
        Ok(true)
    }
}