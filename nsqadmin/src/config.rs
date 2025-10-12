//! NSQAdmin configuration

use nsq_common::NsqadminConfig;
use clap::Parser;
use std::path::PathBuf;

/// NSQAdmin command line arguments
#[derive(Parser, Debug)]
#[command(name = "nsqadmin")]
#[command(about = "NSQ admin web interface")]
pub struct Args {
    /// HTTP address to listen on
    #[arg(long, default_value = "0.0.0.0:4171")]
    pub http_address: String,
    
    /// Lookupd HTTP addresses
    #[arg(long)]
    pub lookupd_http_addresses: Vec<String>,
    
    /// NSQd HTTP addresses
    #[arg(long)]
    pub nsqd_http_addresses: Vec<String>,
    
    /// Template directory
    #[arg(long)]
    pub template_dir: Option<PathBuf>,
    
    /// Static directory
    #[arg(long)]
    pub static_dir: Option<PathBuf>,
    
    /// Development static directory
    #[arg(long)]
    pub dev_static_dir: Option<PathBuf>,
    
    /// Graphite URL
    #[arg(long)]
    pub graphite_url: Option<String>,
    
    /// Proxy graph queries
    #[arg(long)]
    pub proxy_graphite: bool,
    
    /// Notification HTTP endpoint
    #[arg(long)]
    pub notification_http_endpoint: Option<String>,
    
    /// Log level
    #[arg(long, default_value = "info")]
    pub log_level: String,
    
    /// Log format
    #[arg(long, default_value = "text")]
    pub log_format: String,
}

impl From<Args> for NsqadminConfig {
    fn from(args: Args) -> Self {
        Self {
            base: nsq_common::BaseConfig {
                log_level: args.log_level,
                log_format: args.log_format,
                statsd_address: None,
                statsd_prefix: "nsqadmin".to_string(),
            },
            http_address: args.http_address,
            lookupd_http_addresses: if args.lookupd_http_addresses.is_empty() {
                vec!["127.0.0.1:4161".to_string()]
            } else {
                args.lookupd_http_addresses
            },
            nsqd_http_addresses: args.nsqd_http_addresses,
            template_dir: args.template_dir,
            static_dir: args.static_dir,
            dev_static_dir: args.dev_static_dir,
            graphite_url: args.graphite_url,
            proxy_graphite: args.proxy_graphite,
            notification_http_endpoint: args.notification_http_endpoint,
        }
    }
}
