//! nsq_stat - Display NSQ statistics

use clap::Parser;

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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    println!("nsq_stat - Not yet implemented");
    println!("Would display stats from:");
    for addr in &args.nsqd_http_address {
        println!("  NSQd: {}", addr);
    }
    for addr in &args.lookupd_http_address {
        println!("  Lookupd: {}", addr);
    }
    
    Ok(())
}

