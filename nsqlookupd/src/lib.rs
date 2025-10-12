//! NSQLookupd - Service Discovery Daemon
//! 
//! Service discovery daemon for NSQ

pub mod server;
pub mod config;

pub use server::*;
pub use config::*;

