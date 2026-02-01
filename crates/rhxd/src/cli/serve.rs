//! Server serve command

use crate::{Config, Server};
use anyhow::Result;

pub async fn run(config_path: &str) -> Result<()> {
    // Load configuration
    let config = Config::load(config_path)?;
    
    tracing::info!("Starting rhxd server");
    tracing::info!("Server name: {}", config.server.name);
    tracing::info!("Listening on: {}:{}", config.server.address, config.server.port);
    
    // Create and run server
    let server = Server::new(config).await?;
    server.run().await?;
    
    Ok(())
}
