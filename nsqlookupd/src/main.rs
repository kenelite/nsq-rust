//! NSQLookupd main entry point

use nsqlookupd::{config::Args, server::NsqlookupdServer};
use nsq_common::init_logging;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args = Args::parse();
    
    // Convert to configuration
    let config: nsq_common::NsqlookupdConfig = args.into();
    
    // Initialize logging
    init_logging(&config.base)?;
    
    // Create and start server
    let mut server = NsqlookupdServer::new(config)?;
    server.start().await?;
    
    // Keep the main thread alive
    tokio::signal::ctrl_c().await?;
    
    Ok(())
}

