//! nsq_to_nsq - Topic/channel replication tool

use clap::Parser;
use futures::SinkExt;
use nsq_protocol::{Command, Frame, FrameType, Message, NsqDecoder, NsqEncoder};
use tokio::net::TcpStream;
use tokio_stream::StreamExt;
use tokio_util::codec::{FramedRead, FramedWrite};
use tracing::{error, info, warn};

#[derive(Parser, Debug)]
#[command(name = "nsq_to_nsq")]
#[command(about = "NSQ topic/channel replication tool")]
struct Args {
    /// Source NSQd TCP addresses
    #[arg(long)]
    src_nsqd_tcp_address: Vec<String>,
    
    /// Source Lookupd HTTP addresses
    #[arg(long)]
    src_lookupd_http_address: Vec<String>,
    
    /// Source topic
    #[arg(long)]
    src_topic: String,
    
    /// Source channel
    #[arg(long)]
    src_channel: String,
    
    /// Destination NSQd TCP address
    #[arg(long)]
    dst_nsqd_tcp_address: String,
    
    /// Destination topic
    #[arg(long)]
    dst_topic: String,
    
    /// Destination channel (defaults to source channel if not specified)
    #[arg(long)]
    dst_channel: Option<String>,
    
    /// Buffer size for message replication
    #[arg(long, default_value = "1000")]
    buffer_size: usize,
    
    /// Batch size for publishing messages
    #[arg(long, default_value = "10")]
    batch_size: usize,
}

struct NsqReplicator {
    src_topic: String,
    src_channel: String,
    dst_topic: String,
    dst_channel: String,
    buffer_size: usize,
    batch_size: usize,
}

impl NsqReplicator {
    fn new(
        src_topic: String,
        src_channel: String,
        dst_topic: String,
        dst_channel: Option<String>,
        buffer_size: usize,
        batch_size: usize,
    ) -> Self {
        Self {
            src_topic,
            src_channel: src_channel.clone(),
            dst_topic,
            dst_channel: dst_channel.unwrap_or_else(|| src_channel),
            buffer_size,
            batch_size,
        }
    }

    async fn replicate(&self, src_address: &str, dst_address: &str) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting replication from {} to {}", src_address, dst_address);
        
        // Connect to source NSQd
        let src_stream = TcpStream::connect(src_address).await?;
        let (src_read_half, src_write_half) = src_stream.into_split();
        
        let mut src_framed_read = FramedRead::new(src_read_half, NsqDecoder::new());
        let mut src_framed_write = FramedWrite::new(src_write_half, NsqEncoder);
        
        // Connect to destination NSQd
        let dst_stream = TcpStream::connect(dst_address).await?;
        let (dst_read_half, dst_write_half) = dst_stream.into_split();
        
        let mut dst_framed_read = FramedRead::new(dst_read_half, NsqDecoder::new());
        let mut dst_framed_write = FramedWrite::new(dst_write_half, NsqEncoder);
        
        // Setup source connection
        self.setup_source_connection(&mut src_framed_read, &mut src_framed_write).await?;
        
        // Setup destination connection
        self.setup_destination_connection(&mut dst_framed_read, &mut dst_framed_write).await?;
        
        info!("Replicating messages from topic '{}' channel '{}' to topic '{}' channel '{}'",
            self.src_topic, self.src_channel, self.dst_topic, self.dst_channel);
        
        // Message replication loop
        let mut message_batch = Vec::new();
        let mut messages_processed = 0usize;
        
        while let Some(frame) = src_framed_read.next().await {
            let frame = frame?;
            
            match frame.frame_type {
                FrameType::Message => {
                    let message = Message::from_bytes(frame.body)?;
                    message_batch.push(message);
                    messages_processed += 1;
                    
                    // Periodically refresh RDY count to maintain flow
                    if messages_processed % (self.buffer_size / 4).max(1) == 0 {
                        let rdy_cmd = Command::Rdy { count: self.buffer_size as u32 };
                        let rdy_frame = Frame::new(FrameType::Response, rdy_cmd.to_bytes()?);
                        src_framed_write.send(rdy_frame).await?;
                    }
                    
                    // Publish batch when it reaches batch_size
                    if message_batch.len() >= self.batch_size {
                        self.publish_batch(&mut dst_framed_write, &message_batch).await?;
                        message_batch.clear();
                    }
                }
                FrameType::Response => {
                    info!("Source response: {}", String::from_utf8_lossy(&frame.body));
                }
                FrameType::Error => {
                    error!("Source error: {}", String::from_utf8_lossy(&frame.body));
                    return Err(format!("Source NSQ error: {}", String::from_utf8_lossy(&frame.body)).into());
                }
            }
        }
        
        // Publish remaining messages
        if !message_batch.is_empty() {
            self.publish_batch(&mut dst_framed_write, &message_batch).await?;
        }
        
