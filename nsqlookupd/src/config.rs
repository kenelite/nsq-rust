//! NSQLookupd configuration

use nsq_common::NsqlookupdConfig;
use clap::Parser;

/// NSQLookupd command line arguments
#[derive(Parser, Debug)]
#[command(name = "nsqlookupd")]
#[command(about = "NSQ service discovery daemon")]
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
}

impl From<Args> for NsqlookupdConfig {
    fn from(args: Args) -> Self {
        Self {
            base: nsq_common::BaseConfig {
                log_level: args.log_level,
                log_format: args.log_format,
                statsd_address: None,
                statsd_prefix: "nsqlookupd".to_string(),
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

