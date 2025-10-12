//! Configuration management

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::errors::{NsqError, Result};

/// Base configuration for NSQ components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseConfig {
    /// Log level
    pub log_level: String,
    /// Log format (json, text)
    pub log_format: String,
    /// Statsd address
    pub statsd_address: Option<String>,
    /// Statsd prefix
    pub statsd_prefix: String,
}

impl Default for BaseConfig {
    fn default() -> Self {
        Self {
            log_level: "info".to_string(),
            log_format: "text".to_string(),
            statsd_address: None,
            statsd_prefix: "nsq".to_string(),
        }
    }
}

/// NSQd configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NsqdConfig {
    #[serde(flatten)]
    pub base: BaseConfig,
    
    /// TCP address to listen on
    pub tcp_address: String,
    /// HTTP address to listen on
    pub http_address: String,
    /// HTTPS address to listen on
    pub https_address: Option<String>,
    
    /// Unix socket paths
    pub tcp_socket_path: Option<String>,
    pub http_socket_path: Option<String>,
    pub https_socket_path: Option<String>,
    
    /// Data directory
    pub data_path: PathBuf,
    
    /// Memory queue size
    pub mem_queue_size: usize,
    
    /// Maximum message size
    pub max_msg_size: usize,
    /// Maximum body size
    pub max_body_size: usize,
    
    /// Maximum request timeout
    pub max_req_timeout: u64,
    /// Maximum message timeout
    pub max_msg_timeout: u64,
    /// Default message timeout
    pub msg_timeout: u64,
    
    /// Maximum output buffer size
    pub max_output_buffer_size: usize,
    /// Maximum output buffer timeout
    pub max_output_buffer_timeout: u64,
    
    /// TLS configuration
    pub tls_cert: Option<PathBuf>,
    pub tls_key: Option<PathBuf>,
    pub tls_root_ca_file: Option<PathBuf>,
    pub tls_min_version: String,
    
    /// E2E processing latency percentile
    pub e2e_processing_latency_percentile: Vec<f64>,
    
    /// Lookupd TCP addresses
    pub lookupd_tcp_addresses: Vec<String>,
    
    /// Disable HTTP interface
    pub disable_http: bool,
    /// Disable HTTPS interface
    pub disable_https: bool,
}

impl Default for NsqdConfig {
    fn default() -> Self {
        Self {
            base: BaseConfig::default(),
            tcp_address: "0.0.0.0:4150".to_string(),
            http_address: "0.0.0.0:4151".to_string(),
            https_address: None,
            tcp_socket_path: None,
            http_socket_path: None,
            https_socket_path: None,
            data_path: PathBuf::from("./data"),
            mem_queue_size: 10000,
            max_msg_size: 1024 * 1024, // 1MB
            max_body_size: 5 * 1024 * 1024, // 5MB
            max_req_timeout: 60 * 1000, // 60 seconds
            max_msg_timeout: 15 * 60 * 1000, // 15 minutes
            msg_timeout: 60 * 1000, // 60 seconds
            max_output_buffer_size: 16 * 1024, // 16KB
            max_output_buffer_timeout: 250, // 250ms
            tls_cert: None,
            tls_key: None,
            tls_root_ca_file: None,
            tls_min_version: "1.2".to_string(),
            e2e_processing_latency_percentile: vec![0.5, 0.75, 0.9, 0.95, 0.99],
            lookupd_tcp_addresses: Vec::new(),
            disable_http: false,
            disable_https: false,
        }
    }
}

/// NSQLookupd configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NsqlookupdConfig {
    #[serde(flatten)]
    pub base: BaseConfig,
    
    /// TCP address to listen on
    pub tcp_address: String,
    /// HTTP address to listen on
    pub http_address: String,
    
    /// Unix socket paths
    pub tcp_socket_path: Option<String>,
    pub http_socket_path: Option<String>,
    
    /// Inactive producer timeout
    pub inactive_producer_timeout: u64,
    /// Tombstone lifetime
    pub tombstone_lifetime: u64,
}

impl Default for NsqlookupdConfig {
    fn default() -> Self {
        Self {
            base: BaseConfig::default(),
            tcp_address: "0.0.0.0:4160".to_string(),
            http_address: "0.0.0.0:4161".to_string(),
            tcp_socket_path: None,
            http_socket_path: None,
            inactive_producer_timeout: 300 * 1000, // 5 minutes
            tombstone_lifetime: 45 * 1000, // 45 seconds
        }
    }
}

/// NSQAdmin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NsqadminConfig {
    #[serde(flatten)]
    pub base: BaseConfig,
    
    /// HTTP address to listen on
    pub http_address: String,
    
    /// Lookupd HTTP addresses
    pub lookupd_http_addresses: Vec<String>,
    
    /// NSQd HTTP addresses
    pub nsqd_http_addresses: Vec<String>,
    
    /// Template directory
    pub template_dir: Option<PathBuf>,
    /// Static directory
    pub static_dir: Option<PathBuf>,
    /// Development static directory
    pub dev_static_dir: Option<PathBuf>,
    
    /// Graphite URL
    pub graphite_url: Option<String>,
    /// Proxy graph queries
    pub proxy_graphite: bool,
    
    /// Notification HTTP endpoint
    pub notification_http_endpoint: Option<String>,
}

impl Default for NsqadminConfig {
    fn default() -> Self {
        Self {
            base: BaseConfig::default(),
            http_address: "0.0.0.0:4171".to_string(),
            lookupd_http_addresses: vec!["127.0.0.1:4161".to_string()],
            nsqd_http_addresses: Vec::new(),
            template_dir: None,
            static_dir: None,
            dev_static_dir: None,
            graphite_url: None,
            proxy_graphite: false,
            notification_http_endpoint: None,
        }
    }
}

/// Load configuration from file
pub fn load_config<T>(path: &str) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let settings = config::Config::builder()
        .add_source(config::File::with_name(path))
        .add_source(config::Environment::with_prefix("NSQ"))
        .build()
        .map_err(|e| NsqError::Config(e.to_string()))?;
    
    settings.try_deserialize()
        .map_err(|e| NsqError::Config(e.to_string()))
}

/// Save configuration to file
pub fn save_config<T>(config: &T, path: &str) -> Result<()>
where
    T: Serialize,
{
    let content = serde_json::to_string_pretty(config)
        .map_err(|e| NsqError::Config(e.to_string()))?;
    
    std::fs::write(path, content)
        .map_err(|e| NsqError::Config(e.to_string()))?;
    
    Ok(())
}
