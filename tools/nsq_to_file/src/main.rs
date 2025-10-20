//! NSQ to File - Consumer that writes messages to files

use clap::Parser;
use futures::SinkExt;
use nsq_protocol::{Command, Frame, FrameType, Message, NsqDecoder, NsqEncoder};
use std::path::PathBuf;
use std::time::Duration;
use tokio::fs::{File, OpenOptions};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::time::interval;
use tokio_stream::StreamExt;
use tokio_util::codec::{FramedRead, FramedWrite};
use tracing::{error, info, warn};

#[derive(Parser, Debug)]
#[command(name = "nsq_to_file")]
#[command(about = "NSQ consumer that writes messages to files")]
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
    
    /// Output directory
    #[arg(long, default_value = ".")]
    output_dir: String,
    
    /// Output filename pattern (supports {timestamp}, {topic}, {channel})
    #[arg(long, default_value = "{topic}_{channel}_{timestamp}.log")]
    filename_pattern: String,
    
    /// Maximum file size before rotation (in bytes)
    #[arg(long, default_value = "104857600")] // 100MB
    max_file_size: u64,
    
    /// Maximum number of files to keep
    #[arg(long, default_value = "10")]
    max_files: usize,
    
    /// Flush interval in seconds
    #[arg(long, default_value = "1")]
    flush_interval: u64,
}

struct FileWriter {
    output_dir: PathBuf,
    filename_pattern: String,
    max_file_size: u64,
    max_files: usize,
    current_file: Option<File>,
    current_file_path: Option<PathBuf>,
    current_file_size: u64,
    file_counter: u64,
}

impl FileWriter {
    fn new(
        output_dir: String,
        filename_pattern: String,
        max_file_size: u64,
        max_files: usize,
    ) -> Self {
        Self {
            output_dir: PathBuf::from(output_dir),
            filename_pattern,
            max_file_size,
            max_files,
            current_file: None,
            current_file_path: None,
            current_file_size: 0,
            file_counter: 0,
        }
    }

    async fn write_message(&mut self, message: &Message, topic: &str, channel: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Generate filename based on pattern
        let filename = self.filename_pattern
            .replace("{timestamp}", &chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string())
            .replace("{topic}", topic)
            .replace("{channel}", channel)
            .replace("{counter}", &self.file_counter.to_string());
        
        let file_path = self.output_dir.join(filename);
        
        // Check if we need to rotate the file
        if self.current_file_size >= self.max_file_size || 
           self.current_file_path.as_ref().map_or(true, |p| p != &file_path) {
            self.rotate_file(file_path).await?;
        }
        
        // Write message to file
        if let Some(file) = &mut self.current_file {
            let message_line = format!("[{}] {} (attempts: {}, size: {} bytes)\n",
                message.timestamp.format("%Y-%m-%d %H:%M:%S%.3f"),
                String::from_utf8_lossy(&message.body),
                message.attempts,
                message.body.len()
            );
            
            file.write_all(message_line.as_bytes()).await?;
            self.current_file_size += message_line.len() as u64;
        }
        
        Ok(())
    }

    async fn rotate_file(&mut self, new_file_path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        // Close current file
        if let Some(mut file) = self.current_file.take() {
            file.flush().await?;
        }
        
        // Clean up old files
        self.cleanup_old_files().await?;
        
        // Create new file
        std::fs::create_dir_all(&self.output_dir)?;
        
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&new_file_path)
            .await?;
        
        self.current_file = Some(file);
        self.current_file_path = Some(new_file_path);
        self.current_file_size = 0;
        self.file_counter += 1;
        
        info!("Rotated to new file: {:?}", self.current_file_path);
        
        Ok(())
    }

    async fn cleanup_old_files(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.output_dir.exists() {
            return Ok(());
        }
        
        let mut entries = tokio::fs::read_dir(&self.output_dir).await?;
        let mut files = Vec::new();
        
        while let Some(entry) = entries.next_entry().await? {
            if entry.file_type().await?.is_file() {
                if let Some(file_name) = entry.file_name().to_str() {
                    if file_name.contains(&self.filename_pattern.replace("{timestamp}", "").replace("{topic}", "").replace("{channel}", "").replace("{counter}", "")) {
                        files.push(entry.path());
                    }
                }
            }
        }
        
        // Sort by modification time (oldest first)
        files.sort_by(|a, b| {
            let a_meta = std::fs::metadata(a).unwrap();
            let b_meta = std::fs::metadata(b).unwrap();
            a_meta.modified().unwrap().cmp(&b_meta.modified().unwrap())
        });
        
        // Remove excess files
        while files.len() >= self.max_files {
            if let Some(old_file) = files.pop() {
                tokio::fs::remove_file(&old_file).await?;
                info!("Removed old file: {:?}", old_file);
            }
        }
        
        Ok(())
    }

    async fn flush(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(file) = &mut self.current_file {
            file.flush().await?;
        }
        Ok(())
    }
}

struct NsqToFileConsumer {
    topic: String,
    channel: String,
    file_writer: FileWriter,
}

impl NsqToFileConsumer {
    fn new(topic: String, channel: String, file_writer: FileWriter) -> Self {
        Self {
            topic,
            channel,
            file_writer,
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
            "client_id": "nsq_to_file",
            "hostname": "nsq_to_file",
            "user_agent": "nsq_to_file/1.0",
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
        
        // Start flush task
        let flush_interval = Duration::from_secs(1); // Default flush interval
        let mut flush_timer = interval(flush_interval);
        
        // Main message processing loop
        loop {
            tokio::select! {
                frame_result = framed_read.next() => {
                    match frame_result {
                        Some(frame) => {
                            let frame = frame?;
                            
                            match frame.frame_type {
                                FrameType::Message => {
                                    self.handle_message(frame.body).await?;
                                    
                                    // Send RDY for next message
                                    let rdy_cmd = Command::Rdy { count: 1 };
                                    let rdy_frame = Frame::new(FrameType::Response, rdy_cmd.to_bytes()?);
                                    framed_write.send(rdy_frame).await?;
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
                        None => {
                            info!("Connection closed");
                            break;
                        }
                    }
                }
                _ = flush_timer.tick() => {
                    self.file_writer.flush().await?;
                }
            }
        }
        
        Ok(())
    }

    async fn handle_message(&mut self, message_data: bytes::Bytes) -> Result<(), Box<dyn std::error::Error>> {
        let message = Message::from_bytes(message_data)?;
        
        self.file_writer.write_message(&message, &self.topic, &self.channel).await?;
        
        info!("Wrote message to file (size: {} bytes)", message.body.len());
        
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
    
    let file_writer = FileWriter::new(
        args.output_dir,
        args.filename_pattern,
        args.max_file_size,
        args.max_files,
    );
    
    let mut consumer = NsqToFileConsumer::new(
        args.topic,
        args.channel,
        file_writer,
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

