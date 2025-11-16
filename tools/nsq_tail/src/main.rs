//! nsq_tail - Tail NSQ topics like tail -f

use clap::Parser;
use nsq_protocol::{Command, Frame, FrameType, Message, NsqDecoder, NsqEncoder};
use tokio::net::TcpStream;
use tokio_stream::StreamExt;
use tokio_util::codec::{FramedRead, FramedWrite};
use futures::SinkExt;
use tracing::{error, info, warn};

#[derive(Parser, Debug)]
#[command(name = "nsq_tail")]
#[command(about = "Tail NSQ topics like tail -f")]
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
    
    /// Show message metadata (timestamp, attempts, etc.)
    #[arg(long)]
    verbose: bool,
    
    /// Maximum number of messages to display before exiting
    #[arg(long)]
    max_messages: Option<u64>,
}

struct NsqConsumer {
    topic: String,
    channel: String,
    verbose: bool,
    max_messages: Option<u64>,
    message_count: u64,
}

impl NsqConsumer {
    fn new(topic: String, channel: String, verbose: bool, max_messages: Option<u64>) -> Self {
        Self {
            topic,
            channel,
            verbose,
            max_messages,
            message_count: 0,
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
            "client_id": "nsq_tail",
            "hostname": "nsq_tail",
            "user_agent": "nsq_tail/1.0",
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
        
        // Set ready count
        let rdy_cmd = Command::Rdy { count: 1 };
        let rdy_frame = Frame::new(FrameType::Response, rdy_cmd.to_bytes()?);
        framed_write.send(rdy_frame).await?;
        
        info!("Subscribed to topic '{}' channel '{}'", self.topic, self.channel);
        
        // Main message processing loop
        while let Some(frame) = framed_read.next().await {
            let frame = frame?;
            
            match frame.frame_type {
                FrameType::Message => {
                    self.handle_message(frame.body).await?;
                    
                    // Send RDY for next message
                    let rdy_cmd = Command::Rdy { count: 1 };
                    let rdy_frame = Frame::new(FrameType::Response, rdy_cmd.to_bytes()?);
                    framed_write.send(rdy_frame).await?;
                    
                    // Check if we've reached max messages
                    if let Some(max) = self.max_messages {
                        if self.message_count >= max {
                            info!("Reached maximum message count ({}), exiting", max);
                            break;
                        }
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

    async fn handle_message(&mut self, message_data: bytes::Bytes) -> Result<(), Box<dyn std::error::Error>> {
        let message = Message::from_bytes(message_data)?;
        self.message_count += 1;
        
        if self.verbose {
            println!("[{}] {} (attempts: {}, size: {} bytes)", 
                message.timestamp.format("%Y-%m-%d %H:%M:%S%.3f"),
                String::from_utf8_lossy(&message.body),
                message.attempts,
                message.body.len()
            );
        } else {
            println!("{}", String::from_utf8_lossy(&message.body));
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
    
    let mut consumer = NsqConsumer::new(
        args.topic,
        args.channel,
        args.verbose,
        args.max_messages,
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

