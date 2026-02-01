//! Connection handler for individual clients

use crate::connection::Session;
use crate::handlers;
use crate::state::{BroadcastMessage, ServerState};
use anyhow::{Context, Result};
use bytes::BytesMut;
use rhxcore::codec::TransactionCodec;
use rhxcore::protocol::{Handshake, HandshakeReply, Transaction, TransactionType};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

/// Handle an incoming client connection
pub async fn handle_connection(
    mut stream: TcpStream,
    state: Arc<ServerState>,
) -> Result<()> {
    let peer_addr = stream.peer_addr()?;
    
    // Allocate a user ID for this connection
    let user_id = state.allocate_user_id();
    
    tracing::info!("Connection from {} assigned user_id={}", peer_addr, user_id);
    
    // Create session
    let session = Session::new(user_id, peer_addr);
    state.register_session(session.clone());
    
    // Perform handshake
    match perform_handshake(&mut stream, user_id).await {
        Ok(_) => {
            // Update session state to LoginPending
            if let Some(mut session) = state.get_session_mut(user_id) {
                session.complete_handshake();
                tracing::info!("User {} completed handshake", user_id);
            }
        }
        Err(e) => {
            tracing::warn!("Handshake failed for user {}: {}", user_id, e);
            // Cleanup and return
            state.unregister_session(user_id);
            return Err(e);
        }
    }
    
    // Create framed codec for transaction handling
    let mut framed = Framed::new(stream, TransactionCodec::new());
    
    // Main transaction loop
    use futures::StreamExt;
    use futures::SinkExt;
    
    loop {
        tokio::select! {
            // Read transaction from client
            result = framed.next() => {
                match result {
                    Some(Ok(transaction)) => {
                        // Update session activity
                        if let Some(mut session) = state.get_session_mut(user_id) {
                            session.touch();
                        }
                        
                        tracing::debug!(
                            "User {} transaction: type={:?}, id={}, fields={}",
                            user_id,
                            transaction.transaction_type,
                            transaction.id,
                            transaction.fields.len()
                        );
                        
                        // Dispatch to appropriate handler
                        let reply = handle_transaction(transaction, user_id, state.clone()).await;
                        
                        match reply {
                            Ok(Some(reply_transaction)) => {
                                // Send reply
                                if let Err(e) = framed.send(reply_transaction).await {
                                    tracing::error!("Failed to send reply to user {}: {}", user_id, e);
                                    break;
                                }
                            }
                            Ok(None) => {
                                // No reply needed (transaction handled)
                            }
                            Err(e) => {
                                tracing::error!("Error handling transaction for user {}: {}", user_id, e);
                                // Continue processing (don't disconnect on handler errors)
                            }
                        }
                    }
                    Some(Err(e)) => {
                        tracing::warn!("Error reading transaction from user {}: {}", user_id, e);
                        break;
                    }
                    None => {
                        tracing::debug!("User {} connection closed", user_id);
                        break;
                    }
                }
            }
            
            // TODO: Handle broadcast messages
            // TODO: Handle timeouts/keepalive
        }
    }
    
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

/// Perform the TRTP handshake with a client
async fn perform_handshake(stream: &mut TcpStream, user_id: u16) -> Result<()> {
    // Read handshake from client (12 bytes)
    let mut buf = [0u8; Handshake::SIZE];
    stream
        .read_exact(&mut buf)
        .await
        .context("Failed to read handshake from client")?;
    
    // Parse handshake
    let handshake = Handshake::from_bytes(&buf)
        .context("Failed to parse handshake")?;
    
    tracing::debug!(
        "User {} handshake: protocol={:?}, sub_protocol={}, version={}, sub_version={}",
        user_id,
        std::str::from_utf8(&handshake.protocol_id).unwrap_or("???"),
        handshake.sub_protocol_id,
        handshake.version,
        handshake.sub_version
    );
    
    // Validate protocol magic
    if !handshake.is_valid() {
        tracing::warn!(
            "User {} sent invalid protocol magic: {:?}",
            user_id,
            handshake.protocol_id
        );
        
        // Send error reply
        let reply = HandshakeReply::error(1); // Error code 1: invalid protocol
        let mut reply_buf = BytesMut::with_capacity(HandshakeReply::SIZE);
        reply.to_bytes(&mut reply_buf);
        stream.write_all(&reply_buf).await?;
        
        return Err(anyhow::anyhow!("Invalid protocol magic"));
    }
    
    // Validate protocol version (we support version 1)
    if handshake.version != 1 {
        tracing::warn!(
            "User {} sent unsupported protocol version: {}",
            user_id,
            handshake.version
        );
        
        // Send error reply
        let reply = HandshakeReply::error(2); // Error code 2: unsupported version
        let mut reply_buf = BytesMut::with_capacity(HandshakeReply::SIZE);
        reply.to_bytes(&mut reply_buf);
        stream.write_all(&reply_buf).await?;
        
        return Err(anyhow::anyhow!(
            "Unsupported protocol version: {}",
            handshake.version
        ));
    }
    
    // Send success reply (8 bytes)
    let reply = HandshakeReply::new();
    let mut reply_buf = BytesMut::with_capacity(HandshakeReply::SIZE);
    reply.to_bytes(&mut reply_buf);
    
    stream
        .write_all(&reply_buf)
        .await
        .context("Failed to send handshake reply")?;
    
    stream.flush().await.context("Failed to flush handshake reply")?;
    
    tracing::debug!("User {} handshake successful", user_id);
    
    Ok(())
}

/// Dispatch transaction to appropriate handler
async fn handle_transaction(
    transaction: Transaction,
    user_id: u16,
    state: Arc<ServerState>,
) -> Result<Option<Transaction>> {
    match transaction.transaction_type {
        TransactionType::Login => {
            let reply = handlers::login::handle_login(transaction, user_id, state).await?;
            Ok(Some(reply))
        }
        
        // TODO: Implement other handlers
        TransactionType::Agreed => {
            tracing::info!("User {} sent Agreed transaction (not yet implemented)", user_id);
            Ok(None)
        }
        
        TransactionType::SendChat => {
            tracing::info!("User {} sent chat message (not yet implemented)", user_id);
            Ok(None)
        }
        
        TransactionType::GetUserNameList => {
            tracing::info!("User {} requested user list (not yet implemented)", user_id);
            Ok(None)
        }
        
        _ => {
            tracing::warn!(
                "User {} sent unhandled transaction type: {:?}",
                user_id,
                transaction.transaction_type
            );
            Ok(None)
        }
    }
}
