//! NSQ to File - Consumer that writes messages to files

use clap::Parser;

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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    println!("nsq_to_file - Not yet implemented");
    println!("Would consume from topic: {}", args.topic);
    println!("Channel: {}", args.channel);
    println!("Output directory: {}", args.output_dir);
    
    Ok(())
}

