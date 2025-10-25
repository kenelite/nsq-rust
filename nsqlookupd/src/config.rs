//! NSQLookupd configuration

use nsq_common::NsqlookupdConfig;
use clap::Parser;
use std::net::SocketAddr;

/// NSQLookupd command line arguments
#[derive(Parser, Debug)]
#[command(name = "nsqlookupd")]
#[command(about = "NSQ service discovery daemon")]
#[command(version = env!("CARGO_PKG_VERSION"))]
pub struct Args {
    /// TCP address to listen on
    #[arg(long, default_value = "0.0.0.0:4160")]
    pub tcp_address: String,
    
    /// HTTP address to listen on
    #[arg(long, default_value = "0.0.0.0:4161")]
    pub http_address: String,
    
    /// TCP unix socket path
    #[arg(long)]
    pub tcp_socket_path: Option<String>,
    
    /// HTTP unix socket path
    #[arg(long)]
    pub http_socket_path: Option<String>,
    
    /// Inactive producer timeout (ms)
    #[arg(long, default_value = "300000")]
    pub inactive_producer_timeout: u64,
    
    /// Tombstone lifetime (ms)
    #[arg(long, default_value = "45000")]
    pub tombstone_lifetime: u64,
    
    /// Log level
    #[arg(long, default_value = "info")]
    pub log_level: String,
    
    /// Log format
    #[arg(long, default_value = "text")]
    pub log_format: String,
    
    /// Broadcast address for producers
    #[arg(long)]
    pub broadcast_address: Option<String>,
    
    /// Statsd address
    #[arg(long)]
    pub statsd_address: Option<String>,
    
    /// Statsd prefix
    #[arg(long, default_value = "nsqlookupd")]
    pub statsd_prefix: String,
}

impl Args {
    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate TCP address
        if !self.tcp_address.is_empty() {
            self.tcp_address.parse::<SocketAddr>()
                .map_err(|e| format!("Invalid TCP address '{}': {}", self.tcp_address, e))?;
        }
        
        // Validate HTTP address
        if !self.http_address.is_empty() {
            self.http_address.parse::<SocketAddr>()
                .map_err(|e| format!("Invalid HTTP address '{}': {}", self.http_address, e))?;
        }
        
        // Validate timeout values
        if self.inactive_producer_timeout == 0 {
            return Err("inactive_producer_timeout must be greater than 0".to_string());
        }
        
        if self.tombstone_lifetime == 0 {
            return Err("tombstone_lifetime must be greater than 0".to_string());
        }
        
        // Validate log level
        match self.log_level.as_str() {
            "trace" | "debug" | "info" | "warn" | "error" => {},
            _ => return Err(format!("Invalid log level '{}'. Must be one of: trace, debug, info, warn, error", self.log_level)),
        }
        
        // Validate log format
        match self.log_format.as_str() {
            "text" | "json" => {},
            _ => return Err(format!("Invalid log format '{}'. Must be one of: text, json", self.log_format)),
        }
        
        Ok(())
    }
}

impl From<Args> for NsqlookupdConfig {
    fn from(args: Args) -> Self {
        Self {
            base: nsq_common::BaseConfig {
                log_level: args.log_level,
                log_format: args.log_format,
                statsd_address: args.statsd_address,
                statsd_prefix: args.statsd_prefix,
            },
            tcp_address: args.tcp_address,
            http_address: args.http_address,
            tcp_socket_path: args.tcp_socket_path,
            http_socket_path: args.http_socket_path,
            inactive_producer_timeout: args.inactive_producer_timeout,
            tombstone_lifetime: args.tombstone_lifetime,
        }
    }
}

