//! NSQd configuration

pub use nsq_common::NsqdConfig;
use clap::Parser;
use std::path::PathBuf;

/// NSQd command line arguments
#[derive(Parser, Debug)]
#[command(name = "nsqd")]
#[command(about = "NSQ message queue daemon")]
pub struct Args {
    /// TCP address to listen on
    #[arg(long, default_value = "0.0.0.0:4150")]
    pub tcp_address: String,
    
    /// HTTP address to listen on
    #[arg(long, default_value = "0.0.0.0:4151")]
    pub http_address: String,
    
    /// HTTPS address to listen on
    #[arg(long)]
    pub https_address: Option<String>,
    
    /// TCP unix socket path
    #[arg(long)]
    pub tcp_socket_path: Option<String>,
    
    /// HTTP unix socket path
    #[arg(long)]
    pub http_socket_path: Option<String>,
    
    /// HTTPS unix socket path
    #[arg(long)]
    pub https_socket_path: Option<String>,
    
    /// Data directory
    #[arg(long, default_value = "./data")]
    pub data_path: PathBuf,
    
    /// Memory queue size
    #[arg(long, default_value = "10000")]
    pub mem_queue_size: usize,
    
    /// Maximum message size
    #[arg(long, default_value = "1048576")]
    pub max_msg_size: usize,
    
    /// Maximum body size
    #[arg(long, default_value = "5242880")]
    pub max_body_size: usize,
    
    /// Maximum request timeout
    #[arg(long, default_value = "60000")]
    pub max_req_timeout: u64,
    
    /// Maximum message timeout
    #[arg(long, default_value = "900000")]
    pub max_msg_timeout: u64,
    
    /// Message timeout
    #[arg(long, default_value = "60000")]
    pub msg_timeout: u64,
    
    /// Maximum output buffer size
    #[arg(long, default_value = "16384")]
    pub max_output_buffer_size: usize,
    
    /// Maximum output buffer timeout
    #[arg(long, default_value = "250")]
    pub max_output_buffer_timeout: u64,
    
    /// TLS certificate file
    #[arg(long)]
    pub tls_cert: Option<PathBuf>,
    
    /// TLS key file
    #[arg(long)]
    pub tls_key: Option<PathBuf>,
    
    /// TLS root CA file
    #[arg(long)]
    pub tls_root_ca_file: Option<PathBuf>,
    
    /// TLS minimum version
    #[arg(long, default_value = "1.2")]
    pub tls_min_version: String,
    
    /// Statsd address
    #[arg(long)]
    pub statsd_address: Option<String>,
    
    /// Statsd prefix
    #[arg(long, default_value = "nsq")]
    pub statsd_prefix: String,
    
    /// Log level
    #[arg(long, default_value = "info")]
    pub log_level: String,
    
    /// Log format
    #[arg(long, default_value = "text")]
    pub log_format: String,
    
    /// Lookupd TCP addresses
    #[arg(long)]
    pub lookupd_tcp_addresses: Vec<String>,
    
    /// Disable HTTP interface
    #[arg(long)]
    pub disable_http: bool,
    
    /// Disable HTTPS interface
    #[arg(long)]
    pub disable_https: bool,
    
    /// E2E processing latency percentiles
    #[arg(long)]
    pub e2e_processing_latency_percentile: Vec<f64>,
}

impl From<Args> for NsqdConfig {
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
            https_address: args.https_address,
            tcp_socket_path: args.tcp_socket_path,
            http_socket_path: args.http_socket_path,
            https_socket_path: args.https_socket_path,
            data_path: args.data_path,
            mem_queue_size: args.mem_queue_size,
            max_msg_size: args.max_msg_size,
            max_body_size: args.max_body_size,
            max_req_timeout: args.max_req_timeout,
            max_msg_timeout: args.max_msg_timeout,
            msg_timeout: args.msg_timeout,
            max_output_buffer_size: args.max_output_buffer_size,
            max_output_buffer_timeout: args.max_output_buffer_timeout,
            tls_cert: args.tls_cert,
            tls_key: args.tls_key,
            tls_root_ca_file: args.tls_root_ca_file,
            tls_min_version: args.tls_min_version,
            e2e_processing_latency_percentile: if args.e2e_processing_latency_percentile.is_empty() {
                vec![0.5, 0.75, 0.9, 0.95, 0.99]
            } else {
                args.e2e_processing_latency_percentile
            },
            lookupd_tcp_addresses: args.lookupd_tcp_addresses,
            disable_http: args.disable_http,
            disable_https: args.disable_https,
        }
    }
}
