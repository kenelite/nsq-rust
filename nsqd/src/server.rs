//! NSQd server implementation

use std::sync::Arc;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;
use uuid::Uuid;
use parking_lot::RwLock;
use tokio::net::{TcpListener, TcpStream};
use tokio::time::interval;
use tokio_util::codec::Framed;
use axum::{
    extract::{Path, State},
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::Value;
use nsq_protocol::{NsqDecoder};
use nsq_common::{Metrics, Result, NsqError, init_logging};
use crate::config::NsqdConfig;
use crate::topic::Topic;
use crate::client::{Client, ClientInfo};
use crate::stats::{StatsCollector, NsqdStats};

/// NSQd server
pub struct NsqdServer {
    /// Server configuration
    config: NsqdConfig,
    /// Metrics collector
    metrics: Metrics,
    /// Statistics collector
    stats: Arc<StatsCollector>,
    /// Topics
    topics: Arc<RwLock<HashMap<String, Arc<Topic>>>>,
    /// Clients
    clients: Arc<RwLock<HashMap<Uuid, Arc<Client>>>>,
    /// TCP listener
    tcp_listener: Option<TcpListener>,
    /// HTTP listener
    http_listener: Option<TcpListener>,
    /// HTTPS listener
    https_listener: Option<TcpListener>,
}

impl NsqdServer {
    /// Create a new NSQd server
    pub fn new(config: NsqdConfig) -> Result<Self> {
        // Initialize logging
        init_logging(&config.base)?;
        
        // Initialize metrics
        let metrics = Metrics::new(&config.base)?;
        
        // Initialize statistics collector
        let stats = Arc::new(StatsCollector::new(metrics.clone()));
        
        Ok(Self {
            config,
            metrics,
            stats,
            topics: Arc::new(RwLock::new(HashMap::new())),
            clients: Arc::new(RwLock::new(HashMap::new())),
            tcp_listener: None,
            http_listener: None,
            https_listener: None,
        })
    }
    
    /// Start the server
    pub async fn start(&mut self) -> Result<()> {
        tracing::info!("Starting NSQd server");
        
        // Start TCP server
        if let Some(tcp_addr) = self.parse_address(&self.config.tcp_address)? {
            let listener = TcpListener::bind(tcp_addr).await
                .map_err(|e| NsqError::Io(e))?;
            self.tcp_listener = Some(listener);
            tracing::info!("TCP server listening on {}", tcp_addr);
        }
        
        // Start HTTP server
        if !self.config.disable_http {
            if let Some(http_addr) = self.parse_address(&self.config.http_address)? {
                let listener = TcpListener::bind(http_addr).await
                    .map_err(|e| NsqError::Io(e))?;
                self.http_listener = Some(listener);
                tracing::info!("HTTP server listening on {}", http_addr);
            }
        }
        
        // Start HTTPS server
        if !self.config.disable_https {
            if let Some(https_addr) = self.parse_address(&self.config.https_address.as_ref().unwrap_or(&"".to_string()))? {
                let listener = TcpListener::bind(https_addr).await
                    .map_err(|e| NsqError::Io(e))?;
                self.https_listener = Some(listener);
                tracing::info!("HTTPS server listening on {}", https_addr);
            }
        }
        
        // Start background tasks
        self.start_background_tasks().await;
        
        // Start TCP server
        if let Some(listener) = self.tcp_listener.take() {
            let server = self.clone();
            tokio::spawn(async move {
                if let Err(e) = server.handle_tcp_connections(listener).await {
                    tracing::error!("TCP server error: {}", e);
                }
            });
        }
        
        // Start HTTP server
        if let Some(listener) = self.http_listener.take() {
            let server = self.clone();
            tokio::spawn(async move {
                if let Err(e) = server.handle_http_connections(listener).await {
                    tracing::error!("HTTP server error: {}", e);
                }
            });
        }
        
        // Start HTTPS server
        if let Some(listener) = self.https_listener.take() {
            let server = self.clone();
            tokio::spawn(async move {
                if let Err(e) = server.handle_https_connections(listener).await {
                    tracing::error!("HTTPS server error: {}", e);
                }
            });
        }
        
        tracing::info!("NSQd server started successfully");
        Ok(())
    }
    
    /// Parse address string
    fn parse_address(&self, addr: &str) -> Result<Option<SocketAddr>> {
        if addr.is_empty() {
            return Ok(None);
        }
        
        if addr.starts_with('/') {
            // Unix socket path
            return Ok(None);
        }
        
        let socket_addr = addr.parse::<SocketAddr>()
            .map_err(|e| NsqError::Validation(format!("Invalid address: {}", e)))?;
        
        Ok(Some(socket_addr))
    }
    
    /// Start background tasks
    async fn start_background_tasks(&self) {
        // Message processing task
        let topics = self.topics.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(100));
            loop {
                interval.tick().await;
                
                let topics = topics.read();
                for topic in topics.values() {
                    if let Err(e) = topic.process_deferred() {
                        tracing::warn!("Failed to process deferred messages for topic {}: {}", topic.name, e);
                    }
                    
                    if let Err(e) = topic.cleanup_timeouts() {
                        tracing::warn!("Failed to cleanup timeouts for topic {}: {}", topic.name, e);
                    }
                }
            }
        });
        
        // Client cleanup task
        let clients = self.clients.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30));
            loop {
                interval.tick().await;
                
                let mut clients = clients.write();
                let timed_out_clients: Vec<Uuid> = clients
                    .iter()
                    .filter(|(_, client)| client.is_timed_out())
                    .map(|(id, _)| *id)
                    .collect();
                
                for client_id in timed_out_clients {
                    if let Some(_client) = clients.remove(&client_id) {
                        tracing::info!("Client {} timed out", client_id);
                    }
                }
            }
        });
    }
    
    /// Handle TCP connections
    async fn handle_tcp_connections(&self, listener: TcpListener) -> Result<()> {
        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    let server = self.clone();
                    tokio::spawn(async move {
                        if let Err(e) = server.handle_tcp_connection(stream, addr).await {
                            tracing::error!("TCP connection error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    tracing::error!("Failed to accept TCP connection: {}", e);
                }
            }
        }
    }
    
    /// Handle HTTP connections
    async fn handle_http_connections(&self, listener: TcpListener) -> Result<()> {
        let app = self.create_http_router();
        
        axum::serve(listener, app).await
            .map_err(|e| NsqError::Io(e))?;
        
        Ok(())
    }
    
    /// Handle HTTPS connections
    async fn handle_https_connections(&self, _listener: TcpListener) -> Result<()> {
        // TODO: Implement HTTPS with TLS
        tracing::warn!("HTTPS server not implemented yet");
        Ok(())
    }
    
    /// Handle individual TCP connection
    async fn handle_tcp_connection(&self, stream: TcpStream, addr: SocketAddr) -> Result<()> {
        let framed = Framed::new(stream, NsqDecoder::new());
        let client_info = ClientInfo {
            remote_addr: addr.to_string(),
            ..Default::default()
        };
        
        let client = Arc::new(Client::new(client_info, framed, self.metrics.clone()));
        let client_id = client.info.id;
        
        self.stats.add_client(client_id, client.clone());
        self.clients.write().insert(client_id, client.clone());
        
        tracing::info!("New TCP connection from {}", addr);
        
        // Handle client protocol
        self.handle_client_protocol(client).await?;
        
        // Cleanup
        self.clients.write().remove(&client_id);
        self.stats.remove_client(&client_id);
        
        tracing::info!("TCP connection from {} closed", addr);
        Ok(())
    }
    
    /// Handle client protocol
    async fn handle_client_protocol(&self, _client: Arc<Client>) -> Result<()> {
        // TODO: Implement client protocol handling
        // This would include:
        // - IDENTIFY command
        // - SUB command
        // - RDY command
        // - Message delivery
        // - FIN/REQ/TOUCH commands
        // - Heartbeat handling
        
        tracing::info!("Client protocol handling not implemented yet");
        Ok(())
    }
    
    /// Create HTTP router
    fn create_http_router(&self) -> Router {
        let server = self.clone();
        
        Router::new()
            .route("/ping", get(|| async { "OK" }))
            .route("/info", get(|| async { Json(serde_json::json!({"version": "1.3.0"})) }))
            .route("/stats", get(|| async { Json(serde_json::json!({"topics": []})) }))
            .route("/pub/:topic", post(|| async { "OK" }))
            .route("/mpub/:topic", post(|| async { "OK" }))
            .route("/dpub/:topic", post(|| async { "OK" }))
            .route("/topic/create/:topic", post(|| async { "OK" }))
            .route("/topic/delete/:topic", post(|| async { "OK" }))
            .route("/topic/pause/:topic", post(|| async { "OK" }))
            .route("/topic/unpause/:topic", post(|| async { "OK" }))
            .route("/channel/create/:topic/:channel", post(|| async { "OK" }))
            .route("/channel/delete/:topic/:channel", post(|| async { "OK" }))
            .route("/channel/pause/:topic/:channel", post(|| async { "OK" }))
            .route("/channel/unpause/:topic/:channel", post(|| async { "OK" }))
            .route("/config/:key", get(|| async { Json(serde_json::json!({"value": ""})) }))
            .route("/config/:key", post(|| async { "OK" }))
            .route("/debug/freememory", get(|| async { Json(serde_json::json!({"memory": 0})) }))
            .with_state(server)
    }
}

impl Clone for NsqdServer {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            metrics: self.metrics.clone(),
            stats: self.stats.clone(),
            topics: self.topics.clone(),
            clients: self.clients.clone(),
            tcp_listener: None,
            http_listener: None,
            https_listener: None,
        }
    }
}
