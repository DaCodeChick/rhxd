//! Server implementation (stub for now)

use crate::{Config, ServerState};
use anyhow::Result;
use std::sync::Arc;

pub struct Server {
    _state: Arc<ServerState>,
}

impl Server {
    pub async fn new(config: Config) -> Result<Self> {
        let state = ServerState::new(config).await?;
        Ok(Self {
            _state: Arc::new(state),
        })
    }
    
    pub async fn run(self) -> Result<()> {
        tracing::info!("Server started");
        
        // TODO: Implement server main loop
        // - TCP listener
        // - Connection handling
        // - Signal handling
        
        tokio::signal::ctrl_c().await?;
        tracing::info!("Shutting down...");
        
        Ok(())
    }
}
