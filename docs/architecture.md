# Architecture Guide

This document describes the architecture and design principles of NSQ Rust.

## Table of Contents

- [Overview](#overview)
- [System Architecture](#system-architecture)
- [Component Design](#component-design)
- [Data Flow](#data-flow)
- [Protocol Design](#protocol-design)
- [Storage Design](#storage-design)
- [Network Design](#network-design)
- [Concurrency Model](#concurrency-model)
- [Error Handling](#error-handling)
- [Performance Considerations](#performance-considerations)
- [Scalability](#scalability)
- [Security](#security)
- [Monitoring](#monitoring)

## Overview

NSQ Rust is a complete rewrite of NSQ 1.3 in Rust, maintaining API compatibility while leveraging Rust's safety guarantees and performance characteristics. The system is designed to be highly available, fault-tolerant, and performant.

### Key Design Principles

1. **API Compatibility**: Maintains full compatibility with NSQ 1.3 APIs
2. **Safety**: Leverages Rust's memory safety and type system
3. **Performance**: Optimized for high throughput and low latency
4. **Reliability**: Fault-tolerant design with graceful degradation
5. **Scalability**: Horizontal scaling capabilities
6. **Observability**: Comprehensive monitoring and logging

## System Architecture

### High-Level Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Publishers    │    │   Consumers     │    │   NSQAdmin      │
│                 │    │                 │    │                 │
│  ┌───────────┐  │    │  ┌───────────┐  │    │  ┌───────────┐  │
│  │   App 1   │  │    │  │   App 1   │  │    │  │   Web UI   │  │
│  └───────────┘  │    │  └───────────┘  │    │  └───────────┘  │
│  ┌───────────┐  │    │  ┌───────────┐  │    │  ┌───────────┐  │
│  │   App 2   │  │    │  │   App 2   │  │    │  │   API     │  │
│  └───────────┘  │    │  └───────────┘  │    │  └───────────┘  │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────────────────────────────────────────────────────┐
│                        NSQ Rust Cluster                        │
│                                                                 │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐        │
│  │   NSQD 1    │    │   NSQD 2    │    │   NSQD 3    │        │
│  │             │    │             │    │             │        │
│  │ ┌─────────┐ │    │ ┌─────────┐ │    │ ┌─────────┐ │        │
│  │ │ Topics  │ │    │ │ Topics  │ │    │ │ Topics  │ │        │
│  │ │ Channels│ │    │ │ Channels│ │    │ │ Channels│ │        │
│  │ │ Messages│ │    │ │ Messages│ │    │ │ Messages│ │        │
│  │ └─────────┘ │    │ └─────────┘ │    │ └─────────┘ │        │
│  └─────────────┘    └─────────────┘    └─────────────┘        │
│         │                   │                   │              │
│         └───────────────────┼───────────────────┘              │
│                             │                                  │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐        │
│  │NSQLookupd 1│    │NSQLookupd 2│    │NSQLookupd 3│        │
│  │             │    │             │    │             │        │
│  │ ┌─────────┐ │    │ ┌─────────┐ │    │ ┌─────────┐ │        │
│  │ │Registry │ │    │ │Registry │ │    │ │Registry │ │        │
│  │ │Nodes    │ │    │ │Nodes    │ │    │ │Nodes    │ │        │
│  │ │Topics   │ │    │ │Topics   │ │    │ │Topics   │ │        │
│  │ └─────────┘ │    │ └─────────┘ │    │ └─────────┘ │        │
│  └─────────────┘    └─────────────┘    └─────────────┘        │
└─────────────────────────────────────────────────────────────────┘
```

### Component Relationships

```
NSQLookupd ──────────────┐
    │                    │
    │                    │
    ▼                    ▼
NSQD ────────────────── NSQD
    │                    │
    │                    │
    ▼                    ▼
NSQAdmin ──────────── NSQAdmin
```

## Component Design

### NSQD Design

#### Core Components

```rust
pub struct NsqdServer {
    config: NsqdConfig,
    topics: Arc<RwLock<HashMap<String, Topic>>>,
    clients: Arc<RwLock<HashMap<String, Client>>>,
    metrics: Metrics,
    stats_collector: StatsCollector,
    tcp_server: TcpServer,
    http_server: HttpServer,
}
```

#### Topic Management

```rust
pub struct Topic {
    name: String,
    channels: Arc<RwLock<HashMap<String, Channel>>>,
    message_queue: Arc<MessageQueue>,
    metrics: Metrics,
    config: TopicConfig,
}

impl Topic {
    pub async fn publish(&self, message: Message) -> Result<(), NsqError> {
        // Validate message
        self.validate_message(&message)?;
        
        // Add to queue
        self.message_queue.push(message).await?;
        
        // Update metrics
        self.metrics.counter("messages.published", 1);
        
        Ok(())
    }
    
    pub async fn subscribe(&self, channel_name: String) -> Result<Channel, NsqError> {
        let mut channels = self.channels.write().await;
        
        if let Some(channel) = channels.get(&channel_name) {
            return Ok(channel.clone());
        }
        
        let channel = Channel::new(channel_name.clone(), self.clone())?;
        channels.insert(channel_name, channel.clone());
        
        Ok(channel)
    }
}
```

#### Channel Management

```rust
pub struct Channel {
    name: String,
    topic: Topic,
    clients: Arc<RwLock<HashMap<String, Client>>>,
    message_queue: Arc<MessageQueue>,
    metrics: Metrics,
    config: ChannelConfig,
}

impl Channel {
    pub async fn add_client(&self, client: Client) -> Result<(), NsqError> {
        let mut clients = self.clients.write().await;
        clients.insert(client.id().clone(), client);
        
        self.metrics.gauge("clients.connected", clients.len() as f64);
        Ok(())
    }
    
    pub async fn remove_client(&self, client_id: &str) -> Result<(), NsqError> {
        let mut clients = self.clients.write().await;
        clients.remove(client_id);
        
        self.metrics.gauge("clients.connected", clients.len() as f64);
        Ok(())
    }
    
    pub async fn deliver_message(&self, message: Message) -> Result<(), NsqError> {
        let clients = self.clients.read().await;
        
        // Round-robin delivery
        for client in clients.values() {
            if client.can_receive() {
                client.send_message(message.clone()).await?;
                break;
            }
        }
        
        Ok(())
    }
}
```

#### Message Queue Design

```rust
pub struct MessageQueue {
    memory_queue: Arc<Mutex<VecDeque<Message>>>,
    disk_queue: Arc<DiskQueue>,
    config: QueueConfig,
    metrics: Metrics,
}

impl MessageQueue {
    pub async fn push(&self, message: Message) -> Result<(), NsqError> {
        let mut memory_queue = self.memory_queue.lock().await;
        
        if memory_queue.len() < self.config.max_memory_size {
            memory_queue.push_back(message);
            self.metrics.gauge("queue.memory.depth", memory_queue.len() as f64);
        } else {
            // Spill to disk
            self.disk_queue.push(message).await?;
            self.metrics.gauge("queue.disk.depth", self.disk_queue.len() as f64);
        }
        
        Ok(())
    }
    
    pub async fn pop(&self) -> Option<Message> {
        let mut memory_queue = self.memory_queue.lock().await;
        
        if let Some(message) = memory_queue.pop_front() {
            self.metrics.gauge("queue.memory.depth", memory_queue.len() as f64);
            return Some(message);
        }
        
        // Try disk queue
        if let Some(message) = self.disk_queue.pop().await {
            self.metrics.gauge("queue.disk.depth", self.disk_queue.len() as f64);
            return Some(message);
        }
        
        None
    }
}
```

### NSQLookupd Design

#### Core Components

```rust
pub struct NsqlookupdServer {
    config: NsqlookupdConfig,
    registry: Arc<RwLock<RegistrationDB>>,
    metrics: Metrics,
    tcp_server: TcpServer,
    http_server: HttpServer,
}

pub struct RegistrationDB {
    producers: HashMap<String, Producer>,
    topics: HashMap<String, TopicInfo>,
    channels: HashMap<String, ChannelInfo>,
    tombstones: HashMap<String, Tombstone>,
}
```

#### Service Discovery

```rust
impl NsqlookupdServer {
    pub async fn register_producer(&self, producer: Producer) -> Result<(), NsqError> {
        let mut registry = self.registry.write().await;
        registry.producers.insert(producer.id().clone(), producer);
        
        self.metrics.gauge("producers.registered", registry.producers.len() as f64);
        Ok(())
    }
    
    pub async fn lookup_topic(&self, topic_name: &str) -> Result<Vec<Producer>, NsqError> {
        let registry = self.registry.read().await;
        
        let producers: Vec<Producer> = registry.producers
            .values()
            .filter(|p| p.has_topic(topic_name))
            .cloned()
            .collect();
        
        self.metrics.counter("lookups.topic", 1);
        Ok(producers)
    }
    
    pub async fn lookup_channel(&self, topic_name: &str, channel_name: &str) -> Result<Vec<Producer>, NsqError> {
        let registry = self.registry.read().await;
        
        let producers: Vec<Producer> = registry.producers
            .values()
            .filter(|p| p.has_channel(topic_name, channel_name))
            .cloned()
            .collect();
        
        self.metrics.counter("lookups.channel", 1);
        Ok(producers)
    }
}
```

### NSQAdmin Design

#### Core Components

```rust
pub struct NsqadminServer {
    config: NsqadminConfig,
    lookupd_client: NsqlookupdClient,
    metrics: Metrics,
    http_server: HttpServer,
    ui_server: UiServer,
}

pub struct NsqlookupdClient {
    http_client: reqwest::Client,
    endpoints: Vec<String>,
}
```

#### Statistics Aggregation

```rust
impl NsqadminServer {
    pub async fn get_cluster_stats(&self) -> Result<ClusterStats, NsqError> {
        let mut cluster_stats = ClusterStats::new();
        
        // Get all NSQD nodes
        let nodes = self.lookupd_client.get_nodes().await?;
        
        for node in nodes {
            // Get stats from each node
            let node_stats = self.get_node_stats(&node).await?;
            cluster_stats.merge(node_stats);
        }
        
        Ok(cluster_stats)
    }
    
    async fn get_node_stats(&self, node: &Node) -> Result<NodeStats, NsqError> {
        let response = self.http_client
            .get(&format!("http://{}:{}/stats", node.hostname, node.http_port))
            .send()
            .await?;
        
        let stats: NodeStats = response.json().await?;
        Ok(stats)
    }
}
```

## Data Flow

### Message Publishing Flow

```
Publisher ──┐
            │
            ▼
         NSQD ──┐
                │
                ▼
            Topic ──┐
                   │
                   ▼
               Channel ──┐
                         │
                         ▼
                      Client
```

### Message Consumption Flow

```
Client ──┐
         │
         ▼
      Channel ──┐
                │
                ▼
             Topic ──┐
                     │
                     ▼
                  NSQD ──┐
                         │
                         ▼
                      Consumer
```

### Service Discovery Flow

```
NSQD ──┐
       │
       ▼
   NSQLookupd ──┐
                │
                ▼
             Client
```

## Protocol Design

### TCP Protocol

#### Frame Format

```
[4 bytes: Frame Type][4 bytes: Frame Size][Frame Data]
```

#### Frame Types

- `0`: Response
- `1`: Error
- `2`: Message

#### Message Frame

```
[8 bytes: Timestamp][2 bytes: Attempts][16 bytes: Message ID][Message Body]
```

#### Commands

```rust
pub enum Command {
    Identify(IdentifyRequest),
    Subscribe(SubscribeRequest),
    Publish(PublishRequest),
    Mpub(MpubRequest),
    Ready(ReadyRequest),
    Finish(FinishRequest),
    Requeue(RequeueRequest),
    Nop,
    Close,
}
```

### HTTP Protocol

#### RESTful Endpoints

```rust
// NSQD HTTP API
GET  /ping
GET  /info
GET  /stats
POST /pub?topic=<topic>
POST /mpub?topic=<topic>
POST /topic/create?topic=<topic>
POST /topic/delete?topic=<topic>
POST /topic/pause?topic=<topic>
POST /topic/unpause?topic=<topic>
POST /channel/create?topic=<topic>&channel=<channel>
POST /channel/delete?topic=<topic>&channel=<channel>
POST /channel/pause?topic=<topic>&channel=<channel>
POST /channel/unpause?topic=<topic>&channel=<channel>

// NSQLookupd HTTP API
GET  /ping
GET  /info
GET  /lookup?topic=<topic>
GET  /lookup?topic=<topic>&channel=<channel>
GET  /topics
GET  /channels?topic=<topic>
GET  /nodes
POST /topic/delete?topic=<topic>
POST /channel/delete?topic=<topic>&channel=<channel>
POST /tombstone_topic_producer?topic=<topic>&node=<node>

// NSQAdmin HTTP API
GET  /ping
GET  /info
GET  /api/stats
GET  /api/topics
GET  /api/channels?topic=<topic>
GET  /api/nodes
```

## Storage Design

### Memory Storage

#### In-Memory Queues

```rust
pub struct MemoryQueue {
    queue: VecDeque<Message>,
    max_size: usize,
    metrics: Metrics,
}

impl MemoryQueue {
    pub fn push(&mut self, message: Message) -> Result<(), NsqError> {
        if self.queue.len() >= self.max_size {
            return Err(NsqError::QueueFull);
        }
        
        self.queue.push_back(message);
        self.metrics.gauge("queue.memory.depth", self.queue.len() as f64);
        Ok(())
    }
    
    pub fn pop(&mut self) -> Option<Message> {
        let message = self.queue.pop_front();
        self.metrics.gauge("queue.memory.depth", self.queue.len() as f64);
        message
    }
}
```

### Disk Storage

#### Disk Queue Implementation

```rust
pub struct DiskQueue {
    data_path: PathBuf,
    max_size: usize,
    sync_every: usize,
    sync_timeout: Duration,
    metrics: Metrics,
}

impl DiskQueue {
    pub async fn push(&self, message: Message) -> Result<(), NsqError> {
        let data = bincode::serialize(&message)?;
        let file_path = self.get_next_file_path();
        
        let mut file = File::create(&file_path).await?;
        file.write_all(&data).await?;
        file.sync_all().await?;
        
        self.metrics.gauge("queue.disk.depth", self.get_file_count() as f64);
        Ok(())
    }
    
    pub async fn pop(&self) -> Option<Message> {
        let file_path = self.get_next_file_path();
        
        if let Ok(data) = tokio::fs::read(&file_path).await {
            if let Ok(message) = bincode::deserialize(&data) {
                tokio::fs::remove_file(&file_path).await.ok();
                self.metrics.gauge("queue.disk.depth", self.get_file_count() as f64);
                return Some(message);
            }
        }
        
        None
    }
}
```

## Network Design

### TCP Server

#### Connection Handling

```rust
pub struct TcpServer {
    listener: TcpListener,
    clients: Arc<RwLock<HashMap<String, Client>>>,
    config: TcpConfig,
    metrics: Metrics,
}

impl TcpServer {
    pub async fn run(&self) -> Result<(), NsqError> {
        loop {
            let (stream, addr) = self.listener.accept().await?;
            let client_id = generate_client_id();
            
            let client = Client::new(client_id.clone(), stream, self.config.clone())?;
            self.clients.write().await.insert(client_id, client);
            
            // Handle client in separate task
            tokio::spawn(self.handle_client(client));
        }
    }
    
    async fn handle_client(&self, mut client: Client) {
        while let Some(command) = client.recv_command().await {
            match command {
                Command::Identify(req) => {
                    client.identify(req).await;
                }
                Command::Subscribe(req) => {
                    client.subscribe(req).await;
                }
                Command::Publish(req) => {
                    client.publish(req).await;
                }
                Command::Ready(req) => {
                    client.ready(req).await;
                }
                Command::Finish(req) => {
                    client.finish(req).await;
                }
                Command::Requeue(req) => {
                    client.requeue(req).await;
                }
                Command::Nop => {
                    client.nop().await;
                }
                Command::Close => {
                    break;
                }
            }
        }
    }
}
```

### HTTP Server

#### Request Handling

```rust
pub struct HttpServer {
    router: Router,
    config: HttpConfig,
    metrics: Metrics,
}

impl HttpServer {
    pub fn new(config: HttpConfig) -> Self {
        let router = Router::new()
            .route("/ping", get(handle_ping))
            .route("/info", get(handle_info))
            .route("/stats", get(handle_stats))
            .route("/pub", post(handle_pub))
            .route("/mpub", post(handle_mpub))
            .route("/topic/create", post(handle_topic_create))
            .route("/topic/delete", post(handle_topic_delete))
            .route("/topic/pause", post(handle_topic_pause))
            .route("/topic/unpause", post(handle_topic_unpause))
            .route("/channel/create", post(handle_channel_create))
            .route("/channel/delete", post(handle_channel_delete))
            .route("/channel/pause", post(handle_channel_pause))
            .route("/channel/unpause", post(handle_channel_unpause));
        
        Self {
            router,
            config,
            metrics: Metrics::new(),
        }
    }
}
```

## Concurrency Model

### Async/Await Pattern

NSQ Rust uses Rust's async/await model for concurrent operations:

```rust
pub async fn publish_message(&self, topic: &str, message: &[u8]) -> Result<(), NsqError> {
    let topic = self.get_topic(topic).await?;
    topic.publish(Message::new(message)).await?;
    Ok(())
}
```

### Lock-Free Data Structures

Where possible, NSQ Rust uses lock-free data structures:

```rust
use crossbeam_channel::{unbounded, Sender, Receiver};

pub struct MessageQueue {
    sender: Sender<Message>,
    receiver: Receiver<Message>,
}

impl MessageQueue {
    pub fn new() -> Self {
        let (sender, receiver) = unbounded();
        Self { sender, receiver }
    }
    
    pub fn push(&self, message: Message) -> Result<(), NsqError> {
        self.sender.send(message)?;
        Ok(())
    }
    
    pub async fn pop(&self) -> Option<Message> {
        self.receiver.recv().ok()
    }
}
```

### Worker Pools

NSQ Rust uses worker pools for CPU-intensive tasks:

```rust
pub struct WorkerPool {
    workers: Vec<Worker>,
    task_sender: Sender<Task>,
}

impl WorkerPool {
    pub fn new(num_workers: usize) -> Self {
        let (task_sender, task_receiver) = unbounded();
        let mut workers = Vec::new();
        
        for i in 0..num_workers {
            let worker = Worker::new(i, task_receiver.clone());
            workers.push(worker);
        }
        
        Self { workers, task_sender }
    }
    
    pub async fn submit(&self, task: Task) -> Result<(), NsqError> {
        self.task_sender.send(task)?;
        Ok(())
    }
}
```

## Error Handling

### Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum NsqError {
    #[error("Invalid command: {0}")]
    InvalidCommand(String),
    
    #[error("Bad topic: {0}")]
    BadTopic(String),
    
    #[error("Bad channel: {0}")]
    BadChannel(String),
    
    #[error("Bad message: {0}")]
    BadMessage(String),
    
    #[error("Publish failed: {0}")]
    PublishFailed(String),
    
    #[error("Queue full")]
    QueueFull,
    
    #[error("Connection closed")]
    ConnectionClosed,
    
    #[error("Authentication failed: {0}")]
    AuthFailed(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),
    
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
}
```

### Error Recovery

```rust
impl NsqdServer {
    pub async fn handle_error(&self, error: NsqError) {
        match error {
            NsqError::ConnectionClosed => {
                self.metrics.counter("errors.connection_closed", 1);
                // Clean up client resources
            }
            NsqError::QueueFull => {
                self.metrics.counter("errors.queue_full", 1);
                // Trigger backpressure
            }
            NsqError::AuthFailed(_) => {
                self.metrics.counter("errors.auth_failed", 1);
                // Close connection
            }
            _ => {
                self.metrics.counter("errors.other", 1);
                // Log error
                tracing::error!("Unexpected error: {}", error);
            }
        }
    }
}
```

## Performance Considerations

### Memory Management

#### Memory Pools

```rust
pub struct MessagePool {
    pool: Arc<Mutex<Vec<Message>>>,
    max_size: usize,
}

impl MessagePool {
    pub fn new(max_size: usize) -> Self {
        Self {
            pool: Arc::new(Mutex::new(Vec::with_capacity(max_size))),
            max_size,
        }
    }
    
    pub fn get(&self) -> Option<Message> {
        self.pool.lock().unwrap().pop()
    }
    
    pub fn put(&self, mut message: Message) {
        let mut pool = self.pool.lock().unwrap();
        if pool.len() < self.max_size {
            message.clear();
            pool.push(message);
        }
    }
}
```

#### Zero-Copy Operations

```rust
use bytes::{Bytes, BytesMut};

pub struct Message {
    id: [u8; 16],
    body: Bytes,
    timestamp: u64,
    attempts: u16,
}

impl Message {
    pub fn new(body: &[u8]) -> Self {
        Self {
            id: generate_id(),
            body: Bytes::from(body),
            timestamp: current_timestamp(),
            attempts: 0,
        }
    }
    
    pub fn body(&self) -> &[u8] {
        &self.body
    }
}
```

### CPU Optimization

#### SIMD Operations

```rust
use std::arch::x86_64::*;

pub fn fast_memcpy(dst: &mut [u8], src: &[u8]) {
    unsafe {
        let mut i = 0;
        while i + 16 <= src.len() {
            let data = _mm_loadu_si128(src.as_ptr().add(i) as *const __m128i);
            _mm_storeu_si128(dst.as_mut_ptr().add(i) as *mut __m128i, data);
            i += 16;
        }
        
        // Handle remaining bytes
        while i < src.len() {
            dst[i] = src[i];
            i += 1;
        }
    }
}
```

#### Branch Prediction

```rust
pub fn process_message(message: &Message) -> bool {
    // Use likely/unlikely hints for branch prediction
    if likely!(message.attempts < MAX_ATTEMPTS) {
        // Fast path
        true
    } else {
        // Slow path
        false
    }
}
```

## Scalability

### Horizontal Scaling

#### Load Balancing

```rust
pub struct LoadBalancer {
    nodes: Vec<Node>,
    strategy: LoadBalanceStrategy,
    metrics: Metrics,
}

pub enum LoadBalanceStrategy {
    RoundRobin,
    LeastConnections,
    WeightedRoundRobin,
    ConsistentHash,
}

impl LoadBalancer {
    pub fn select_node(&self) -> Option<&Node> {
        match self.strategy {
            LoadBalanceStrategy::RoundRobin => self.round_robin(),
            LoadBalanceStrategy::LeastConnections => self.least_connections(),
            LoadBalanceStrategy::WeightedRoundRobin => self.weighted_round_robin(),
            LoadBalanceStrategy::ConsistentHash => self.consistent_hash(),
        }
    }
}
```

#### Sharding

```rust
pub struct ShardedTopic {
    shards: Vec<TopicShard>,
    shard_count: usize,
    hash_function: fn(&str) -> usize,
}

impl ShardedTopic {
    pub fn new(shard_count: usize) -> Self {
        let mut shards = Vec::new();
        for i in 0..shard_count {
            shards.push(TopicShard::new(i));
        }
        
        Self {
            shards,
            shard_count,
            hash_function: |key| {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                
                let mut hasher = DefaultHasher::new();
                key.hash(&mut hasher);
                hasher.finish() as usize % shard_count
            },
        }
    }
    
    pub fn get_shard(&self, key: &str) -> &TopicShard {
        let shard_index = (self.hash_function)(key);
        &self.shards[shard_index]
    }
}
```

### Vertical Scaling

#### Resource Optimization

```rust
pub struct ResourceManager {
    cpu_cores: usize,
    memory_limit: usize,
    disk_limit: usize,
    network_limit: usize,
}

impl ResourceManager {
    pub fn optimize_for_workload(&self, workload: WorkloadType) -> ResourceConfig {
        match workload {
            WorkloadType::HighThroughput => ResourceConfig {
                worker_pools: self.cpu_cores * 2,
                memory_queue_size: 50000,
                disk_queue_size: 1000000,
                batch_size: 1000,
            },
            WorkloadType::LowLatency => ResourceConfig {
                worker_pools: self.cpu_cores,
                memory_queue_size: 10000,
                disk_queue_size: 100000,
                batch_size: 100,
            },
            WorkloadType::Balanced => ResourceConfig {
                worker_pools: self.cpu_cores,
                memory_queue_size: 25000,
                disk_queue_size: 500000,
                batch_size: 500,
            },
        }
    }
}
```

## Security

### Authentication

#### Token-Based Authentication

```rust
pub struct AuthManager {
    tokens: HashMap<String, TokenInfo>,
    secret_key: [u8; 32],
}

pub struct TokenInfo {
    user_id: String,
    permissions: Vec<Permission>,
    expires_at: u64,
}

impl AuthManager {
    pub fn generate_token(&self, user_id: &str, permissions: Vec<Permission>) -> String {
        let token_info = TokenInfo {
            user_id: user_id.to_string(),
            permissions,
            expires_at: current_timestamp() + 3600, // 1 hour
        };
        
        let token_data = bincode::serialize(&token_info).unwrap();
        let signature = self.sign(&token_data);
        
        base64::encode(&[token_data, signature].concat())
    }
    
    pub fn validate_token(&self, token: &str) -> Result<TokenInfo, AuthError> {
        let decoded = base64::decode(token)?;
        let (token_data, signature) = decoded.split_at(decoded.len() - 32);
        
        if !self.verify_signature(token_data, signature) {
            return Err(AuthError::InvalidSignature);
        }
        
        let token_info: TokenInfo = bincode::deserialize(token_data)?;
        
        if token_info.expires_at < current_timestamp() {
            return Err(AuthError::TokenExpired);
        }
        
        Ok(token_info)
    }
}
```

### Encryption

#### TLS Support

```rust
pub struct TlsConfig {
    cert_file: PathBuf,
    key_file: PathBuf,
    client_auth_policy: ClientAuthPolicy,
    min_version: TlsVersion,
    max_version: TlsVersion,
    cipher_suites: Vec<CipherSuite>,
}

impl TlsConfig {
    pub fn create_server_config(&self) -> Result<ServerConfig, TlsError> {
        let cert_chain = load_cert_chain(&self.cert_file)?;
        let private_key = load_private_key(&self.key_file)?;
        
        let mut config = ServerConfig::builder()
            .with_cipher_suites(&self.cipher_suites)
            .with_safe_default_kx_groups()
            .with_protocol_versions(&[&self.min_version, &self.max_version])?
            .with_client_cert_verifier(self.client_auth_policy.create_verifier())
            .with_single_cert(cert_chain, private_key)?;
        
        config.ignore_client_order = true;
        Ok(config)
    }
}
```

## Monitoring

### Metrics Collection

#### Prometheus Metrics

```rust
pub struct PrometheusMetrics {
    registry: Registry,
    counters: HashMap<String, Counter>,
    gauges: HashMap<String, Gauge>,
    histograms: HashMap<String, Histogram>,
}

impl PrometheusMetrics {
    pub fn new() -> Self {
        let registry = Registry::new();
        
        Self {
            registry,
            counters: HashMap::new(),
            gauges: HashMap::new(),
            histograms: HashMap::new(),
        }
    }
    
    pub fn counter(&self, name: &str, value: u64) {
        if let Some(counter) = self.counters.get(name) {
            counter.inc_by(value);
        }
    }
    
    pub fn gauge(&self, name: &str, value: f64) {
        if let Some(gauge) = self.gauges.get(name) {
            gauge.set(value);
        }
    }
    
    pub fn histogram(&self, name: &str, value: f64) {
        if let Some(histogram) = self.histograms.get(name) {
            histogram.observe(value);
        }
    }
}
```

#### Health Checks

```rust
pub struct HealthChecker {
    checks: Vec<Box<dyn HealthCheck>>,
    metrics: Metrics,
}

pub trait HealthCheck: Send + Sync {
    fn name(&self) -> &str;
    fn check(&self) -> HealthStatus;
}

pub enum HealthStatus {
    Healthy,
    Unhealthy(String),
    Degraded(String),
}

impl HealthChecker {
    pub fn add_check(&mut self, check: Box<dyn HealthCheck>) {
        self.checks.push(check);
    }
    
    pub fn run_checks(&self) -> HealthReport {
        let mut report = HealthReport::new();
        
        for check in &self.checks {
            let status = check.check();
            report.add_check(check.name(), status);
        }
        
        report
    }
}
```

### Logging

#### Structured Logging

```rust
use tracing::{info, warn, error, debug};

pub struct Logger {
    level: LogLevel,
    format: LogFormat,
    output: LogOutput,
}

pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

pub enum LogFormat {
    Text,
    Json,
}

pub enum LogOutput {
    Stdout,
    Stderr,
    File(PathBuf),
    Syslog,
}

impl Logger {
    pub fn log_message(&self, level: LogLevel, message: &str, fields: &[(&str, &str)]) {
        match level {
            LogLevel::Debug => debug!(fields = ?fields, "{}", message),
            LogLevel::Info => info!(fields = ?fields, "{}", message),
            LogLevel::Warn => warn!(fields = ?fields, "{}", message),
            LogLevel::Error => error!(fields = ?fields, "{}", message),
        }
    }
}
```

## Additional Resources

- [NSQ Architecture](https://nsq.io/overview/design.html)
- [Rust Async Book](https://rust-lang.github.io/async-book/)
- [Tokio Documentation](https://tokio.rs/)
- [Axum Documentation](https://docs.rs/axum/)
- [Prometheus Documentation](https://prometheus.io/docs/)
- [Tracing Documentation](https://tracing.rs/)