        Ok(())
    }

    async fn setup_source_connection(
        &self,
        framed_read: &mut FramedRead<tokio::net::tcp::OwnedReadHalf, NsqDecoder>,
        framed_write: &mut FramedWrite<tokio::net::tcp::OwnedWriteHalf, NsqEncoder>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Send IDENTIFY command
        let identify_data = serde_json::json!({
            "client_id": "nsq_to_nsq_src",
            "hostname": "nsq_to_nsq_src",
            "user_agent": "nsq_to_nsq/1.0",
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
            info!("Source connection established");
        }
        
        // Subscribe to source topic/channel
        let sub_cmd = Command::Sub {
            topic: self.src_topic.clone(),
            channel: self.src_channel.clone(),
        };
        let sub_frame = Frame::new(FrameType::Response, sub_cmd.to_bytes()?);
        framed_write.send(sub_frame).await?;
        
        // Set ready count based on buffer_size to control in-flight messages
        let rdy_cmd = Command::Rdy { count: self.buffer_size as u32 };
        let rdy_frame = Frame::new(FrameType::Response, rdy_cmd.to_bytes()?);
        framed_write.send(rdy_frame).await?;
        
        info!("Source ready to receive up to {} in-flight messages", self.buffer_size);
        
        Ok(())
    }

    async fn setup_destination_connection(
        &self,
        framed_read: &mut FramedRead<tokio::net::tcp::OwnedReadHalf, NsqDecoder>,
        framed_write: &mut FramedWrite<tokio::net::tcp::OwnedWriteHalf, NsqEncoder>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Send IDENTIFY command
        let identify_data = serde_json::json!({
            "client_id": "nsq_to_nsq_dst",
            "hostname": "nsq_to_nsq_dst",
            "user_agent": "nsq_to_nsq/1.0",
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
            info!("Destination connection established");
        }
        
        Ok(())
    }

    async fn publish_batch(
        &self,
        framed_write: &mut FramedWrite<tokio::net::tcp::OwnedWriteHalf, NsqEncoder>,
        messages: &[Message],
    ) -> Result<(), Box<dyn std::error::Error>> {
        if messages.is_empty() {
            return Ok(());
        }
        
        if messages.len() == 1 {
            // Single message
            let pub_cmd = Command::Pub {
                topic: self.dst_topic.clone(),
                body: messages[0].body.clone(),
            };
            let pub_frame = Frame::new(FrameType::Response, pub_cmd.to_bytes()?);
            framed_write.send(pub_frame).await?;
        } else {
            // Batch messages
            let bodies: Vec<bytes::Bytes> = messages.iter().map(|m| m.body.clone()).collect();
            let mpub_cmd = Command::Mpub {
                topic: self.dst_topic.clone(),
                bodies,
            };
            let mpub_frame = Frame::new(FrameType::Response, mpub_cmd.to_bytes()?);
            framed_write.send(mpub_frame).await?;
        }
        
        info!("Published batch of {} messages to destination", messages.len());
        Ok(())
    }
}

async fn discover_nsqd_addresses(lookupd_addresses: &[String]) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut nsqd_addresses = Vec::new();
    
    for lookupd_addr in lookupd_addresses {
        let url = format!("http://{}/nodes", lookupd_addr);
        match reqwest::get(&url).await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<serde_json::Value>().await {
                        Ok(nodes) => {
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
                        Err(e) => {
                            warn!("Failed to parse JSON from {}: {}", lookupd_addr, e);
                        }
                    }
                } else {
                    warn!("Failed to query lookupd {}: HTTP {}", lookupd_addr, response.status());
                }
            }
            Err(e) => {
                warn!("Failed to connect to lookupd {}: {}", lookupd_addr, e);
            }
        }
    }
    
    Ok(nsqd_addresses)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    let args = Args::parse();
    
    if args.src_nsqd_tcp_address.is_empty() && args.src_lookupd_http_address.is_empty() {
        eprintln!("Error: At least one source NSQd TCP address or Lookupd HTTP address must be specified");
        std::process::exit(1);
    }
    
    let mut src_nsqd_addresses = args.src_nsqd_tcp_address;
    
    // Discover source NSQd addresses from lookupd if provided
    if !args.src_lookupd_http_address.is_empty() {
        match discover_nsqd_addresses(&args.src_lookupd_http_address).await {
            Ok(discovered) => {
                info!("Discovered {} source NSQd instances from lookupd", discovered.len());
                src_nsqd_addresses.extend(discovered);
            }
            Err(e) => {
                warn!("Failed to discover NSQd addresses from lookupd: {}", e);
            }
        }
    }
    
    if src_nsqd_addresses.is_empty() {
        eprintln!("Error: No source NSQd addresses available");
        std::process::exit(1);
    }
    
    let replicator = NsqReplicator::new(
        args.src_topic,
        args.src_channel,
        args.dst_topic,
        args.dst_channel,
        args.buffer_size,
        args.batch_size,
    );
    
    // Try to connect to the first available source NSQd
    let mut connected = false;
    for src_address in &src_nsqd_addresses {
        match replicator.replicate(src_address, &args.dst_nsqd_tcp_address).await {
            Ok(_) => {
                connected = true;
                break;
            }
            Err(e) => {
                error!("Failed to replicate from {} to {}: {}", src_address, args.dst_nsqd_tcp_address, e);
                continue;
            }
        }
    }
    
    if !connected {
        eprintln!("Error: Failed to connect to any source NSQd instance");
        std::process::exit(1);
    }
    
    Ok(())
}

