mod cache;
mod config;
mod logging;
mod persistence;
mod errors;

use crate::cache::CacheManager;
use crate::config::ConfigManager;
pub use crate::logging::LoggingManager;
use crate::persistence::PersistenceManager;
use crate::errors::Errors;

#[derive(Debug, Clone)]
pub struct CoreContext {
    pub cache_manager: CacheManager,
    pub config_manager: ConfigManager,
    pub logging_manager: LoggingManager,
    pub persisence_manager: PersistenceManager
}

impl CoreContext {
    pub fn new(config_path: &str) -> Self {
        Self {
            cache_manager: CacheManager::new(),
            config_manager: ConfigManager::new(config_path),
            logging_manager: LoggingManager {  },
            persisence_manager: PersistenceManager::new()
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_core_context() {
        let coreVal: CoreContext = CoreContext::new("./src/config/config.toml");
        println!("coreValue: {:?}", &coreVal);
    }
}
