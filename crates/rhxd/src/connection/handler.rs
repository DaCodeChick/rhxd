//! Connection handler for individual clients

use crate::connection::Session;
use crate::state::{BroadcastMessage, ServerState};
use anyhow::Result;
use rhxcore::codec::TransactionCodec;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

/// Handle an incoming client connection
pub async fn handle_connection(
    stream: TcpStream,
    state: Arc<ServerState>,
) -> Result<()> {
    let peer_addr = stream.peer_addr()?;
    
    // Allocate a user ID for this connection
    let user_id = state.allocate_user_id();
    
    tracing::info!("Connection from {} assigned user_id={}", peer_addr, user_id);
    
    // Create session
    let session = Session::new(user_id, peer_addr);
    state.register_session(session.clone());
    
    // Create framed codec for transaction handling
    let _framed = Framed::new(stream, TransactionCodec::new());
    
    // TODO: Implement handshake (Task #3)
    // - Read handshake bytes from client
    // - Validate protocol version
    // - Send handshake response
    // - Update session state to LoginPending
    
    // TODO: Main transaction loop
    // - Read transactions from framed stream
    // - Dispatch to appropriate handlers based on transaction type
    // - Handle errors and disconnections
    // - Update session activity timestamps
    
    tracing::info!("TODO: Handshake and transaction handling not yet implemented");
    
    // Simulate connection staying open briefly for testing
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    
    // Cleanup on disconnect
    if let Some(session) = state.unregister_session(user_id) {
        tracing::info!(
            "User {} ({}) disconnected",
            session.user_id,
            session.nickname
        );
        
        // Broadcast user left if they were authenticated
        if session.is_authenticated() {
            state.broadcast(BroadcastMessage::UserLeft { user_id });
        }
    }
    
    Ok(())
}
