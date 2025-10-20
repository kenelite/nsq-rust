//! to_nsq - Producer that reads from stdin/files

use clap::Parser;
use nsq_protocol::{Command, Frame, FrameType, Message, NsqDecoder, NsqEncoder};
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader, stdin};
use tokio::net::TcpStream;
use tokio_stream::StreamExt;
use tokio_util::codec::{FramedRead, FramedWrite, Encoder};
use futures::SinkExt;
use tracing::{error, info, warn};
use url::Url;

#[derive(Parser, Debug)]
#[command(name = "to_nsq")]
#[command(about = "NSQ producer that reads from stdin/files")]
struct Args {
    /// NSQd TCP address
    #[arg(long)]
    nsqd_tcp_address: String,
    
    /// Topic to publish to
    #[arg(long)]
    topic: String,
    
    /// Input file (if not specified, reads from stdin)
    #[arg(long)]
    input_file: Option<String>,
    
    /// Batch size for publishing messages
    #[arg(long, default_value = "1")]
    batch_size: usize,
    
    /// Delay between messages in milliseconds
    #[arg(long, default_value = "0")]
    delay_ms: u64,
    
    /// Maximum message size in bytes
    #[arg(long, default_value = "1048576")] // 1MB
    max_message_size: usize,
    
    /// Read input line by line (for text files)
    #[arg(long)]
    line_by_line: bool,
    
    /// Add timestamp to each message
    #[arg(long)]
    add_timestamp: bool,
    
    /// Message prefix
    #[arg(long)]
    prefix: Option<String>,
}

struct NsqProducer {
    topic: String,
    batch_size: usize,
    delay_ms: u64,
    max_message_size: usize,
    add_timestamp: bool,
    prefix: Option<String>,
}

impl NsqProducer {
    fn new(
        topic: String,
        batch_size: usize,
        delay_ms: u64,
        max_message_size: usize,
        add_timestamp: bool,
        prefix: Option<String>,
    ) -> Self {
        Self {
            topic,
            batch_size,
            delay_ms,
            max_message_size,
            add_timestamp,
            prefix,
        }
    }

