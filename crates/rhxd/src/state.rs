//! Server state management

use crate::connection::Session;
use crate::db::Database;
use crate::Config;
use anyhow::Result;
use dashmap::DashMap;
use std::sync::atomic::{AtomicU16, Ordering};
use tokio::sync::broadcast;

/// Message types that can be broadcast to all connected sessions
#[derive(Debug, Clone)]
pub enum BroadcastMessage {
    /// User joined the server
    UserJoined { user_id: u16, nickname: String },
    /// User left the server
    UserLeft { user_id: u16 },
    /// Server is shutting down
    ServerShutdown,
    /// Server message/announcement
    ServerMessage { message: String },
    /// Chat message to broadcast to all users
    ChatMessage { sender_id: u16, message: Vec<u8> },
}

/// Shared server state accessible by all connection handlers
pub struct ServerState {
    /// Server configuration
    pub config: Config,
    
    /// Database connection pool
    pub database: Database,
    
    /// Active sessions indexed by user_id (1-65535)
    pub sessions: DashMap<u16, Session>,
    
    /// Next available user ID (wraps at 65535, skips 0)
    next_user_id: AtomicU16,
    
    /// Broadcast channel for server-wide messages
    pub broadcast_tx: broadcast::Sender<BroadcastMessage>,
}

impl ServerState {
    /// Create a new server state instance
    pub async fn new(config: Config) -> Result<Self> {
        // Initialize database connection
        let database = Database::new(&config.database.path).await?;
        
        // Initialize schema
        database.init_schema().await?;
        
        // Health check
        database.health_check().await?;
        
        // Create broadcast channel (buffer 100 messages)
        let (broadcast_tx, _) = broadcast::channel(100);
        
        Ok(Self {
            config,
            database,
            sessions: DashMap::new(),
            next_user_id: AtomicU16::new(1),
            broadcast_tx,
        })
    }
    
    /// Allocate the next available user ID (1-65535, wrapping)
    pub fn allocate_user_id(&self) -> u16 {
        loop {
            let id = self.next_user_id.fetch_add(1, Ordering::Relaxed);
            
            // Wrap at 65535, skip 0 (invalid)
            let id = if id == 0 || id == u16::MAX {
                self.next_user_id.store(1, Ordering::Relaxed);
                1
            } else {
                id
            };
            
            // Ensure the ID isn't currently in use
            if !self.sessions.contains_key(&id) {
                return id;
            }
        }
    }
    
    /// Register a new session
    pub fn register_session(&self, session: Session) {
        self.sessions.insert(session.user_id, session);
    }
    
    /// Unregister a session by user ID
    pub fn unregister_session(&self, user_id: u16) -> Option<Session> {
        self.sessions.remove(&user_id).map(|(_, session)| session)
    }
    
    /// Get a session by user ID
    pub fn get_session(&self, user_id: u16) -> Option<dashmap::mapref::one::Ref<u16, Session>> {
        self.sessions.get(&user_id)
    }
    
    /// Get a mutable reference to a session by user ID
    pub fn get_session_mut(&self, user_id: u16) -> Option<dashmap::mapref::one::RefMut<'_, u16, Session>> {
        self.sessions.get_mut(&user_id)
    }
    
    /// Broadcast a message to all connected clients
    pub fn broadcast(&self, message: BroadcastMessage) {
        // Ignore send errors (no receivers is fine)
        let _ = self.broadcast_tx.send(message);
    }
    
    /// Get the number of active sessions
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }
}
