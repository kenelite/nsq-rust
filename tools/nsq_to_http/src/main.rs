//! nsq_to_http - Consumer that posts messages to HTTP endpoints

use clap::Parser;
use futures::SinkExt;
use nsq_protocol::{Command, Frame, FrameType, Message, NsqDecoder, NsqEncoder};
use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::Semaphore;
use tokio_stream::StreamExt;
use tokio_util::codec::{FramedRead, FramedWrite};
use tracing::{error, info, warn};

#[derive(Parser, Debug)]
#[command(name = "nsq_to_http")]
#[command(about = "NSQ consumer that posts messages to HTTP endpoints")]
struct Args {
    /// NSQd TCP addresses
    #[arg(long)]
    nsqd_tcp_address: Vec<String>,
    
    /// Lookupd HTTP addresses
    #[arg(long)]
    lookupd_http_address: Vec<String>,
    
    /// Topic to subscribe to
    #[arg(long)]
    topic: String,
    
    /// Channel name
    #[arg(long)]
    channel: String,
    
    /// HTTP endpoint URL
    #[arg(long)]
    http_endpoint: String,
    
    /// HTTP method (GET, POST, PUT, PATCH)
    #[arg(long, default_value = "POST")]
    http_method: String,
    
    /// HTTP headers (format: "Header: Value")
    #[arg(long)]
    http_headers: Vec<String>,
    
    /// HTTP timeout in seconds
    #[arg(long, default_value = "30")]
    http_timeout: u64,
    
    /// Maximum number of concurrent HTTP requests
    #[arg(long, default_value = "10")]
    max_concurrent_requests: usize,
    
    /// Retry failed requests
    #[arg(long)]
    retry_failed: bool,
    
    /// Maximum retry attempts
    #[arg(long, default_value = "3")]
    max_retries: u32,
}

struct HttpPoster {
    client: Client,
    endpoint: String,
    method: String,
    headers: Vec<(String, String)>,
    max_concurrent: usize,
    semaphore: Arc<Semaphore>,
    retry_failed: bool,
    max_retries: u32,
}

impl HttpPoster {
    fn new(
        endpoint: String,
        method: String,
        headers: Vec<String>,
        timeout: u64,
        max_concurrent: usize,
        retry_failed: bool,
        max_retries: u32,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout))
            .build()?;
        
        let mut parsed_headers = Vec::new();
        for header in headers {
            if let Some((key, value)) = header.split_once(':') {
                parsed_headers.push((key.trim().to_string(), value.trim().to_string()));
            } else {
                return Err(format!("Invalid header format: {}", header).into());
            }
        }
        
        Ok(Self {
            client,
            endpoint,
            method,
            headers: parsed_headers,
            max_concurrent,
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            retry_failed,
            max_retries,
        })
    }

    async fn post_message(&self, message: &Message) -> Result<(), Box<dyn std::error::Error>> {
        // Acquire semaphore permit to control concurrency
        let _permit = self.semaphore.acquire().await
            .map_err(|e| format!("Failed to acquire semaphore: {}", e))?;
        
        info!("Processing message (concurrent requests: {})", 
            self.max_concurrent - self.semaphore.available_permits());
        
        let mut request = match self.method.to_uppercase().as_str() {
            "GET" => self.client.get(&self.endpoint),
            "POST" => self.client.post(&self.endpoint),
            "PUT" => self.client.put(&self.endpoint),
            "PATCH" => self.client.patch(&self.endpoint),
            _ => return Err(format!("Unsupported HTTP method: {}", self.method).into()),
        };
        
        // Add headers
        for (key, value) in &self.headers {
            request = request.header(key, value);
        }
        
        // Add message data as JSON body
        let message_data = serde_json::json!({
            "id": message.id.to_string(),
            "timestamp": message.timestamp.to_rfc3339(),
            "attempts": message.attempts,
            "body": String::from_utf8_lossy(&message.body),
            "size": message.body.len()
        });
        
        request = request.json(&message_data);
        
        // Send request with retries
        let mut last_error = None;
        for attempt in 0..=self.max_retries {
            match request.try_clone().unwrap().send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        info!("Successfully posted message to {} (status: {})", 
                            self.endpoint, response.status());
                        return Ok(());
                    } else {
                        let error_msg = format!("HTTP error: {}", response.status());
                        if attempt < self.max_retries && self.retry_failed {
                            warn!("Attempt {} failed: {}, retrying...", attempt + 1, error_msg);
                            tokio::time::sleep(Duration::from_millis(100 * (attempt + 1) as u64)).await;
                            continue;
                        } else {
                            return Err(error_msg.into());
                        }
                    }
                }
                Err(e) => {
                    last_error = Some(e);
                    if attempt < self.max_retries && self.retry_failed {
                        warn!("Attempt {} failed: {}, retrying...", attempt + 1, last_error.as_ref().unwrap());
                        tokio::time::sleep(Duration::from_millis(100 * (attempt + 1) as u64)).await;
                    }
                }
            }
        }
        
        Err(last_error.unwrap().into())
    }
}

