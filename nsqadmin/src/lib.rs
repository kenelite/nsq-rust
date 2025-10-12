//! NSQAdmin - Admin Web Interface
//! 
//! Web interface for managing NSQ topics, channels, and monitoring

pub mod server;
pub mod config;

pub use server::*;
pub use config::*;
