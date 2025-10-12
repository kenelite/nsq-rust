//! nsq_tail - Tail NSQ topics like tail -f

use clap::Parser;

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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    println!("nsq_tail - Not yet implemented");
    println!("Would tail topic: {}", args.topic);
    println!("Channel: {}", args.channel);
    
    Ok(())
}

