mod adapters;

use polypathroute_core::{CoreContext, LoggingManager};

#[derive(Debug)]
pub struct DalContext {
    core: CoreContext
}

impl DalContext {
    fn new(path: &str) -> DalContext {
        DalContext {
            core: CoreContext::new(path)    
        }
    }

    pub fn create_adapter(&self, adapter_name: &str) -> adapters::DynBridgeAdapter {
        adapters::create_adapter(adapter_name).unwrap()
    }

    pub fn logger(&self) -> &LoggingManager {
        &self.core.logging_manager
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let dal_context = DalContext::new("./src/config/config.toml");

        println!("{:?}", dal_context.core.config_manager.bridges.get("stargate").unwrap().pairs);

        let stargate_adapter = dal_context.create_adapter("stargate");
        dal_context.logger().info("Created Stargate Adapter!").unwrap();
        // println!("fetch_metrics: {:?}", stargate_adapter.fetch_metrics("ethereum", "polygon", "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48", "0x3c499c542cef5e3811e1192ce70d8cc03d5c3359", "1000000", "990000", "0xca699201b15ccef3b8c4012e28570cc5500d9f9a", "0xca699201b15ccef3b8c4012e28570cc5500d9f9a").unwrap());
        // println!("fetch_metrics: {:?}", stargate_adapter.fetch_metrics("base", "arbitrum", "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913", "0xaf88d065e77c8cC2239327C5EDb3A432268e5831", "1000000", "990000", "0xca699201b15ccef3b8c4012e28570cc5500d9f9a", "0xca699201b15ccef3b8c4012e28570cc5500d9f9a").unwrap());
        // println!("fetch_metrics: {:?}", stargate_adapter.fetch_metrics("base", "polygon", "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913", "0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359", "1000000", "990000", "0xca699201b15ccef3b8c4012e28570cc5500d9f9a", "0xca699201b15ccef3b8c4012e28570cc5500d9f9a").unwrap());
        
        
    }
}


