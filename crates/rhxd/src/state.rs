//! Server state (stub for now)

use crate::Config;
use anyhow::Result;
use dashmap::DashMap;
use std::sync::atomic::AtomicU32;

pub struct ServerState {
    pub config: Config,
    pub sessions: DashMap<u32, ()>, // TODO: Implement Session type
    pub next_user_id: AtomicU32,
}

impl ServerState {
    pub async fn new(config: Config) -> Result<Self> {
        // TODO: Initialize database connection
        
        Ok(Self {
            config,
            sessions: DashMap::new(),
            next_user_id: AtomicU32::new(1),
        })
    }
}
