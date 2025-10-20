//! nsq_stat - Display NSQ statistics

use clap::Parser;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};
use url::Url;

#[derive(Parser, Debug)]
#[command(name = "nsq_stat")]
#[command(about = "Display NSQ statistics")]
struct Args {
    /// NSQd HTTP addresses
    #[arg(long)]
    nsqd_http_address: Vec<String>,
    
    /// Lookupd HTTP addresses
    #[arg(long)]
    lookupd_http_address: Vec<String>,
    
    /// Refresh interval in seconds
    #[arg(long, default_value = "1")]
    interval: u64,
    
    /// Show detailed topic/channel information
    #[arg(long)]
    detailed: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct NsqdStats {
    version: String,
    health: String,
    start_time: u64,
    topics: Vec<TopicStats>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TopicStats {
    topic_name: String,
    channels: Vec<ChannelStats>,
    depth: u64,
    backend_depth: u64,
    message_count: u64,
    paused: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChannelStats {
    channel_name: String,
    depth: u64,
    backend_depth: u64,
    inflight_count: u64,
    deferred_count: u64,
    message_count: u64,
    requeue_count: u64,
    timeout_count: u64,
    clients: Vec<ClientStats>,
    paused: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct ClientStats {
    name: String,
    client_id: String,
    hostname: String,
    version: String,
    tcp_port: u16,
    http_port: u16,
    state: u32,
    ready_count: u64,
    in_flight_count: u64,
    message_count: u64,
    finish_count: u64,
    requeue_count: u64,
    connect_ts: u64,
    sample_rate: u32,
    deflate: bool,
    snappy: bool,
    user_agent: String,
    tls_version: String,
    tls_cipher_suite: String,
    tls_negotiated_protocol: String,
    tls_negotiated_protocol_is_mutual: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct LookupdStats {
    producers: Vec<ProducerStats>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProducerStats {
    remote_address: String,
    hostname: String,
    broadcast_address: String,
    tcp_port: u16,
    http_port: u16,
    version: String,
    last_update: u64,
    tombstoned: bool,
    topics: Vec<String>,
}

struct StatsCollector {
    client: Client,
    nsqd_addresses: Vec<String>,
    lookupd_addresses: Vec<String>,
}

impl StatsCollector {
    fn new(nsqd_addresses: Vec<String>, lookupd_addresses: Vec<String>) -> Self {
        Self {
            client: Client::new(),
            nsqd_addresses,
            lookupd_addresses,
        }
    }

    async fn collect_nsqd_stats(&self) -> Vec<NsqdStats> {
        let mut all_stats = Vec::new();
        
        for address in &self.nsqd_addresses {
            match self.fetch_nsqd_stats(address).await {
                Ok(stats) => all_stats.push(stats),
                Err(e) => {
                    error!("Failed to fetch stats from {}: {}", address, e);
                }
            }
        }
        
        all_stats
    }

    async fn fetch_nsqd_stats(&self, address: &str) -> Result<NsqdStats, Box<dyn std::error::Error>> {
        let url = format!("http://{}/stats?format=json", address);
        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            return Err(format!("HTTP error: {}", response.status()).into());
        }
        
        let stats: NsqdStats = response.json().await?;
        Ok(stats)
    }

    async fn collect_lookupd_stats(&self) -> Vec<LookupdStats> {
        let mut all_stats = Vec::new();
        
        for address in &self.lookupd_addresses {
            match self.fetch_lookupd_stats(address).await {
                Ok(stats) => all_stats.push(stats),
                Err(e) => {
                    error!("Failed to fetch lookupd stats from {}: {}", address, e);
                }
            }
        }
        
        all_stats
    }

    async fn fetch_lookupd_stats(&self, address: &str) -> Result<LookupdStats, Box<dyn std::error::Error>> {
        let url = format!("http://{}/nodes", address);
        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            return Err(format!("HTTP error: {}", response.status()).into());
        }
        
        let stats: LookupdStats = response.json().await?;
        Ok(stats)
    }
}

fn print_stats(nsqd_stats: &[NsqdStats], lookupd_stats: &[LookupdStats], detailed: bool) {
    println!("\n=== NSQ Statistics ===");
    
    // Print NSQd stats
    for (i, stats) in nsqd_stats.iter().enumerate() {
        println!("\nNSQd #{}:", i + 1);
        println!("  Version: {}", stats.version);
        println!("  Health: {}", stats.health);
        println!("  Start Time: {}", stats.start_time);
        println!("  Topics: {}", stats.topics.len());
        
        if detailed {
            for topic in &stats.topics {
                println!("    Topic: {}", topic.topic_name);
                println!("      Depth: {}", topic.depth);
                println!("      Backend Depth: {}", topic.backend_depth);
                println!("      Message Count: {}", topic.message_count);
                println!("      Paused: {}", topic.paused);
                println!("      Channels: {}", topic.channels.len());
                
                for channel in &topic.channels {
                    println!("        Channel: {}", channel.channel_name);
                    println!("          Depth: {}", channel.depth);
                    println!("          Backend Depth: {}", channel.backend_depth);
                    println!("          Inflight: {}", channel.inflight_count);
                    println!("          Deferred: {}", channel.deferred_count);
                    println!("          Message Count: {}", channel.message_count);
                    println!("          Requeue Count: {}", channel.requeue_count);
                    println!("          Timeout Count: {}", channel.timeout_count);
                    println!("          Paused: {}", channel.paused);
                    println!("          Clients: {}", channel.clients.len());
                    
                    for client in &channel.clients {
                        println!("            Client: {} ({})", client.name, client.client_id);
                        println!("              Hostname: {}", client.hostname);
                        println!("              Version: {}", client.version);
                        println!("              TCP Port: {}", client.tcp_port);
                        println!("              HTTP Port: {}", client.http_port);
                        println!("              State: {}", client.state);
                        println!("              Ready Count: {}", client.ready_count);
                        println!("              In Flight: {}", client.in_flight_count);
                        println!("              Message Count: {}", client.message_count);
                        println!("              Finish Count: {}", client.finish_count);
                        println!("              Requeue Count: {}", client.requeue_count);
                        println!("              Connect Time: {}", client.connect_ts);
                        println!("              Sample Rate: {}", client.sample_rate);
                        println!("              User Agent: {}", client.user_agent);
                    }
                }
            }
        } else {
            // Summary view
            let total_depth: u64 = stats.topics.iter().map(|t| t.depth).sum();
            let total_backend_depth: u64 = stats.topics.iter().map(|t| t.backend_depth).sum();
            let total_messages: u64 = stats.topics.iter().map(|t| t.message_count).sum();
            let total_channels: usize = stats.topics.iter().map(|t| t.channels.len()).sum();
            
            println!("  Total Depth: {}", total_depth);
            println!("  Total Backend Depth: {}", total_backend_depth);
            println!("  Total Messages: {}", total_messages);
            println!("  Total Channels: {}", total_channels);
        }
    }
    
    // Print Lookupd stats
    for (i, stats) in lookupd_stats.iter().enumerate() {
        println!("\nLookupd #{}:", i + 1);
        println!("  Producers: {}", stats.producers.len());
        
        if detailed {
            for producer in &stats.producers {
                println!("    Producer: {}", producer.hostname);
                println!("      Remote Address: {}", producer.remote_address);
                println!("      Broadcast Address: {}", producer.broadcast_address);
                println!("      TCP Port: {}", producer.tcp_port);
                println!("      HTTP Port: {}", producer.http_port);
                println!("      Version: {}", producer.version);
                println!("      Last Update: {}", producer.last_update);
                println!("      Tombstoned: {}", producer.tombstoned);
                println!("      Topics: {}", producer.topics.join(", "));
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    let args = Args::parse();
    
    if args.nsqd_http_address.is_empty() && args.lookupd_http_address.is_empty() {
        eprintln!("Error: At least one NSQd HTTP address or Lookupd HTTP address must be specified");
        std::process::exit(1);
    }
    
    let collector = StatsCollector::new(args.nsqd_http_address, args.lookupd_http_address);
    
    loop {
        // Clear screen (works on most terminals)
        print!("\x1B[2J\x1B[1;1H");
        
        let nsqd_stats = collector.collect_nsqd_stats().await;
        let lookupd_stats = collector.collect_lookupd_stats().await;
        
        print_stats(&nsqd_stats, &lookupd_stats, args.detailed);
        
        println!("\nPress Ctrl+C to exit");
        println!("Refreshing in {} seconds...", args.interval);
        
        sleep(Duration::from_secs(args.interval)).await;
    }
}

