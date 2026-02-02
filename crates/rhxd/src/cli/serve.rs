//! Server serve command

use crate::console;
use crate::{Config, Server};
use anyhow::Result;

pub async fn run(config_path: &str) -> Result<()> {
    // Load configuration
    let config = Config::load(config_path)?;
    
    tracing::info!("Starting rhxd server");
    tracing::info!("Server name: {}", config.server.name);
    tracing::info!("Listening on: {}:{}", config.server.address, config.server.port);
    
    // Create server
    let server = Server::new(config).await?;
    
    // Get state and shutdown handle for console
    let state = server.state();
    let shutdown = server.shutdown_handle();
    
    // Spawn server in background task
    let server_handle = tokio::spawn(async move {
        if let Err(e) = server.run().await {
            tracing::error!("Server error: {}", e);
        }
    });
    
    // Run console in main task
    let console_handle = tokio::spawn(async move {
        if let Err(e) = console::run_console(state).await {
            tracing::error!("Console error: {}", e);
        }
        // When console exits, trigger server shutdown
        shutdown.notify_waiters();
    });
    
    // Wait for both tasks
    tokio::select! {
        _ = server_handle => {
            tracing::info!("Server task completed");
        }
        _ = console_handle => {
            tracing::info!("Console task completed");
        }
    }
    
    Ok(())
}
