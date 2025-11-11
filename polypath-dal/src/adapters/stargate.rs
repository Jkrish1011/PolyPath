use super::{
    BridgeAdapter,
    BridgeEdge
};

use std::collections::HashMap;
use reqwest::blocking::Client;
use serde_json::Value;
use anyhow::Result;

pub struct StargateAdapter {
    pub name: String,
    private_key: String,
    pub base_url: String
}

impl StargateAdapter {
    pub fn new() -> Self {
        Self {
            name: "stargate".to_string(),
            private_key: "".to_string(),
            base_url: "".to_string()
        }
    }
}

impl BridgeAdapter for StargateAdapter {
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
        let client = Client::new();
        let params = [
            ("srcChainKey", src_chain),
            ("dstChainKey", dst_chain),
            ("srcToken", src_token),
            ("dstToken", dst_token),
            ("srcAmount", src_amount),
            ("dstAmountMin", dst_amount),
            ("srcAddress", src_address),
            ("dstAddress", dst_address),
        ];

        let response: Value = client
            .get("https://stargate.finance/api/v1/quotes")
            .query(&params)
            .send()?
            .json()?;

        let quote = response
                    .get("quotes")
                    .and_then(|quotes| quotes.as_array())
                    .and_then(|quotes| quotes.first())
                    .ok_or_else(|| anyhow::anyhow!("No quotes found in the response!"));
        
        let quote = quote.unwrap();

        let src_chain_key = quote
                                    .get("srcChainKey")
                                    .and_then(|v| v.as_str());
        
        let dst_chain_key = quote
                                    .get("dstChainKey")
                                    .and_then(|v| v.as_str());

        let cost = quote
                            .get("fees")
                            .and_then(|fees| fees.as_array())
                            .map(|fees| {
                                fees.iter()
                                    .filter_map(|fee| {
                                        fee.get("amount")
                                            .and_then(|v| v.as_str())
                                            .and_then(|s| s.parse::<f64>().ok())
                                    })
                                    .sum::<f64>()
                            })
                            .unwrap_or(0.0);
        
        let speed = quote
                            .get("duration")
                            .and_then(|d| d.get("estimated"))
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0);
        
        let liquidity = quote.get("dstAmount")
                                    .and_then(|v| v.as_str())
                                    .and_then(|s| s.parse::<f64>().ok())
                                    .or_else(|| {
                                        quote.get("srcAmount")
                                                .and_then(|v| v.as_str())
                                                .and_then(|s| s.parse::<f64>().ok())
                                    });
        
        let risk = if speed > 0.0 {
            (speed * 10.0).min(1000.0)
        } else {
            500.0
        };

        let bridge_edge = BridgeEdge {
            from: src_chain_key.unwrap().to_string(),
            to: dst_chain_key.unwrap().to_string(),
            cost: cost,
            speed: speed,
            liquidity: liquidity.unwrap(),
            risk: risk
        };

        Ok(serde_json::to_value(&bridge_edge)?)
    }
}