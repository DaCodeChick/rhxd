//! Chat transaction handlers

use crate::state::{BroadcastMessage, ServerState};
use anyhow::{Context, Result};
use rhxcore::protocol::{FieldId, Transaction};
use rhxcore::types::ChatOptions;
use std::sync::Arc;

/// Handle SendChat transaction (105)
///
/// Client sends:
/// - Field 101: Message data (binary)
/// - Field 109: Chat options (optional, 0=normal, 1=emote)
///
/// Server broadcasts ChatMessage (106) to all connected users:
/// - Field 101: Message data
/// - Field 103: Sender user ID
/// - Field 102: Sender nickname
pub async fn handle_send_chat(
    transaction: Transaction,
    user_id: u16,
    state: Arc<ServerState>,
) -> Result<Option<Transaction>> {
    // Check if user is authenticated
    let sender_info = {
        let session = state.get_session(user_id)
            .context("Session not found")?;
        
        if !session.is_authenticated() {
            tracing::warn!("User {} tried to send chat without authentication", user_id);
            return Ok(None);
        }
        
        (session.user_id, session.nickname.clone())
    };
    
    // Extract message data and chat options
    let mut message_data: Option<Vec<u8>> = None;
    let mut chat_options = ChatOptions::NORMAL;
    
    for field in &transaction.fields {
        match field.id {
            FieldId::Data => {
                message_data = field.as_binary().map(|b| b.to_vec());
            }
            FieldId::ChatOptions => {
                if let Some(value) = field.as_integer() {
                    chat_options = ChatOptions::from_i16(value as i16);
                }
            }
            _ => {}
        }
    }
    
    let message_data = message_data.context("Missing message data")?;
    
    // Convert to string for logging
    let message_text = String::from_utf8_lossy(&message_data);
    
    let chat_type = if chat_options.is_emote() {
        "emote"
    } else {
        "normal"
    };
    
    tracing::info!(
        "User {} ({}) sent {} chat: {}",
        sender_info.0,
        sender_info.1,
        chat_type,
        message_text.chars().take(50).collect::<String>()
    );
    
    // Broadcast chat message to all connected users
    state.broadcast(BroadcastMessage::ChatMessage {
        sender_id: sender_info.0,
        message: message_data,
        chat_options,
    });
    
    // No direct reply to sender (broadcast is the response)
    Ok(None)
}
