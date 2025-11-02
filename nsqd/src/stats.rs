//! Statistics and monitoring

use std::sync::Arc;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use parking_lot::RwLock;
use nsq_common::Metrics;
use crate::topic::Topic;
use crate::client::Client;

/// NSQd statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NsqdStats {
    /// Server information
    pub server: ServerInfo,
    /// Topic statistics
    pub topics: Vec<TopicStats>,
    /// Client statistics
    pub clients: Vec<ClientStats>,
    /// Overall statistics
    pub overall: OverallStats,
}

/// Server information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub version: String,
    pub build_version: String,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub uptime: u64,
    pub tcp_port: u16,
    pub http_port: u16,
    pub https_port: Option<u16>,
    pub tcp_socket_path: Option<String>,
    pub http_socket_path: Option<String>,
    pub https_socket_path: Option<String>,
    pub data_path: String,
    pub mem_queue_size: usize,
    pub max_msg_size: usize,
    pub max_body_size: usize,
    pub max_req_timeout: u64,
    pub max_msg_timeout: u64,
    pub msg_timeout: u64,
    pub max_output_buffer_size: usize,
    pub max_output_buffer_timeout: u64,
    pub tls_cert: Option<String>,
    pub tls_key: Option<String>,
    pub tls_root_ca_file: Option<String>,
    pub tls_min_version: String,
    pub statsd_address: Option<String>,
    pub statsd_prefix: String,
    pub e2e_processing_latency_percentile: Vec<f64>,
    pub lookupd_tcp_addresses: Vec<String>,
    pub disable_http: bool,
    pub disable_https: bool,
}

/// Topic statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicStats {
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub paused: bool,
    pub message_count: u64,
    pub channel_count: u64,
    pub depth: u64,
    pub backend_depth: u64,
    pub in_flight_count: u64,
    pub deferred_count: u64,
    pub requeue_count: u64,
    pub timeout_count: u64,
    pub channels: Vec<ChannelStats>,
}

/// Channel statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelStats {
    pub name: String,
    pub topic_name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub paused: bool,
    pub message_count: u64,
    pub depth: u64,
    pub backend_depth: u64,
    pub in_flight_count: u64,
    pub deferred_count: u64,
    pub requeue_count: u64,
    pub timeout_count: u64,
    pub client_count: u64,
}

/// Client statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientStats {
    pub id: Uuid,
    pub remote_addr: String,
    pub user_agent: Option<String>,
    pub client_version: Option<String>,
    pub hostname: Option<String>,
    pub tls_version: Option<String>,
    pub tls_cipher_suite: Option<String>,
    pub deflate: bool,
    pub snappy: bool,
    pub sample_rate: u32,
    pub heartbeat_interval: u64,
    pub output_buffer_size: usize,
    pub output_buffer_timeout: u64,
    pub max_rdy_count: u32,
    pub max_msg_timeout: u64,
    pub msg_timeout: u64,
    pub state: String,
    pub topic: Option<String>,
    pub channel: Option<String>,
    pub rdy_count: u32,
    pub in_flight_count: u64,
    pub messages_received: u64,
    pub messages_finished: u64,
    pub messages_requeued: u64,
    pub messages_timed_out: u64,
    pub bytes_received: u64,
    pub bytes_sent: u64,
    pub commands_received: u64,
    pub commands_sent: u64,
}

/// Overall statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverallStats {
    pub topic_count: u64,
    pub channel_count: u64,
    pub client_count: u64,
    pub message_count: u64,
    pub total_depth: u64,
    pub total_backend_depth: u64,
    pub total_in_flight_count: u64,
    pub total_deferred_count: u64,
    pub total_requeue_count: u64,
    pub total_timeout_count: u64,
    pub total_bytes_received: u64,
    pub total_bytes_sent: u64,
    pub total_commands_received: u64,
    pub total_commands_sent: u64,
}

