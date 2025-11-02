//! NSQAdmin server implementation

use std::sync::Arc;
use std::collections::{HashMap, HashSet};
use tokio::net::TcpListener;
use axum::{
    extract::{State, Path as AxumPath},
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use nsq_common::{Metrics, Result, NsqError, NsqadminConfig};
use tower_http::{
    services::ServeDir,
    cors::{CorsLayer, Any},
};

pub struct NsqadminServer {
    config: NsqadminConfig,
    metrics: Metrics,
    http_client: reqwest::Client,
    start_time: chrono::DateTime<chrono::Utc>,
    start_instant: std::time::Instant,
}

#[derive(Debug, Serialize, Deserialize)]
struct TopicInfo {
    topic_name: String,
    channels: Vec<ChannelInfo>,
    depth: u64,
    backend_depth: u64,
    message_count: u64,
    paused: bool,
    nodes: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChannelInfo {
    channel_name: String,
    depth: u64,
    backend_depth: u64,
    in_flight_count: u64,
    deferred_count: u64,
    message_count: u64,
    requeue_count: u64,
    timeout_count: u64,
    paused: bool,
    clients: Vec<ClientInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ClientInfo {
    client_id: String,
    hostname: String,
    remote_address: String,
}

impl NsqadminServer {
    /// Create a new NSQAdmin server
    pub fn new(config: NsqadminConfig) -> Result<Self> {
        // Initialize metrics
        let metrics = Metrics::new(&config.base)?;
        let http_client = reqwest::Client::new();
        
        Ok(Self {
            config,
            metrics,
            http_client,
            start_time: chrono::Utc::now(),
            start_instant: std::time::Instant::now(),
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
        
        // Configure CORS
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);
        
        Router::new()
            // API routes
            .route("/api/ping", get(Self::handle_ping))
            .route("/api/info", get(Self::handle_info))
            .route("/api/stats", get(Self::handle_stats))
            .route("/api/topics", get(Self::handle_topics))
            .route("/api/topics/:topic", get(Self::handle_topic_detail))
            .route("/api/nodes", get(Self::handle_nodes))
            .route("/api/topic/:topic/pause", post(Self::handle_topic_pause))
            .route("/api/topic/:topic/unpause", post(Self::handle_topic_unpause))
            .route("/api/topic/:topic/delete", post(Self::handle_topic_delete))
            .route("/api/topic/:topic/create", post(Self::handle_topic_create))
            .route("/api/channel/:topic/:channel/pause", post(Self::handle_channel_pause))
            .route("/api/channel/:topic/:channel/unpause", post(Self::handle_channel_unpause))
            .route("/api/channel/:topic/:channel/delete", post(Self::handle_channel_delete))
            .route("/api/channel/:topic/:channel/create", post(Self::handle_channel_create))
            .route("/api/channel/:topic/:channel/empty", post(Self::handle_channel_empty))
            // Serve static files from nsqadmin-ui/dist
            .nest_service("/", ServeDir::new("../nsqadmin-ui/dist"))
            .layer(cors)
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
    async fn handle_stats(State(server): State<Arc<NsqadminServer>>) -> Json<serde_json::Value> {
        // Compute uptime
        let uptime_seconds = server.start_instant.elapsed().as_secs();
        let hours = uptime_seconds / 3600;
        let minutes = (uptime_seconds % 3600) / 60;
        let seconds = uptime_seconds % 60;
        let uptime_display = format!("{}h {}m {}s", hours, minutes, seconds);

        // Aggregate topics and nodes from all sources
        let topics = server.aggregate_topic_stats().await.unwrap_or_default();
        let producers = server.fetch_all_producers().await.unwrap_or_default();

        // Present statistics
        Json(json!({
            "version": env!("CARGO_PKG_VERSION"),
            "health": "ok",
            "start_time": server.start_time.timestamp(),
            "uptime": uptime_display,
            "uptime_seconds": uptime_seconds,
            "producers": producers,
            "topics": topics,
        }))
    }
    
    /// Handle topics endpoint
    async fn handle_topics(State(server): State<Arc<NsqadminServer>>) -> Json<serde_json::Value> {
        let topics = server.aggregate_topic_stats().await.unwrap_or_default();
        Json(json!({
            "topics": topics
        }))
    }
    
    /// Handle topic detail endpoint
    async fn handle_topic_detail(
        State(server): State<Arc<NsqadminServer>>,
        AxumPath(topic): AxumPath<String>
    ) -> Json<serde_json::Value> {
        let topic_info = server.get_topic_detail(&topic).await.unwrap_or_default();
        Json(topic_info)
    }
    
    /// Handle nodes endpoint
    async fn handle_nodes(State(server): State<Arc<NsqadminServer>>) -> Json<serde_json::Value> {
        let producers = server.fetch_all_producers().await.unwrap_or_default();
        Json(json!({
            "producers": producers
        }))
    }

    // --- Helper methods ---
    
    fn normalize_address(addr: &str) -> String {
        if addr.starts_with("http://") || addr.starts_with("https://") {
            addr.to_string()
        } else {
            format!("http://{}", addr)
        }
    }

    /// Get all nsqd HTTP addresses from lookupd and direct config
    async fn get_all_nsqd_addresses(&self) -> Vec<String> {
        let mut addresses = HashSet::new();
        
        // Add directly configured nsqd addresses
        for addr in &self.config.nsqd_http_addresses {
            addresses.insert(Self::normalize_address(addr));
        }
        
        // Query lookupd for all producers
        for lookupd_addr in &self.config.lookupd_http_addresses {
            let base = Self::normalize_address(lookupd_addr);
            let url = format!("{}/nodes", base);
            if let Ok(resp) = self.http_client.get(&url).send().await {
                if let Ok(json) = resp.json::<serde_json::Value>().await {
                    if let Some(arr) = json.get("producers").and_then(|v| v.as_array()) {
                        for producer in arr {
                            if let (Some(addr), Some(port)) = (
                                producer.get("broadcast_address").and_then(|v| v.as_str()),
                                producer.get("http_port").and_then(|v| v.as_u64())
                            ) {
                                addresses.insert(format!("http://{}:{}", addr, port));
                            }
                        }
                    }
                }
            }
        }
        
        addresses.into_iter().collect()
    }

    /// Fetch producers from all sources
    async fn fetch_all_producers(&self) -> std::result::Result<Vec<serde_json::Value>, reqwest::Error> {
        let mut producers_map: HashMap<String, serde_json::Value> = HashMap::new();
        
        // From lookupd
        for addr in &self.config.lookupd_http_addresses {
            let base = Self::normalize_address(addr);
            let url = format!("{}/nodes", base);
            if let Ok(resp) = self.http_client.get(&url).send().await {
                if let Ok(json) = resp.json::<serde_json::Value>().await {
                    if let Some(arr) = json.get("producers").and_then(|v| v.as_array()) {
                        for p in arr {
                            if let Some(addr) = p.get("broadcast_address").and_then(|v| v.as_str()) {
                                producers_map.insert(addr.to_string(), p.clone());
                            }
                        }
                    }
                }
            }
        }
        
        // Add directly configured nsqd nodes (if not already from lookupd)
        for addr in &self.config.nsqd_http_addresses {
            let base = Self::normalize_address(addr);
            
            // Try to get node info from nsqd /stats endpoint
            if let Ok(resp) = self.http_client.get(&format!("{}/stats?format=json", base)).send().await {
                if let Ok(stats) = resp.json::<serde_json::Value>().await {
                    // Extract host and port from address
                    let parts: Vec<&str> = base.trim_start_matches("http://").trim_start_matches("https://").split(':').collect();
                    let host = parts.first().unwrap_or(&"127.0.0.1");
                    let http_port = parts.get(1).and_then(|p| p.parse::<u64>().ok()).unwrap_or(4151);
                    
                    // Create producer info
                    let producer = json!({
                        "broadcast_address": host,
                        "hostname": stats.get("host").and_then(|v| v.as_str()).unwrap_or(host),
                        "http_port": http_port,
                        "tcp_port": http_port - 1, // Assume TCP port is HTTP port - 1
                        "version": stats.get("version").and_then(|v| v.as_str()).unwrap_or("1.3.0"),
                        "last_update": chrono::Utc::now().timestamp(),
                        "topics": stats.get("topics").and_then(|v| v.as_array()).map(|t| t.len()).unwrap_or(0),
                    });
                    
                    producers_map.insert(host.to_string(), producer);
                }
            }
        }
        
        Ok(producers_map.into_values().collect())
    }

    /// Aggregate topic statistics from all nsqd nodes
    async fn aggregate_topic_stats(&self) -> std::result::Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
        let nsqd_addresses = self.get_all_nsqd_addresses().await;
        let mut topics_map: HashMap<String, TopicInfo> = HashMap::new();
        
        for nsqd_addr in nsqd_addresses {
            let url = format!("{}/stats?format=json", nsqd_addr);
            if let Ok(resp) = self.http_client.get(&url).send().await {
                if let Ok(json) = resp.json::<serde_json::Value>().await {
                    if let Some(topics) = json.get("topics").and_then(|v| v.as_array()) {
                        for topic in topics {
                            if let Some(topic_name) = topic.get("topic_name").and_then(|v| v.as_str()) {
                                let entry = topics_map.entry(topic_name.to_string()).or_insert_with(|| TopicInfo {
                                    topic_name: topic_name.to_string(),
                                    channels: Vec::new(),
                                    depth: 0,
                                    backend_depth: 0,
                                    message_count: 0,
                                    paused: false,
                                    nodes: Vec::new(),
                                });
                                
                                entry.nodes.push(nsqd_addr.clone());
                                entry.depth += topic.get("depth").and_then(|v| v.as_u64()).unwrap_or(0);
                                entry.backend_depth += topic.get("backend_depth").and_then(|v| v.as_u64()).unwrap_or(0);
                                entry.message_count += topic.get("message_count").and_then(|v| v.as_u64()).unwrap_or(0);
                                entry.paused = topic.get("paused").and_then(|v| v.as_bool()).unwrap_or(false);
                                
                                // Aggregate channels
                                if let Some(channels) = topic.get("channels").and_then(|v| v.as_array()) {
                                    for channel in channels {
                                        if let Some(channel_name) = channel.get("channel_name").and_then(|v| v.as_str()) {
                                            if let Some(existing_channel) = entry.channels.iter_mut().find(|c| c.channel_name == channel_name) {
                                                existing_channel.depth += channel.get("depth").and_then(|v| v.as_u64()).unwrap_or(0);
                                                existing_channel.backend_depth += channel.get("backend_depth").and_then(|v| v.as_u64()).unwrap_or(0);
                                                existing_channel.message_count += channel.get("message_count").and_then(|v| v.as_u64()).unwrap_or(0);
                                                existing_channel.in_flight_count += channel.get("in_flight_count").and_then(|v| v.as_u64()).unwrap_or(0);
                                                existing_channel.deferred_count += channel.get("deferred_count").and_then(|v| v.as_u64()).unwrap_or(0);
                                                existing_channel.requeue_count += channel.get("requeue_count").and_then(|v| v.as_u64()).unwrap_or(0);
                                                existing_channel.timeout_count += channel.get("timeout_count").and_then(|v| v.as_u64()).unwrap_or(0);
                                            } else {
                                                entry.channels.push(ChannelInfo {
                                                    channel_name: channel_name.to_string(),
                                                    depth: channel.get("depth").and_then(|v| v.as_u64()).unwrap_or(0),
                                                    backend_depth: channel.get("backend_depth").and_then(|v| v.as_u64()).unwrap_or(0),
                                                    message_count: channel.get("message_count").and_then(|v| v.as_u64()).unwrap_or(0),
                                                    in_flight_count: channel.get("in_flight_count").and_then(|v| v.as_u64()).unwrap_or(0),
                                                    deferred_count: channel.get("deferred_count").and_then(|v| v.as_u64()).unwrap_or(0),
                                                    requeue_count: channel.get("requeue_count").and_then(|v| v.as_u64()).unwrap_or(0),
                                                    timeout_count: channel.get("timeout_count").and_then(|v| v.as_u64()).unwrap_or(0),
                                                    paused: channel.get("paused").and_then(|v| v.as_bool()).unwrap_or(false),
                                                    clients: Vec::new(),
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        let topics: Vec<serde_json::Value> = topics_map.into_values()
            .map(|t| json!({
                "topic_name": t.topic_name,
                "channels": t.channels.into_iter().map(|c| json!({
                    "channel_name": c.channel_name,
                    "depth": c.depth,
                    "backend_depth": c.backend_depth,
                    "message_count": c.message_count,
                    "in_flight_count": c.in_flight_count,
                    "deferred_count": c.deferred_count,
                    "requeue_count": c.requeue_count,
                    "timeout_count": c.timeout_count,
                    "paused": c.paused,
                    "clients": c.clients,
                })).collect::<Vec<_>>(),
                "depth": t.depth,
                "backend_depth": t.backend_depth,
                "message_count": t.message_count,
                "paused": t.paused,
                "nodes": t.nodes,
            }))
            .collect();
        
        Ok(topics)
    }

    /// Get detailed information about a specific topic
    async fn get_topic_detail(&self, topic_name: &str) -> std::result::Result<serde_json::Value, Box<dyn std::error::Error>> {
        let topics = self.aggregate_topic_stats().await?;
        
        for topic in topics {
            if topic.get("topic_name").and_then(|v| v.as_str()) == Some(topic_name) {
                return Ok(topic);
            }
        }
        
        Ok(json!({
            "topic_name": topic_name,
            "channels": [],
            "depth": 0,
            "backend_depth": 0,
            "message_count": 0,
            "paused": false,
            "nodes": [],
        }))
    }
    
    /// Send command to all nsqd nodes for a topic
    async fn send_to_all_nsqd(&self, endpoint: &str, topic: &str, channel: Option<&str>) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let nsqd_addresses = self.get_all_nsqd_addresses().await;
        
        for addr in nsqd_addresses {
            let mut url = format!("{}/{}?topic={}", addr, endpoint, topic);
            if let Some(ch) = channel {
                url = format!("{}&channel={}", url, ch);
            }
            
            match self.http_client.post(&url).send().await {
                Ok(resp) => {
                    if !resp.status().is_success() {
                        tracing::warn!("Failed to {} topic {} on {}: status {}", endpoint, topic, addr, resp.status());
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to {} topic {} on {}: {}", endpoint, topic, addr, e);
                }
            }
        }
        
        Ok(())
    }
    
    /// Handle topic create
    async fn handle_topic_create(
        State(server): State<Arc<NsqadminServer>>,
        AxumPath(topic): AxumPath<String>
    ) -> Json<serde_json::Value> {
        tracing::info!("Creating topic: {}", topic);
        
        match server.send_to_all_nsqd("topic/create", &topic, None).await {
            Ok(_) => Json(json!({"status": "ok", "message": format!("Topic {} created", topic)})),
            Err(e) => Json(json!({"status": "error", "message": format!("Failed to create topic {}: {}", topic, e)})),
        }
    }
    
    /// Handle topic pause
    async fn handle_topic_pause(
        State(server): State<Arc<NsqadminServer>>,
        AxumPath(topic): AxumPath<String>
    ) -> Json<serde_json::Value> {
        tracing::info!("Pausing topic: {}", topic);
        
        match server.send_to_all_nsqd("topic/pause", &topic, None).await {
            Ok(_) => Json(json!({"status": "ok", "message": format!("Topic {} paused", topic)})),
            Err(e) => Json(json!({"status": "error", "message": format!("Failed to pause topic {}: {}", topic, e)})),
        }
    }
    
    /// Handle topic unpause
    async fn handle_topic_unpause(
        State(server): State<Arc<NsqadminServer>>,
        AxumPath(topic): AxumPath<String>
    ) -> Json<serde_json::Value> {
        tracing::info!("Unpausing topic: {}", topic);
        
        match server.send_to_all_nsqd("topic/unpause", &topic, None).await {
            Ok(_) => Json(json!({"status": "ok", "message": format!("Topic {} unpaused", topic)})),
            Err(e) => Json(json!({"status": "error", "message": format!("Failed to unpause topic {}: {}", topic, e)})),
        }
    }
    
    /// Handle topic delete
    async fn handle_topic_delete(
        State(server): State<Arc<NsqadminServer>>,
        AxumPath(topic): AxumPath<String>
    ) -> Json<serde_json::Value> {
        tracing::info!("Deleting topic: {}", topic);
        
        match server.send_to_all_nsqd("topic/delete", &topic, None).await {
            Ok(_) => Json(json!({"status": "ok", "message": format!("Topic {} deleted", topic)})),
            Err(e) => Json(json!({"status": "error", "message": format!("Failed to delete topic {}: {}", topic, e)})),
        }
    }
    
    /// Handle channel create
    async fn handle_channel_create(
        State(server): State<Arc<NsqadminServer>>,
        AxumPath((topic, channel)): AxumPath<(String, String)>
    ) -> Json<serde_json::Value> {
        tracing::info!("Creating channel: {} on topic: {}", channel, topic);
        
        match server.send_to_all_nsqd("channel/create", &topic, Some(&channel)).await {
            Ok(_) => Json(json!({"status": "ok", "message": format!("Channel {} on topic {} created", channel, topic)})),
            Err(e) => Json(json!({"status": "error", "message": format!("Failed to create channel {} on topic {}: {}", channel, topic, e)})),
        }
    }
    
    /// Handle channel pause
    async fn handle_channel_pause(
        State(server): State<Arc<NsqadminServer>>,
        AxumPath((topic, channel)): AxumPath<(String, String)>
    ) -> Json<serde_json::Value> {
        tracing::info!("Pausing channel: {} on topic: {}", channel, topic);
        
        match server.send_to_all_nsqd("channel/pause", &topic, Some(&channel)).await {
            Ok(_) => Json(json!({"status": "ok", "message": format!("Channel {} on topic {} paused", channel, topic)})),
            Err(e) => Json(json!({"status": "error", "message": format!("Failed to pause channel {} on topic {}: {}", channel, topic, e)})),
        }
    }
    
    /// Handle channel unpause
    async fn handle_channel_unpause(
        State(server): State<Arc<NsqadminServer>>,
        AxumPath((topic, channel)): AxumPath<(String, String)>
    ) -> Json<serde_json::Value> {
        tracing::info!("Unpausing channel: {} on topic: {}", channel, topic);
        
        match server.send_to_all_nsqd("channel/unpause", &topic, Some(&channel)).await {
            Ok(_) => Json(json!({"status": "ok", "message": format!("Channel {} on topic {} unpaused", channel, topic)})),
            Err(e) => Json(json!({"status": "error", "message": format!("Failed to unpause channel {} on topic {}: {}", channel, topic, e)})),
        }
    }
    
    /// Handle channel delete
    async fn handle_channel_delete(
        State(server): State<Arc<NsqadminServer>>,
        AxumPath((topic, channel)): AxumPath<(String, String)>
    ) -> Json<serde_json::Value> {
        tracing::info!("Deleting channel: {} on topic: {}", channel, topic);
        
        match server.send_to_all_nsqd("channel/delete", &topic, Some(&channel)).await {
            Ok(_) => Json(json!({"status": "ok", "message": format!("Channel {} on topic {} deleted", channel, topic)})),
            Err(e) => Json(json!({"status": "error", "message": format!("Failed to delete channel {} on topic {}: {}", channel, topic, e)})),
        }
    }
    
    /// Handle channel empty
    async fn handle_channel_empty(
        State(server): State<Arc<NsqadminServer>>,
        AxumPath((topic, channel)): AxumPath<(String, String)>
    ) -> Json<serde_json::Value> {
        tracing::info!("Emptying channel: {} on topic: {}", channel, topic);
        
        match server.send_to_all_nsqd("channel/empty", &topic, Some(&channel)).await {
            Ok(_) => Json(json!({"status": "ok", "message": format!("Channel {} on topic {} emptied", channel, topic)})),
            Err(e) => Json(json!({"status": "error", "message": format!("Failed to empty channel {} on topic {}: {}", channel, topic, e)})),
        }
    }
}

impl Clone for NsqadminServer {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            metrics: self.metrics.clone(),
            http_client: self.http_client.clone(),
            start_time: self.start_time,
            start_instant: self.start_instant,
        }
    }
}