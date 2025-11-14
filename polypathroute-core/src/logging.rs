// Provides tracing::span log context

use tracing::{event, Level};
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct LoggingManager;

impl LoggingManager {

    pub fn info(&self, value: &str) -> Result<()> {
        event!(Level::INFO, value);
        Ok(())
    }

    // debug, warn, error
    pub fn debug(&self, value: &str) -> Result<()> {
        event!(Level::DEBUG, value);
        Ok(())
    }

    pub fn warn(&self, value: &str) -> Result<()> {
        event!(Level::WARN, value);
        Ok(())
    }

    pub fn error(&self, value: &str) -> Result<()> {
        event!(Level::ERROR, value);
        Ok(())
    }
}