/// Statistics collector
pub struct StatsCollector {
    /// Server information
    server_info: Arc<RwLock<ServerInfo>>,
    /// Topics
    topics: Arc<RwLock<HashMap<String, Arc<Topic>>>>,
    /// Clients
    clients: Arc<RwLock<HashMap<Uuid, Arc<Client>>>>,
    /// Metrics
    #[allow(dead_code)]
    metrics: Metrics,
    /// Start time
    start_time: std::time::Instant,
}

impl StatsCollector {
    /// Create a new statistics collector
    pub fn new(metrics: Metrics) -> Self {
        Self {
            server_info: Arc::new(RwLock::new(ServerInfo {
                version: env!("CARGO_PKG_VERSION").to_string(),
                build_version: env!("CARGO_PKG_VERSION").to_string(),
                start_time: chrono::Utc::now(),
                uptime: 0,
                tcp_port: 4150,
                http_port: 4151,
                https_port: None,
                tcp_socket_path: None,
                http_socket_path: None,
                https_socket_path: None,
                data_path: "./data".to_string(),
                mem_queue_size: 10000,
                max_msg_size: 1024 * 1024,
                max_body_size: 5 * 1024 * 1024,
                max_req_timeout: 60 * 1000,
                max_msg_timeout: 15 * 60 * 1000,
                msg_timeout: 60 * 1000,
                max_output_buffer_size: 16 * 1024,
                max_output_buffer_timeout: 250,
                tls_cert: None,
                tls_key: None,
                tls_root_ca_file: None,
                tls_min_version: "1.2".to_string(),
                statsd_address: None,
                statsd_prefix: "nsq".to_string(),
                e2e_processing_latency_percentile: vec![0.5, 0.75, 0.9, 0.95, 0.99],
                lookupd_tcp_addresses: Vec::new(),
                disable_http: false,
                disable_https: false,
            })),
            topics: Arc::new(RwLock::new(HashMap::new())),
            clients: Arc::new(RwLock::new(HashMap::new())),
            metrics,
            start_time: std::time::Instant::now(),
        }
    }
    
    /// Update server information
    pub fn update_server_info(&self, info: ServerInfo) {
        *self.server_info.write() = info;
    }
    
    /// Add a topic
    pub fn add_topic(&self, name: String, topic: Arc<Topic>) {
        self.topics.write().insert(name, topic);
    }
    
    /// Remove a topic
    pub fn remove_topic(&self, name: &str) {
        self.topics.write().remove(name);
    }
    
    /// Add a client
    pub fn add_client(&self, id: Uuid, client: Arc<Client>) {
        self.clients.write().insert(id, client);
    }
    
    /// Remove a client
    pub fn remove_client(&self, id: &Uuid) {
        self.clients.write().remove(id);
    }
    
    /// Get statistics
    pub fn get_stats(&self) -> NsqdStats {
        let server_info = {
            let mut info = self.server_info.read().clone();
            info.uptime = self.start_time.elapsed().as_secs();
            info
        };
        
        let topics = self.get_topic_stats();
        let clients = self.get_client_stats();
        let overall = self.get_overall_stats(&topics, &clients);
        
        NsqdStats {
            server: server_info,
            topics,
            clients,
            overall,
        }
    }
    
    /// Get topic statistics
    fn get_topic_stats(&self) -> Vec<TopicStats> {
        let topics = self.topics.read();
        let mut topic_stats = Vec::new();
        
        for (name, topic) in topics.iter() {
            let topic_stat = topic.stats();
            let channels = topic.get_channels();
            let mut channel_stats = Vec::new();
            
            for channel in channels {
                let channel_stat = channel.stats();
                channel_stats.push(ChannelStats {
                    name: channel.name.clone(),
                    topic_name: channel.topic_name.clone(),
                    created_at: channel.created_at,
                    paused: channel.is_paused(),
                    message_count: channel_stat.message_count,
                    depth: channel_stat.depth,
                    backend_depth: channel_stat.backend_depth,
                    in_flight_count: channel_stat.in_flight_count,
                    deferred_count: channel_stat.deferred_count,
                    requeue_count: channel_stat.requeue_count,
                    timeout_count: channel_stat.timeout_count,
                    client_count: channel_stat.client_count,
                });
            }
            
            topic_stats.push(TopicStats {
                name: name.clone(),
                created_at: topic.created_at,
                paused: topic.is_paused(),
                message_count: topic_stat.message_count,
                channel_count: topic_stat.channel_count,
                depth: topic_stat.depth,
                backend_depth: topic_stat.backend_depth,
                in_flight_count: topic_stat.in_flight_count,
                deferred_count: topic_stat.deferred_count,
                requeue_count: topic_stat.requeue_count,
                timeout_count: topic_stat.timeout_count,
                channels: channel_stats,
            });
        }
        
        topic_stats
    }
    
