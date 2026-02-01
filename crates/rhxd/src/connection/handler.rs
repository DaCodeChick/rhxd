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
    
    // Subscribe to broadcast messages
    let mut broadcast_rx = state.broadcast_tx.subscribe();
    
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
            
            // TODO: Handle timeouts/keepalive
            
            // Handle broadcast messages
            msg = broadcast_rx.recv() => {
                match msg {
                    Ok(broadcast) => {
                        // Convert broadcast to transaction if needed
                        let transaction = match broadcast {
                            BroadcastMessage::ChatMessage { sender_id, message } => {
                                // Get sender nickname
                                let sender_nickname = state.get_session(sender_id)
                                    .map(|s| s.nickname.clone())
                                    .unwrap_or_else(|| format!("User {}", sender_id));
                                
                                Some(Transaction {
                                    flags: 0,
                                    is_reply: false,
                                    transaction_type: TransactionType::ChatMessage,
                                    id: 0, // Server-initiated transaction
                                    error_code: 0,
                                    total_size: 0,
                                    data_size: 0,
                                    fields: vec![
                                        rhxcore::protocol::Field::binary(rhxcore::protocol::FieldId::Data, message),
                                        rhxcore::protocol::Field::integer(rhxcore::protocol::FieldId::UserId, sender_id as i32),
                                        rhxcore::protocol::Field::string(rhxcore::protocol::FieldId::UserName, sender_nickname),
                                    ],
                                })
                            }
                            BroadcastMessage::UserJoined { user_id: joined_user_id, nickname } => {
                                // Get user info from session
                                let (icon_id, flags) = state.get_session(joined_user_id)
                                    .map(|s| (s.icon_id, s.flags))
                                    .unwrap_or((0, 0));
                                
                                // Build UserNameWithInfo field (Field 300)
                                // Format: user_id (2 bytes) + icon_id (2 bytes) + flags (2 bytes) + name_len (2 bytes) + name
                                let mut user_info = Vec::new();
                                user_info.extend_from_slice(&joined_user_id.to_be_bytes());
                                user_info.extend_from_slice(&icon_id.to_be_bytes());
                                user_info.extend_from_slice(&flags.to_be_bytes());
                                user_info.extend_from_slice(&(nickname.len() as u16).to_be_bytes());
                                user_info.extend_from_slice(nickname.as_bytes());
                                
                                Some(Transaction {
                                    flags: 0,
                                    is_reply: false,
                                    transaction_type: TransactionType::NotifyChangeUser,
                                    id: 0, // Server-initiated transaction
                                    error_code: 0,
                                    total_size: 0,
                                    data_size: 0,
                                    fields: vec![
                                        rhxcore::protocol::Field::binary(rhxcore::protocol::FieldId::UserNameWithInfo, user_info),
                                    ],
                                })
                            }
                            BroadcastMessage::UserLeft { user_id: left_user_id } => {
                                Some(Transaction {
                                    flags: 0,
                                    is_reply: false,
                                    transaction_type: TransactionType::NotifyDeleteUser,
                                    id: 0, // Server-initiated transaction
                                    error_code: 0,
                                    total_size: 0,
                                    data_size: 0,
                                    fields: vec![
                                        rhxcore::protocol::Field::integer(rhxcore::protocol::FieldId::UserId, left_user_id as i32),
                                    ],
                                })
                            }
                            BroadcastMessage::ServerShutdown => {
                                tracing::info!("User {} notified of server shutdown", user_id);
                                break;
                            }
                            BroadcastMessage::ServerMessage { message } => {
                                Some(Transaction {
                                    flags: 0,
                                    is_reply: false,
                                    transaction_type: TransactionType::ServerMessage,
                                    id: 0, // Server-initiated transaction
                                    error_code: 0,
                                    total_size: 0,
                                    data_size: 0,
                                    fields: vec![
                                        rhxcore::protocol::Field::string(rhxcore::protocol::FieldId::Data, message),
                                    ],
                                })
                            }
                        };
                        
                        // Send transaction if we created one
                        if let Some(tx) = transaction {
                            if let Err(e) = framed.send(tx).await {
                                tracing::error!("Failed to send broadcast to user {}: {}", user_id, e);
                                break;
                            }
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                        tracing::warn!("User {} lagged behind, skipped {} broadcasts", user_id, skipped);
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        tracing::info!("Broadcast channel closed for user {}", user_id);
                        break;
                    }
                }
            }
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
        
        TransactionType::Agreed => {
            let result = handlers::agreed::handle_agreed(transaction, user_id, state).await?;
            Ok(result)
        }
        
        TransactionType::SendChat => {
            let result = handlers::chat::handle_send_chat(transaction, user_id, state).await?;
            Ok(result)
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
