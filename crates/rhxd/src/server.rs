//! Server implementation

use crate::connection::handler::handle_connection;
use crate::state::BroadcastMessage;
use crate::{Config, ServerState};
use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Notify;

pub struct Server {
    state: Arc<ServerState>,
    shutdown: Arc<Notify>,
}

impl Server {
    /// Create a new server instance
    pub async fn new(config: Config) -> Result<Self> {
        let state = ServerState::new(config).await?;
        Ok(Self {
            state: Arc::new(state),
            shutdown: Arc::new(Notify::new()),
        })
    }
    
    /// Run the server main loop
    pub async fn run(self) -> Result<()> {
        let addr = format!(
            "{}:{}",
            self.state.config.server.address,
            self.state.config.server.port
        );
        
        // Bind TCP listener
        let listener = TcpListener::bind(&addr)
            .await
            .context(format!("Failed to bind to {}", addr))?;
        
        tracing::info!(
            "Server '{}' listening on {}",
            self.state.config.server.name,
            addr
        );
        
        // Spawn signal handler for graceful shutdown
        let shutdown = self.shutdown.clone();
        tokio::spawn(async move {
            if let Err(e) = tokio::signal::ctrl_c().await {
                tracing::error!("Failed to listen for shutdown signal: {}", e);
            } else {
                tracing::info!("Received shutdown signal");
            }
            shutdown.notify_waiters();
        });
        
        // Main accept loop
        loop {
            tokio::select! {
                // Wait for shutdown signal
                _ = self.shutdown.notified() => {
                    tracing::info!("Shutting down server...");
                    break;
                }
                
                // Accept new connections
                result = listener.accept() => {
                    match result {
                        Ok((stream, addr)) => {
                            // Check connection limit
                            if self.state.session_count() >= self.state.config.server.max_connections {
                                tracing::warn!("Connection limit reached, rejecting connection from {}", addr);
                                drop(stream);
                                continue;
                            }
                            
                            let state = self.state.clone();
                            
                            // Spawn connection handler
                            tokio::spawn(async move {
                                if let Err(e) = handle_connection(stream, state).await {
                                    tracing::error!("Connection handler error: {}", e);
                                }
                            });
                        }
                        Err(e) => {
                            tracing::error!("Failed to accept connection: {}", e);
                        }
                    }
                }
            }
        }
        
        // Broadcast shutdown message to all clients
        self.state.broadcast(BroadcastMessage::ServerShutdown);
        
        // Give clients a moment to disconnect gracefully
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        tracing::info!(
            "Server shutdown complete ({} active sessions)",
            self.state.session_count()
        );
        
        Ok(())
    }
}
