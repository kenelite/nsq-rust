//! NSQLookupd server implementation

use std::sync::Arc;
use std::collections::HashMap;
use tokio::net::TcpListener;
use axum::{
    extract::State,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use parking_lot::RwLock;
use nsq_common::{Metrics, Result, NsqError, init_logging, NsqlookupdConfig};

/// Producer registration information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Producer {
    pub remote_address: String,
    pub hostname: String,
    pub broadcast_address: String,
    pub tcp_port: u16,
    pub http_port: u16,
    pub version: String,
    pub last_update: chrono::DateTime<chrono::Utc>,
}

/// Registration database
#[derive(Debug)]
pub struct RegistrationDB {
    /// Topic -> Producers
    topics: Arc<RwLock<HashMap<String, Vec<Producer>>>>,
    /// Channel -> Producers
    _channels: Arc<RwLock<HashMap<String, Vec<Producer>>>>,
    /// Tombstoned topics
    _tombstones: Arc<RwLock<HashMap<String, chrono::DateTime<chrono::Utc>>>>,
}

impl RegistrationDB {
    pub fn new() -> Self {
        Self {
            topics: Arc::new(RwLock::new(HashMap::new())),
            _channels: Arc::new(RwLock::new(HashMap::new())),
            _tombstones: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub fn register_producer(&self, topic: String, producer: Producer) {
        self.topics.write().entry(topic).or_insert_with(Vec::new).push(producer);
    }
    
    pub fn get_producers(&self, topic: &str) -> Vec<Producer> {
        self.topics.read().get(topic).cloned().unwrap_or_default()
    }
    
    pub fn get_all_topics(&self) -> Vec<String> {
        self.topics.read().keys().cloned().collect()
    }
}

/// NSQLookupd server
pub struct NsqlookupdServer {
    /// Server configuration
    config: NsqlookupdConfig,
    /// Metrics collector
    _metrics: Metrics,
    /// Registration database
    db: Arc<RegistrationDB>,
}

impl NsqlookupdServer {
    /// Create a new NSQLookupd server
    pub fn new(config: NsqlookupdConfig) -> Result<Self> {
        // Initialize logging
        init_logging(&config.base)?;
        
        // Initialize metrics
        let metrics = Metrics::new(&config.base)?;
        
        Ok(Self {
            config,
            _metrics: metrics,
            db: Arc::new(RegistrationDB::new()),
        })
    }
    
    /// Start the server
    pub async fn start(&mut self) -> Result<()> {
        tracing::info!("Starting NSQLookupd server");
        
        // Parse HTTP address
        let http_addr = self.config.http_address.parse::<std::net::SocketAddr>()
            .map_err(|e| NsqError::Validation(format!("Invalid HTTP address: {}", e)))?;
        
        // Create HTTP listener
        let listener = TcpListener::bind(http_addr).await
            .map_err(|e| NsqError::Io(e))?;
        
        tracing::info!("HTTP server listening on {}", http_addr);
        
        // Create router
        let app = self.create_router();
        
        // Start server
        axum::serve(listener, app).await
            .map_err(|e| NsqError::Io(e))?;
        
        Ok(())
    }
    
    /// Create HTTP router
    fn create_router(&self) -> Router {
        let server = Arc::new(self.clone());
        
        Router::new()
            .route("/ping", get(|| async { "OK" }))
            .route("/info", get(Self::handle_info))
            .route("/lookup", get(Self::handle_lookup))
            .route("/topics", get(Self::handle_topics))
            .route("/channels", get(Self::handle_channels))
            .route("/nodes", get(Self::handle_nodes))
            .route("/topic/create", post(Self::handle_topic_create))
            .route("/topic/delete", post(Self::handle_topic_delete))
            .route("/channel/create", post(Self::handle_channel_create))
            .route("/channel/delete", post(Self::handle_channel_delete))
            .route("/tombstone_topic_producer", post(Self::handle_tombstone))
            .with_state(server)
    }
    
    /// Handle info endpoint
    async fn handle_info() -> Json<serde_json::Value> {
        Json(serde_json::json!({
            "version": env!("CARGO_PKG_VERSION")
        }))
    }
    
    /// Handle lookup endpoint
    async fn handle_lookup(State(server): State<Arc<NsqlookupdServer>>) -> Json<serde_json::Value> {
        let topics = server.db.get_all_topics();
        Json(serde_json::json!({
            "topics": topics
        }))
    }
    
    /// Handle topics endpoint
    async fn handle_topics(State(server): State<Arc<NsqlookupdServer>>) -> Json<serde_json::Value> {
        let topics = server.db.get_all_topics();
        Json(serde_json::json!({
            "topics": topics
        }))
    }
    
    /// Handle channels endpoint
    async fn handle_channels() -> Json<serde_json::Value> {
        Json(serde_json::json!({
            "channels": []
        }))
    }
    
    /// Handle nodes endpoint
    async fn handle_nodes() -> Json<serde_json::Value> {
        Json(serde_json::json!({
            "producers": []
        }))
    }
    
    /// Handle topic create endpoint
    async fn handle_topic_create() -> &'static str {
        "OK"
    }
    
    /// Handle topic delete endpoint
    async fn handle_topic_delete() -> &'static str {
        "OK"
    }
    
    /// Handle channel create endpoint
    async fn handle_channel_create() -> &'static str {
        "OK"
    }
    
    /// Handle channel delete endpoint
    async fn handle_channel_delete() -> &'static str {
        "OK"
    }
    
    /// Handle tombstone endpoint
    async fn handle_tombstone() -> &'static str {
        "OK"
    }
}

impl Clone for NsqlookupdServer {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            _metrics: Metrics::new(&self.config.base).unwrap_or_else(|_| {
                Metrics::new(&nsq_common::BaseConfig::default()).unwrap()
            }),
            db: self.db.clone(),
        }
    }
}

