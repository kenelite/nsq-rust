//! Client connection management

use std::sync::Arc;
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;
use parking_lot::RwLock;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;
use nsq_protocol::{Command, Message, NsqDecoder};
use nsq_common::{Metrics, Result, NsqError};

/// Client connection state
#[derive(Debug, Clone, PartialEq)]
pub enum ClientState {
    /// Initial state
    Initial,
    /// Identified state
    Identified,
    /// Subscribed to a topic/channel
    Subscribed,
    /// Ready to receive messages
    Ready,
    /// Closed
    Closed,
}

/// Client information
#[derive(Debug, Clone)]
pub struct ClientInfo {
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
    pub heartbeat_interval: Duration,
    pub output_buffer_size: usize,
    pub output_buffer_timeout: Duration,
    pub max_rdy_count: u32,
    pub max_msg_timeout: Duration,
    pub msg_timeout: Duration,
}

impl Default for ClientInfo {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            remote_addr: "unknown".to_string(),
            user_agent: None,
            client_version: None,
            hostname: None,
            tls_version: None,
            tls_cipher_suite: None,
            deflate: false,
            snappy: false,
            sample_rate: 0,
            heartbeat_interval: Duration::from_secs(30),
            output_buffer_size: 16 * 1024, // 16KB
            output_buffer_timeout: Duration::from_millis(250),
            max_rdy_count: 2500,
            max_msg_timeout: Duration::from_secs(15 * 60), // 15 minutes
            msg_timeout: Duration::from_secs(60), // 1 minute
        }
    }
}

/// Client connection
pub struct Client {
    /// Client information
    pub info: ClientInfo,
    /// Current state
    state: Arc<RwLock<ClientState>>,
    /// Subscribed topic
    topic: Arc<RwLock<Option<String>>>,
    /// Subscribed channel
    channel: Arc<RwLock<Option<String>>>,
    /// Current RDY count
    rdy_count: Arc<RwLock<u32>>,
    /// Last message time
    last_message_time: Arc<RwLock<Option<std::time::Instant>>>,
    /// In-flight messages
    in_flight_messages: Arc<RwLock<HashMap<Uuid, Message>>>,
    /// TCP stream
    stream: Option<Framed<TcpStream, NsqDecoder>>,
    /// Metrics
    metrics: Metrics,
    /// Client statistics
    stats: Arc<RwLock<ClientStats>>,
}

/// Client statistics
#[derive(Debug, Clone, Default)]
pub struct ClientStats {
    pub messages_received: u64,
    pub messages_finished: u64,
    pub messages_requeued: u64,
    pub messages_timed_out: u64,
    pub bytes_received: u64,
    pub bytes_sent: u64,
    pub commands_received: u64,
    pub commands_sent: u64,
}

impl Client {
    /// Create a new client
    pub fn new(
        info: ClientInfo,
        stream: Framed<TcpStream, NsqDecoder>,
        metrics: Metrics,
    ) -> Self {
        Self {
            info,
            state: Arc::new(RwLock::new(ClientState::Initial)),
            topic: Arc::new(RwLock::new(None)),
            channel: Arc::new(RwLock::new(None)),
            rdy_count: Arc::new(RwLock::new(0)),
            last_message_time: Arc::new(RwLock::new(None)),
            in_flight_messages: Arc::new(RwLock::new(HashMap::new())),
            stream: Some(stream),
            metrics,
            stats: Arc::new(RwLock::new(ClientStats::default())),
        }
    }
    
    /// Get current state
    pub fn state(&self) -> ClientState {
        self.state.read().clone()
    }
    
    /// Set client state
    pub fn set_state(&self, state: ClientState) {
        *self.state.write() = state;
    }
    
    /// Get subscribed topic
    pub fn topic(&self) -> Option<String> {
        self.topic.read().clone()
    }
    
    /// Set subscribed topic
    pub fn set_topic(&self, topic: String) {
        *self.topic.write() = Some(topic);
    }
    
    /// Get subscribed channel
    pub fn channel(&self) -> Option<String> {
        self.channel.read().clone()
    }
    
