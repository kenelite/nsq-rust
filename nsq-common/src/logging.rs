//! Logging infrastructure

use tracing::{Level, Subscriber};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};
use crate::config::BaseConfig;
use crate::errors::Result;

/// Initialize logging based on configuration
pub fn init_logging(config: &BaseConfig) -> Result<()> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            let level = config.log_level.parse::<Level>()
                .unwrap_or(Level::INFO);
            EnvFilter::new(&format!("{}", level))
        });
    
    let registry = Registry::default().with(filter);
    
    // Use try_init to avoid panicking if a global subscriber was already set
    let result = match config.log_format.as_str() {
        "json" => registry
            .with(fmt::layer().with_target(false))
            .try_init(),
        _ => registry
            .with(fmt::layer().with_target(false))
            .try_init(),
    };
    
    // If already set, ignore; otherwise return the error
    if let Err(e) = result {
        if format!("{}", e).contains("already been set") {
            // no-op: another part of the application set it
        } else {
            return Err(crate::errors::NsqError::Config(format!("failed to init logging: {}", e)));
        }
    }
    
    Ok(())
}

/// Create a subscriber for testing
pub fn init_test_logging() -> impl Subscriber {
    Registry::default()
        .with(EnvFilter::new("debug"))
        .with(fmt::layer().with_test_writer())
}
