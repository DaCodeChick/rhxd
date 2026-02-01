//! Agreed transaction handler

use crate::state::{BroadcastMessage, ServerState};
use anyhow::Result;
use rhxcore::protocol::{ErrorCode, FieldId, Transaction, TransactionType};
use rhxcore::types::UserFlags;
use std::sync::Arc;

/// Handle Agreed transaction (121)
///
/// Client sends after accepting server agreement:
/// - Field 102: User name (nickname to display)
/// - Field 104: Icon ID
/// - Field 113: Options (user flags)
/// - Field 215: Auto-response (optional)
///
/// Server:
/// 1. Updates the session with user-provided nickname and icon
/// 2. Sends acknowledgment reply
/// 3. Broadcasts NotifyChangeUser (301) to all connected users
pub async fn handle_agreed(
    transaction: Transaction,
    user_id: u16,
    state: Arc<ServerState>,
) -> Result<Option<Transaction>> {
    tracing::debug!("User {} sent agreed transaction", user_id);
    
    // Check if user is authenticated
    let session_exists = state.get_session(user_id).is_some();
    if !session_exists {
        tracing::warn!("User {} sent agreed but session not found", user_id);
        return Ok(None);
    }
    
    // Extract fields
    let mut nickname: Option<String> = None;
    let mut icon_id: Option<i32> = None;
    let mut options: Option<i32> = None;
    
    for field in &transaction.fields {
        match field.id {
            FieldId::UserName => {
                nickname = field.as_string().map(|s| s.to_string());
            }
            FieldId::UserIconId => {
                icon_id = field.as_integer();
            }
            FieldId::Options => {
                options = field.as_integer();
            }
            _ => {}
        }
    }
    
    // Use default values if not provided
    // Handle empty nickname strings
    let nickname = match nickname {
        Some(n) if !n.trim().is_empty() => n,
        _ => format!("Guest {}", user_id),
    };
    let mut icon_id = icon_id.unwrap_or(0) as u16;
    let mut flags = options.unwrap_or(0) as u16;
    
    // Determine access privileges
    let access_privileges = {
        if let Some(session) = state.get_session(user_id) {
            if let Some(account_id) = session.account_id {
                // For authenticated users, fetch from database
                match crate::db::accounts::get_account_by_id(state.database.pool(), account_id).await {
                    Ok(Some(account)) => account.access_privileges(),
                    _ => rhxcore::types::AccessPrivileges::guest(),
                }
            } else {
                // Guest user
                rhxcore::types::AccessPrivileges::guest()
            }
        } else {
            // Session not found, use guest as fallback
            rhxcore::types::AccessPrivileges::guest()
        }
    };
    
    // Set admin flag and icon if user has administrative privileges
    let is_admin = access_privileges.contains(rhxcore::types::AccessPrivileges::DISCONNECT_USERS);
    if is_admin {
        flags |= UserFlags::ADMIN.bits();
        
        // If client didn't specify an icon, use the default admin icon (410)
        if icon_id == 0 {
            icon_id = 410;
        }
    }
    
    tracing::info!(
        "User {} agreed with nickname='{}', icon={}, flags=0x{:04X}, is_admin={}, access=0x{:016X}",
        user_id,
        nickname,
        icon_id,
        flags,
        is_admin,
        access_privileges.bits()
    );
    
    // Update session with user-provided info and computed flags
    if let Some(mut session) = state.get_session_mut(user_id) {
        session.nickname = nickname.clone();
        session.icon_id = icon_id;
        session.flags = flags;
    }
    
    // Broadcast NotifyChangeUser to all users
    state.broadcast(BroadcastMessage::UserJoined {
        user_id,
        nickname: nickname.clone(),
    });
    
    // Send acknowledgment reply (no fields needed)
    Ok(Some(Transaction {
        flags: 0,
        is_reply: true,
        transaction_type: TransactionType::Agreed,
        id: transaction.id,
        error_code: ErrorCode::NoError.to_u32(),
        total_size: 0,
        data_size: 0,
        fields: vec![],
    }))
}