    /// Get client statistics
    fn get_client_stats(&self) -> Vec<ClientStats> {
        let clients = self.clients.read();
        let mut client_stats = Vec::new();
        
        for (id, client) in clients.iter() {
            let stats = client.stats();
            client_stats.push(ClientStats {
                id: *id,
                remote_addr: client.info.remote_addr.clone(),
                user_agent: client.info.user_agent.clone(),
                client_version: client.info.client_version.clone(),
                hostname: client.info.hostname.clone(),
                tls_version: client.info.tls_version.clone(),
                tls_cipher_suite: client.info.tls_cipher_suite.clone(),
                deflate: client.info.deflate,
                snappy: client.info.snappy,
                sample_rate: client.info.sample_rate,
                heartbeat_interval: client.info.heartbeat_interval.as_millis() as u64,
                output_buffer_size: client.info.output_buffer_size,
                output_buffer_timeout: client.info.output_buffer_timeout.as_millis() as u64,
                max_rdy_count: client.info.max_rdy_count,
                max_msg_timeout: client.info.max_msg_timeout.as_millis() as u64,
                msg_timeout: client.info.msg_timeout.as_millis() as u64,
                state: format!("{:?}", client.state()),
                topic: client.topic(),
                channel: client.channel(),
                rdy_count: client.rdy_count(),
                in_flight_count: client.in_flight_count() as u64,
                messages_received: stats.messages_received,
                messages_finished: stats.messages_finished,
                messages_requeued: stats.messages_requeued,
                messages_timed_out: stats.messages_timed_out,
                bytes_received: stats.bytes_received,
                bytes_sent: stats.bytes_sent,
                commands_received: stats.commands_received,
                commands_sent: stats.commands_sent,
            });
        }
        
        client_stats
    }
    
    /// Get metrics
    pub fn metrics(&self) -> &Metrics {
        &self.metrics
    }
    
    /// Get overall statistics
    fn get_overall_stats(&self, topics: &[TopicStats], clients: &[ClientStats]) -> OverallStats {
        let mut overall = OverallStats {
            topic_count: topics.len() as u64,
            channel_count: 0,
            client_count: clients.len() as u64,
            message_count: 0,
            total_depth: 0,
            total_backend_depth: 0,
            total_in_flight_count: 0,
            total_deferred_count: 0,
            total_requeue_count: 0,
            total_timeout_count: 0,
            total_bytes_received: 0,
            total_bytes_sent: 0,
            total_commands_received: 0,
            total_commands_sent: 0,
        };
        
        for topic in topics {
            overall.channel_count += topic.channel_count;
            overall.message_count += topic.message_count;
            overall.total_depth += topic.depth;
            overall.total_backend_depth += topic.backend_depth;
            overall.total_in_flight_count += topic.in_flight_count;
            overall.total_deferred_count += topic.deferred_count;
            overall.total_requeue_count += topic.requeue_count;
            overall.total_timeout_count += topic.timeout_count;
        }
        
        for client in clients {
            overall.total_bytes_received += client.bytes_received;
            overall.total_bytes_sent += client.bytes_sent;
            overall.total_commands_received += client.commands_received;
            overall.total_commands_sent += client.commands_sent;
        }
        
        overall
    }
}