    async fn connect_and_publish(&self, address: &str) -> Result<(), Box<dyn std::error::Error>> {
        info!("Connecting to NSQd at {}", address);
        
        let stream = TcpStream::connect(address).await?;
        let (read_half, write_half) = stream.into_split();
        
        let mut framed_read = FramedRead::new(read_half, NsqDecoder::new());
        let mut framed_write = FramedWrite::new(write_half, NsqEncoder);
        
        // Send IDENTIFY command
        let identify_data = serde_json::json!({
            "client_id": "to_nsq",
            "hostname": "to_nsq",
            "user_agent": "to_nsq/1.0",
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
        
        info!("Ready to publish to topic '{}'", self.topic);
        
        // Start publishing task
        let publish_task = tokio::spawn(async move {
            // This will be handled by the main function
        });
        
        Ok(())
    }

    async fn publish_message(&self, framed_write: &mut FramedWrite<tokio::net::tcp::OwnedWriteHalf, NsqEncoder>, content: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        if content.len() > self.max_message_size {
            return Err(format!("Message too large: {} bytes (max: {})", content.len(), self.max_message_size).into());
        }
        
        let mut message_body = content.to_vec();
        
        // Add prefix if specified
        if let Some(prefix) = &self.prefix {
            let mut prefixed = prefix.as_bytes().to_vec();
            prefixed.extend_from_slice(&message_body);
            message_body = prefixed;
        }
        
        // Add timestamp if specified
        if self.add_timestamp {
            let timestamp = chrono::Utc::now().to_rfc3339();
            let mut timestamped = format!("[{}] ", timestamp).as_bytes().to_vec();
            timestamped.extend_from_slice(&message_body);
            message_body = timestamped;
        }
        
        let pub_cmd = Command::Pub {
            topic: self.topic.clone(),
            body: bytes::Bytes::from(message_body),
        };
        let pub_frame = Frame::new(FrameType::Response, pub_cmd.to_bytes()?);
        framed_write.send(pub_frame).await?;
        
        Ok(())
    }

    async fn publish_batch(&self, framed_write: &mut FramedWrite<tokio::net::tcp::OwnedWriteHalf, NsqEncoder>, messages: &[Vec<u8>]) -> Result<(), Box<dyn std::error::Error>> {
        if messages.is_empty() {
            return Ok(());
        }
        
        // Validate all messages
        for msg in messages {
            if msg.len() > self.max_message_size {
                return Err(format!("Message too large: {} bytes (max: {})", msg.len(), self.max_message_size).into());
            }
        }
        
        if messages.len() == 1 {
            // Single message
            self.publish_message(framed_write, &messages[0]).await?;
        } else {
            // Batch messages
            let mut bodies = Vec::new();
            for msg in messages {
                let mut message_body = msg.clone();
                
                // Add prefix if specified
                if let Some(prefix) = &self.prefix {
                    let mut prefixed = prefix.as_bytes().to_vec();
                    prefixed.extend_from_slice(&message_body);
                    message_body = prefixed;
                }
                
                // Add timestamp if specified
                if self.add_timestamp {
                    let timestamp = chrono::Utc::now().to_rfc3339();
                    let mut timestamped = format!("[{}] ", timestamp).as_bytes().to_vec();
                    timestamped.extend_from_slice(&message_body);
                    message_body = timestamped;
                }
                
                bodies.push(bytes::Bytes::from(message_body));
            }
            
            let mpub_cmd = Command::Mpub {
                topic: self.topic.clone(),
                bodies,
            };
            let mpub_frame = Frame::new(FrameType::Response, mpub_cmd.to_bytes()?);
            framed_write.send(mpub_frame).await?;
        }
        
        info!("Published batch of {} messages", messages.len());
        Ok(())
    }
}

async fn read_from_stdin(line_by_line: bool) -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
    let mut messages = Vec::new();
    
    if line_by_line {
        let stdin = stdin();
        let reader = BufReader::new(stdin);
        let mut lines = reader.lines();
        
        while let Some(line) = lines.next_line().await? {
            messages.push(line.into_bytes());
        }
    } else {
        let mut stdin = stdin();
        let mut buffer = Vec::new();
        stdin.read_to_end(&mut buffer).await?;
        messages.push(buffer);
    }
    
    Ok(messages)
}

async fn read_from_file(file_path: &str, line_by_line: bool) -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
    let mut messages = Vec::new();
    let file = File::open(file_path).await?;
    let reader = BufReader::new(file);
    
    if line_by_line {
        let mut lines = reader.lines();
        while let Some(line) = lines.next_line().await? {
            messages.push(line.into_bytes());
        }
    } else {
        let mut file = File::open(file_path).await?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await?;
        messages.push(buffer);
    }
    
    Ok(messages)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    let args = Args::parse();
    
    let producer = NsqProducer::new(
        args.topic,
        args.batch_size,
        args.delay_ms,
        args.max_message_size,
        args.add_timestamp,
        args.prefix,
    );
    
    // Connect to NSQd
    let stream = TcpStream::connect(&args.nsqd_tcp_address).await?;
    let (read_half, write_half) = stream.into_split();
    
    let mut framed_read = FramedRead::new(read_half, NsqDecoder::new());
    let mut framed_write = FramedWrite::new(write_half, NsqEncoder);
    
    // Send IDENTIFY command
    let identify_data = serde_json::json!({
        "client_id": "to_nsq",
        "hostname": "to_nsq",
        "user_agent": "to_nsq/1.0",
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
    
    info!("Ready to publish to topic '{}'", args.topic);
    
    // Read input data
    let messages = if let Some(input_file) = &args.input_file {
        read_from_file(input_file, args.line_by_line).await?
    } else {
        read_from_stdin(args.line_by_line).await?
    };
    
    if messages.is_empty() {
        warn!("No data to publish");
        return Ok(());
    }
    
    info!("Read {} messages from input", messages.len());
    
    // Publish messages in batches
    let mut batch = Vec::new();
    for message in messages {
        batch.push(message);
        
        if batch.len() >= args.batch_size {
            producer.publish_batch(&mut framed_write, &batch).await?;
            batch.clear();
            
            // Add delay between batches
            if args.delay_ms > 0 {
                tokio::time::sleep(tokio::time::Duration::from_millis(args.delay_ms)).await;
            }
        }
    }
    
    // Publish remaining messages
    if !batch.is_empty() {
        producer.publish_batch(&mut framed_write, &batch).await?;
    }
    
    info!("Finished publishing all messages");
    
    Ok(())
}

