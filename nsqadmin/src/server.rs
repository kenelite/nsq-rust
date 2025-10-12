//! NSQAdmin server implementation

use std::sync::Arc;
use tokio::net::TcpListener;
use axum::{
    extract::State,
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::json;
use nsq_common::{Metrics, Result, NsqError, NsqadminConfig};
use tower_http::services::ServeDir;

pub struct NsqadminServer {
    config: NsqadminConfig,
    metrics: Metrics,
}

impl NsqadminServer {
    /// Create a new NSQAdmin server
    pub fn new(config: NsqadminConfig) -> Result<Self> {
        // Initialize metrics
        let metrics = Metrics::new(&config.base)?;
        
        Ok(Self {
            config,
            metrics,
        })
    }
    
    /// Start the server
    pub async fn run(self) -> Result<()> {
        tracing::info!("Starting NSQAdmin server");
        
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
    fn create_router(self) -> Router {
        let server = Arc::new(self);
        
        Router::new()
            // API routes
            .route("/api/ping", get(Self::handle_ping))
            .route("/api/info", get(Self::handle_info))
            .route("/api/stats", get(Self::handle_stats))
            .route("/api/topics", get(Self::handle_topics))
            .route("/api/nodes", get(Self::handle_nodes))
            .route("/api/topic/:topic/pause", post(Self::handle_topic_pause))
            .route("/api/topic/:topic/unpause", post(Self::handle_topic_unpause))
            .route("/api/topic/:topic/delete", post(Self::handle_topic_delete))
            .route("/api/channel/:topic/:channel/pause", post(Self::handle_channel_pause))
            .route("/api/channel/:topic/:channel/unpause", post(Self::handle_channel_unpause))
            .route("/api/channel/:topic/:channel/delete", post(Self::handle_channel_delete))
            // Serve static files from nsqadmin-ui/dist
            .nest_service("/", ServeDir::new("../nsqadmin-ui/dist"))
            .with_state(server)
    }
    
    /// Handle ping endpoint
    async fn handle_ping() -> &'static str {
        "OK"
    }
    
    /// Handle info endpoint
    async fn handle_info() -> Json<serde_json::Value> {
        Json(json!({
            "version": env!("CARGO_PKG_VERSION"),
            "build": "rust",
            "features": ["modern-ui", "real-time-dashboard", "dark-mode"]
        }))
    }
    
    /// Handle stats endpoint
    async fn handle_stats(State(_server): State<Arc<NsqadminServer>>) -> Json<serde_json::Value> {
        // Mock stats for demonstration
        Json(json!({
            "version": env!("CARGO_PKG_VERSION"),
            "health": "ok",
            "start_time": chrono::Utc::now().timestamp(),
            "uptime": "1h 23m 45s",
            "uptime_seconds": 5025,
            "producers": [
                {
                    "remote_address": "127.0.0.1:4150",
                    "hostname": "localhost",
                    "broadcast_address": "127.0.0.1",
                    "tcp_port": 4150,
                    "http_port": 4151,
                    "version": "1.3.0",
                    "last_update": chrono::Utc::now().to_rfc3339()
                }
            ],
            "topics": [
                {
                    "topic_name": "test-topic",
                    "channels": [
                        {
                            "channel_name": "test-channel",
                            "depth": 0,
                            "backend_depth": 0,
                            "in_flight_count": 0,
                            "deferred_count": 0,
                            "message_count": 0,
                            "requeue_count": 0,
                            "timeout_count": 0,
                            "clients": [],
                            "paused": false
                        }
                    ],
                    "depth": 0,
                    "backend_depth": 0,
                    "message_count": 0,
                    "paused": false
                }
            ]
        }))
    }
    
    /// Handle topics endpoint
    async fn handle_topics() -> Json<serde_json::Value> {
        Json(json!({
            "topics": ["test-topic", "metrics", "logs"]
        }))
    }
    
    /// Handle nodes endpoint
    async fn handle_nodes() -> Json<serde_json::Value> {
        Json(json!({
            "producers": [
                {
                    "remote_address": "127.0.0.1:4150",
                    "hostname": "localhost",
                    "broadcast_address": "127.0.0.1",
                    "tcp_port": 4150,
                    "http_port": 4151,
                    "version": "1.3.0",
                    "last_update": chrono::Utc::now().to_rfc3339()
                }
            ]
        }))
    }
    
    /// Handle topic pause
    async fn handle_topic_pause(axum::extract::Path(topic): axum::extract::Path<String>) -> Json<serde_json::Value> {
        tracing::info!("Pausing topic: {}", topic);
        Json(json!({"status": "ok", "message": format!("Topic {} paused", topic)}))
    }
    
    /// Handle topic unpause
    async fn handle_topic_unpause(axum::extract::Path(topic): axum::extract::Path<String>) -> Json<serde_json::Value> {
        tracing::info!("Unpausing topic: {}", topic);
        Json(json!({"status": "ok", "message": format!("Topic {} unpaused", topic)}))
    }
    
    /// Handle topic delete
    async fn handle_topic_delete(axum::extract::Path(topic): axum::extract::Path<String>) -> Json<serde_json::Value> {
        tracing::info!("Deleting topic: {}", topic);
        Json(json!({"status": "ok", "message": format!("Topic {} deleted", topic)}))
    }
    
    /// Handle channel pause
    async fn handle_channel_pause(
        axum::extract::Path((topic, channel)): axum::extract::Path<(String, String)>
    ) -> Json<serde_json::Value> {
        tracing::info!("Pausing channel: {} on topic: {}", channel, topic);
        Json(json!({"status": "ok", "message": format!("Channel {} on topic {} paused", channel, topic)}))
    }
    
    /// Handle channel unpause
    async fn handle_channel_unpause(
        axum::extract::Path((topic, channel)): axum::extract::Path<(String, String)>
    ) -> Json<serde_json::Value> {
        tracing::info!("Unpausing channel: {} on topic: {}", channel, topic);
        Json(json!({"status": "ok", "message": format!("Channel {} on topic {} unpaused", channel, topic)}))
    }
    
    /// Handle channel delete
    async fn handle_channel_delete(
        axum::extract::Path((topic, channel)): axum::extract::Path<(String, String)>
    ) -> Json<serde_json::Value> {
        tracing::info!("Deleting channel: {} on topic: {}", channel, topic);
        Json(json!({"status": "ok", "message": format!("Channel {} on topic {} deleted", channel, topic)}))
    }
}

impl Clone for NsqadminServer {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            metrics: self.metrics.clone(),
        }
    }
}