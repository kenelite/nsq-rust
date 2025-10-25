//! NSQLookupd main entry point

use nsqlookupd::{config::Args, server::NsqlookupdServer};
use nsq_common::init_logging;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args = Args::parse();
    
    // Validate configuration
    if let Err(e) = args.validate() {
        eprintln!("Configuration error: {}", e);
        std::process::exit(1);
    }
    
    // Convert to configuration
    let config: nsq_common::NsqlookupdConfig = args.into();
    
    // Initialize logging
    init_logging(&config.base)?;
    
    tracing::info!("Starting NSQLookupd {}", env!("CARGO_PKG_VERSION"));
    tracing::info!("TCP address: {}", config.tcp_address);
    tracing::info!("HTTP address: {}", config.http_address);
    tracing::info!("Inactive producer timeout: {}ms", config.inactive_producer_timeout);
    tracing::info!("Tombstone lifetime: {}ms", config.tombstone_lifetime);
    
    // Create and start server
    let mut server = NsqlookupdServer::new(config)?;
    
    // Handle graceful shutdown
    let server_handle = tokio::spawn(async move {
        if let Err(e) = server.start().await {
            tracing::error!("Server error: {}", e);
        }
    });
    
    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    tracing::info!("Shutting down NSQLookupd...");
    
    // Cancel server task
    server_handle.abort();
    
    tracing::info!("NSQLookupd stopped");
    Ok(())
}

