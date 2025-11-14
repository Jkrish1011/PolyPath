use super::{
    BridgeAdapter,
    BridgeEdge
};

use std::collections::HashMap;
use serde_json::Value;
use anyhow::Result;

pub struct WormholeAdapter {
    pub name: String,
    private_key: String,
    pub base_url: String
}

impl WormholeAdapter {
    pub fn new() -> Self {
        Self {
            name: "stargate".to_string(),
            private_key: "".to_string(),
            base_url: "".to_string()
        }
    }
}

impl BridgeAdapter for WormholeAdapter {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn supported_pairs(&self) -> HashMap<String, String> {
        HashMap::new()
    }

    fn is_supported_pair(&self) -> bool {
        true
    }

    fn fetch_metrics(&self, src_chain: &str, dst_chain: &str, src_token: &str, dst_token: &str,
        src_amount: &str, dst_amount: &str, src_address: &str, dst_address: &str) -> Result<Value> {    

        Ok(Value::Null)
    }
}