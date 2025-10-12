//! to_nsq - Producer that reads from stdin/files

use clap::Parser;

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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    println!("to_nsq - Not yet implemented");
    println!("Would publish to topic: {}", args.topic);
    println!("NSQd address: {}", args.nsqd_tcp_address);
    
    Ok(())
}

