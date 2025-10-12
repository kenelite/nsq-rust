//! NSQAdmin main entry point

use nsqadmin::server::NsqadminServer;
use nsq_common::init_logging;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args = nsqadmin::config::Args::parse();
    
    // Convert to configuration
    let config: nsq_common::NsqadminConfig = args.into();
    
    // Initialize logging
    init_logging(&config.base)?;
    
    // Create and start server
    let server = NsqadminServer::new(config)?;
    server.run().await?;
    
    // Keep the main thread alive
    tokio::signal::ctrl_c().await?;
    
    Ok(())
}