    /// Set subscribed channel
    pub fn set_channel(&self, channel: String) {
        *self.channel.write() = Some(channel);
    }
    
    /// Get current RDY count
    pub fn rdy_count(&self) -> u32 {
        *self.rdy_count.read()
    }
    
    /// Set RDY count
    pub fn set_rdy_count(&self, count: u32) {
        *self.rdy_count.write() = count;
    }
    
    /// Check if client is ready to receive messages
    pub fn is_ready(&self) -> bool {
        self.state() == ClientState::Ready && self.rdy_count() > 0
    }
    
    /// Add in-flight message
    pub fn add_in_flight(&self, message: Message) {
        let message_id = message.id;
        let message_size = message.size();
        
        self.in_flight_messages.write().insert(message_id, message);
        
        {
            let mut stats = self.stats.write();
            stats.messages_received += 1;
            stats.bytes_received += message_size as u64;
        }
        
        *self.last_message_time.write() = Some(std::time::Instant::now());
        self.metrics.incr("client.messages.in_flight", 1);
    }
    
    /// Remove in-flight message
    pub fn remove_in_flight(&self, message_id: Uuid) -> Option<Message> {
        let message = self.in_flight_messages.write().remove(&message_id);
        
        if message.is_some() {
            {
                let mut stats = self.stats.write();
                stats.messages_finished += 1;
            }
            
            self.metrics.incr("client.messages.finished", 1);
        }
        
        message
    }
    
    /// Get in-flight message count
    pub fn in_flight_count(&self) -> usize {
        self.in_flight_messages.read().len()
    }
    
    /// Get client statistics
    pub fn stats(&self) -> ClientStats {
        self.stats.read().clone()
    }
    
    /// Check if client has timed out
    pub fn is_timed_out(&self) -> bool {
        if let Some(last_time) = *self.last_message_time.read() {
            last_time.elapsed() > self.info.msg_timeout
        } else {
            false
        }
    }
    
    /// Send a command to the client
    pub async fn send_command(&mut self, _command: Command) -> Result<()> {
        if let Some(_stream) = self.stream.as_mut() {
            // TODO: Implement sending command via stream
            // let frame = Frame::new(nsq_protocol::FrameType::Response, command.to_bytes()?);
            // stream.send(frame).await.map_err(|e| NsqError::Io(e))?;
            
            {
                let mut stats = self.stats.write();
                stats.commands_sent += 1;
            }
            
            self.metrics.incr("client.commands.sent", 1);
        } else {
            return Err(NsqError::Validation("Client stream not available".to_string()));
        }
        
        Ok(())
    }
    
    /// Send a message to the client
    pub async fn send_message(&mut self, _message: Message) -> Result<()> {
        if let Some(_stream) = self.stream.as_mut() {
            // TODO: Implement sending message via stream
            // let frame = Frame::new(nsq_protocol::FrameType::Message, message.to_bytes());
            // stream.send(frame).await.map_err(|e| NsqError::Io(e))?;
            
            {
                let mut stats = self.stats.write();
                stats.bytes_sent += _message.size() as u64;
            }
            
            self.metrics.incr("client.messages.sent", 1);
        } else {
            return Err(NsqError::Validation("Client stream not available".to_string()));
        }
        
        Ok(())
    }
    
    /// Send an error to the client
    pub async fn send_error(&mut self, _error: String) -> Result<()> {
        if let Some(_stream) = self.stream.as_mut() {
            // TODO: Implement sending error via stream
            // let frame = Frame::new(nsq_protocol::FrameType::Error, error.into());
            // stream.send(frame).await.map_err(|e| NsqError::Io(e))?;
            
            self.metrics.incr("client.errors.sent", 1);
        } else {
            return Err(NsqError::Validation("Client stream not available".to_string()));
        }
        
        Ok(())
    }
    
    /// Close the client connection
    pub fn close(&mut self) {
        self.set_state(ClientState::Closed);
        self.stream = None;
        self.metrics.incr("client.connections.closed", 1);
    }
    
    /// Check if client is closed
    pub fn is_closed(&self) -> bool {
        self.state() == ClientState::Closed
    }
}
