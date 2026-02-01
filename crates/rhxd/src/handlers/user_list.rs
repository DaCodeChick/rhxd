//! User list transaction handler

use crate::state::ServerState;
use anyhow::Result;
use rhxcore::protocol::{ErrorCode, Field, FieldId, Transaction, TransactionType};
use std::sync::Arc;

/// Handle GetUserNameList transaction (300)
///
/// Client requests list of all connected users.
///
/// Server replies with:
/// - Multiple Field 300 (UserNameWithInfo) entries, one per connected user
///
/// UserNameWithInfo format (binary):
/// - user_id: u16 (2 bytes, big-endian)
/// - icon_id: u16 (2 bytes, big-endian)
/// - flags: u16 (2 bytes, big-endian)
/// - name_len: u16 (2 bytes, big-endian)
/// - name: [u8] (variable length)
pub async fn handle_get_user_name_list(
    transaction: Transaction,
    user_id: u16,
    state: Arc<ServerState>,
) -> Result<Option<Transaction>> {
    tracing::debug!("User {} requested user list", user_id);
    
    // Check if user is authenticated
    let session_exists = state.get_session(user_id).is_some();
    if !session_exists {
        tracing::warn!("User {} requested user list but session not found", user_id);
        return Ok(None);
    }
    
    // Build list of all authenticated users
    let mut user_fields = Vec::new();
    
    for entry in state.sessions.iter() {
        let session = entry.value();
        
        // Only include authenticated users
        if !session.is_authenticated() {
            continue;
        }
        
        // Build UserNameWithInfo field
        let mut user_info = Vec::new();
        user_info.extend_from_slice(&session.user_id.to_be_bytes());
        user_info.extend_from_slice(&session.icon_id.to_be_bytes());
        user_info.extend_from_slice(&session.flags.to_be_bytes());
        user_info.extend_from_slice(&(session.nickname.len() as u16).to_be_bytes());
        user_info.extend_from_slice(session.nickname.as_bytes());
        
        user_fields.push(Field::binary(FieldId::UserNameWithInfo, user_info));
    }
    
    tracing::info!(
        "User {} requested user list, returning {} users",
        user_id,
        user_fields.len()
    );
    
    // Send reply with user list
    Ok(Some(Transaction {
        flags: 0,
        is_reply: true,
        transaction_type: TransactionType::GetUserNameList,
        id: transaction.id,
        error_code: ErrorCode::NoError.to_u32(),
        total_size: 0,
        data_size: 0,
        fields: user_fields,
    }))
}
