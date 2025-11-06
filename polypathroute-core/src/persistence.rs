// Simple K/V store for data snapshots

use anyhow::Result;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct PersistenceManager {
    store: HashMap<String, String>
}

impl PersistenceManager {

    pub fn new() -> Self {
        Self {
            store: HashMap::new()
        }
    }
 
    pub fn store(&self, key: String, value: String) -> Result<bool>{
        // self.store.set(key, value);
        Ok(true)
    }

    pub fn get(&self, key: String) -> Result<String>{
        // store.get(key);
        Ok("value".to_string())
    }

    pub fn clear(&self, key: String) -> Result<bool>{
        // store.get(key);
        Ok(true)
    }
    
}