pub mod stargate;
pub mod wormhole;

use std::collections::HashMap;
use serde::Serialize;
use serde_json::Value;
use anyhow::Result;

#[derive(Serialize, Debug, Clone)]
pub struct BridgeEdge {
    pub from: String,
    pub to: String,
    pub cost: f64,
    pub speed: f64,
    pub liquidity: f64,
    pub risk: f64,
}


pub trait BridgeAdapter {
    fn name(&self) -> String;
    fn supported_pairs(&self) -> HashMap<String, String>;
    fn is_supported_pair(&self) -> bool;
    fn fetch_metrics(&self, src_chain: &str, dst_chain: &str, src_token: &str, dst_token: &str,
        src_amount: &str, dst_amount_min: &str, src_address: &str, dst_address: &str) -> Result<Value>;
}

pub type DynBridgeAdapter = Box<dyn BridgeAdapter + Send + Sync>;

pub fn create_adapter(name: &str) -> Option<DynBridgeAdapter> {
    match name.to_lowercase().as_str() {
        "stargate" => {
            return Some(Box::new(stargate::StargateAdapter::new()));
        }
        "wormhole" => {
            return Some(Box::new(wormhole::WormholeAdapter::new()));
        }
        _ => {
            return None;
        }
    }
}