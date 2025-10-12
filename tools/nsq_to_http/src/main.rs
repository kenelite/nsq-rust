//! nsq_to_http - Consumer that posts messages to HTTP endpoints

use clap::Parser;

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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    println!("nsq_to_http - Not yet implemented");
    println!("Would consume from topic: {}", args.topic);
    println!("Channel: {}", args.channel);
    println!("HTTP endpoint: {}", args.http_endpoint);
    
    Ok(())
}

