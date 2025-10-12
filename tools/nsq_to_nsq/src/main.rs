//! nsq_to_nsq - Topic/channel replication tool

use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "nsq_to_nsq")]
#[command(about = "NSQ topic/channel replication tool")]
struct Args {
    /// Source NSQd TCP addresses
    #[arg(long)]
    src_nsqd_tcp_address: Vec<String>,
    
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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    println!("nsq_to_nsq - Not yet implemented");
    println!("Would replicate from {} to {}", args.src_topic, args.dst_topic);
    
    Ok(())
}