struct NsqToHttpConsumer {
    topic: String,
    channel: String,
    http_poster: Arc<HttpPoster>,
}

impl NsqToHttpConsumer {
    fn new(topic: String, channel: String, http_poster: Arc<HttpPoster>) -> Self {
        Self {
            topic,
            channel,
            http_poster,
        }
    }

    async fn connect_and_consume(&mut self, address: &str) -> Result<(), Box<dyn std::error::Error>> {
        info!("Connecting to NSQd at {}", address);
        
        let stream = TcpStream::connect(address).await?;
        let (read_half, write_half) = stream.into_split();
        
        let mut framed_read = FramedRead::new(read_half, NsqDecoder::new());
        let mut framed_write = FramedWrite::new(write_half, NsqEncoder);
        
        // Send IDENTIFY command
        let identify_data = serde_json::json!({
            "client_id": "nsq_to_http",
            "hostname": "nsq_to_http",
            "user_agent": "nsq_to_http/1.0",
            "feature_negotiation": true,
            "heartbeat_interval": 30000,
            "output_buffer_size": 16384,
            "output_buffer_timeout": 250
        });
        
        let identify_cmd = Command::Identify { data: identify_data };
        let identify_frame = Frame::new(FrameType::Response, identify_cmd.to_bytes()?);
        framed_write.send(identify_frame).await?;
        
        // Wait for OK response
        if let Some(frame) = framed_read.next().await {
            let frame = frame?;
            if frame.frame_type != FrameType::Response {
                return Err("Expected OK response after IDENTIFY".into());
            }
            info!("Connected successfully");
        }
        
        // Subscribe to topic/channel
        let sub_cmd = Command::Sub {
            topic: self.topic.clone(),
            channel: self.channel.clone(),
        };
        let sub_frame = Frame::new(FrameType::Response, sub_cmd.to_bytes()?);
        framed_write.send(sub_frame).await?;
        
        // Set ready count to max_concurrent for parallel processing
        let max_concurrent = self.http_poster.max_concurrent;
        let rdy_cmd = Command::Rdy { count: max_concurrent as u32 };
        let rdy_frame = Frame::new(FrameType::Response, rdy_cmd.to_bytes()?);
        framed_write.send(rdy_frame).await?;
        
        info!("Subscribed to topic '{}' channel '{}' with RDY count {}", 
            self.topic, self.channel, max_concurrent);
        
        // Main message processing loop
        let mut in_flight = 0usize;
        while let Some(frame) = framed_read.next().await {
            let frame = frame?;
            
            match frame.frame_type {
                FrameType::Message => {
                    // Spawn async task to handle message concurrently
                    let http_poster = Arc::clone(&self.http_poster);
                    let message_data = frame.body;
                    
                    tokio::spawn(async move {
                        match Message::from_bytes(message_data) {
                            Ok(message) => {
                                match http_poster.post_message(&message).await {
                                    Ok(_) => {
                                        info!("Successfully posted message to HTTP endpoint");
                                    }
                                    Err(e) => {
                                        error!("Failed to post message to HTTP endpoint: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Failed to parse message: {}", e);
                            }
                        }
                    });
                    
                    in_flight += 1;
                    
                    // Periodically refresh RDY count to maintain flow
                    if in_flight >= max_concurrent / 2 {
                        let rdy_cmd = Command::Rdy { count: max_concurrent as u32 };
                        let rdy_frame = Frame::new(FrameType::Response, rdy_cmd.to_bytes()?);
                        framed_write.send(rdy_frame).await?;
                        in_flight = 0;
                    }
                }
                FrameType::Response => {
                    info!("Received response: {}", String::from_utf8_lossy(&frame.body));
                }
                FrameType::Error => {
                    error!("Received error: {}", String::from_utf8_lossy(&frame.body));
                    return Err(format!("NSQ error: {}", String::from_utf8_lossy(&frame.body)).into());
                }
            }
        }
        
        Ok(())
    }
}

async fn discover_nsqd_addresses(lookupd_addresses: &[String]) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut nsqd_addresses = Vec::new();
    
    for lookupd_addr in lookupd_addresses {
        let url = format!("http://{}/nodes", lookupd_addr);
        let response = reqwest::get(&url).await?;
        
        if response.status().is_success() {
            let nodes: serde_json::Value = response.json().await?;
            
            if let Some(producers) = nodes.get("producers") {
                if let Some(producers_array) = producers.as_array() {
                    for producer in producers_array {
                        if let Some(broadcast_address) = producer.get("broadcast_address") {
                            if let Some(tcp_port) = producer.get("tcp_port") {
                                let address = format!("{}:{}", 
                                    broadcast_address.as_str().unwrap_or("localhost"),
                                    tcp_port.as_u64().unwrap_or(4150)
                                );
                                nsqd_addresses.push(address);
                            }
                        }
                    }
                }
            }
        }
    }
    
    Ok(nsqd_addresses)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    let args = Args::parse();
    
    if args.nsqd_tcp_address.is_empty() && args.lookupd_http_address.is_empty() {
        eprintln!("Error: At least one NSQd TCP address or Lookupd HTTP address must be specified");
        std::process::exit(1);
    }
    
    let mut nsqd_addresses = args.nsqd_tcp_address;
    
    // Discover NSQd addresses from lookupd if provided
    if !args.lookupd_http_address.is_empty() {
        match discover_nsqd_addresses(&args.lookupd_http_address).await {
            Ok(discovered) => {
                info!("Discovered {} NSQd instances from lookupd", discovered.len());
                nsqd_addresses.extend(discovered);
            }
            Err(e) => {
                warn!("Failed to discover NSQd addresses from lookupd: {}", e);
            }
        }
    }
    
    if nsqd_addresses.is_empty() {
        eprintln!("Error: No NSQd addresses available");
        std::process::exit(1);
    }
    
    let http_poster = Arc::new(HttpPoster::new(
        args.http_endpoint,
        args.http_method,
        args.http_headers,
        args.http_timeout,
        args.max_concurrent_requests,
        args.retry_failed,
        args.max_retries,
    )?);
    
    let mut consumer = NsqToHttpConsumer::new(
        args.topic,
        args.channel,
        http_poster,
    );
    
    // Try to connect to the first available NSQd
    let mut connected = false;
    for address in &nsqd_addresses {
        match consumer.connect_and_consume(address).await {
            Ok(_) => {
                connected = true;
                break;
            }
            Err(e) => {
                error!("Failed to connect to {}: {}", address, e);
                continue;
            }
        }
    }
    
    if !connected {
        eprintln!("Error: Failed to connect to any NSQd instance");
        std::process::exit(1);
    }
    
    Ok(())
}

