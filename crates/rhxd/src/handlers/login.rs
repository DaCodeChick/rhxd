//! Login transaction handler

use crate::connection::transaction_helpers::{create_error_reply, create_success_reply};
use crate::state::ServerState;
use anyhow::{Context, Result};
use rhxcore::password::unscramble_password;
use rhxcore::protocol::{ErrorCode, Field, FieldId, Transaction, TransactionType, SERVER_VERSION};
use std::sync::Arc;

/// Handle login transaction (107)
///
/// Client sends:
/// - Field 105: User login (scrambled)
/// - Field 106: User password (scrambled)
/// - Field 160: Client version
///
/// Server replies with:
/// - Field 160: Server version
/// - Field 161: Banner ID (if version >= 151)
/// - Field 162: Server name (if version >= 151)
pub async fn handle_login(
    transaction: Transaction,
    user_id: u16,
    state: Arc<ServerState>,
) -> Result<Transaction> {
    tracing::debug!("User {} sent login transaction", user_id);
    
    // Extract fields
    let mut login: Option<Vec<u8>> = None;
    let mut password: Option<Vec<u8>> = None;
    
    for field in &transaction.fields {
        match field.id {
            FieldId::UserLogin => {
                login = field.as_binary().map(|b| b.to_vec());
            }
            FieldId::UserPassword => {
                password = field.as_binary().map(|b| b.to_vec());
            }
            _ => {}
        }
    }
    
    // Check for guest login (empty login/password)
    let is_guest = login.as_ref().map_or(true, |l| l.is_empty())
        || password.as_ref().map_or(true, |p| p.is_empty());
    
    if is_guest && !state.config.security.allow_guest {
        tracing::warn!("User {} attempted guest login but guests not allowed", user_id);
        return Ok(create_error_reply(&transaction, ErrorCode::PermissionDenied));
    }
    
    // Handle guest login
    if is_guest {
        tracing::info!("User {} logged in as guest", user_id);
        
        // Get guest access privileges
        let guest_access = rhxcore::types::AccessPrivileges::guest();
        
        tracing::info!(
            "User {} guest access: 0x{:016X} (READ_CHAT={}, SEND_CHAT={})",
            user_id,
            guest_access.bits(),
            guest_access.contains(rhxcore::types::AccessPrivileges::READ_CHAT),
            guest_access.contains(rhxcore::types::AccessPrivileges::SEND_CHAT)
        );
        
        // Update session to authenticated
        if let Some(mut session) = state.get_session_mut(user_id) {
            session.authenticate_guest(format!("Guest {}", user_id), 0);
        }
        
        // Encode access as 8 bytes (Int64 in protocol spec)
        // Use to_wire_format() to handle bit reversal on little-endian systems
        let access_bytes = guest_access.to_wire_format().to_vec();
        
        tracing::debug!(
            "Sending UserAccess field (8 bytes): {:02X?}",
            access_bytes
        );
        
        // Create reply
        let mut reply_fields = vec![
            Field::integer(FieldId::Version, SERVER_VERSION as i32),
            Field::integer(FieldId::UserId, user_id as i32),  // Client needs to know their user ID
            Field::binary(FieldId::UserAccess, access_bytes),
        ];
        
        // Add server name and banner for version >= 151
        // Note: We always send these for modern clients
        reply_fields.push(Field::integer(FieldId::BannerId, 0));
        reply_fields.push(Field::string(
            FieldId::ServerName,
            &state.config.server.name,
        ));
        
        return Ok(create_success_reply(&transaction, reply_fields));
    }
    
    // Handle authenticated login
    let login = login.context("Missing login field")?;
    let password = password.context("Missing password field")?;
    
    // Unscramble login and password
    let login_str = String::from_utf8_lossy(&unscramble_password(&login)).to_string();
    let password_bytes = unscramble_password(&password);
    
    tracing::debug!("User {} attempting login as '{}'", user_id, login_str);
    
    // Look up account in database
    let account = crate::db::accounts::get_account_by_login(state.database.pool(), &login_str)
        .await
        .context("Database error during login")?;
    
    match account {
        Some(account) => {
            // Verify password
            let password_hash = hex::decode(&account.password_hash)
                .context("Invalid password hash in database")?;
            
            if rhxcore::password::verify_password(&password_hash, &password_bytes) {
                tracing::info!(
                    "User {} successfully authenticated as '{}' (account_id={})",
                    user_id,
                    login_str,
                    account.id
                );
                
                // Update session with account info
                if let Some(mut session) = state.get_session_mut(user_id) {
                    session.authenticate_user(account.id, account.name.clone(), 0);
                }
                
                // Get user access privileges from account
                let user_access = account.access_privileges();
                
                tracing::info!(
                    "User {} access: 0x{:016X}",
                    user_id,
                    user_access.bits()
                );
                
                // Create reply
                let mut reply_fields = vec![
                    Field::integer(FieldId::Version, SERVER_VERSION as i32),
                    Field::integer(FieldId::UserId, user_id as i32),  // Client needs to know their user ID
                    // UserAccess as 8 bytes (Int64) with proper bit reversal
                    Field::binary(FieldId::UserAccess, user_access.to_wire_format().to_vec()),
                ];
                
                // Add server name and banner
                reply_fields.push(Field::integer(FieldId::BannerId, 0));
                reply_fields.push(Field::string(
                    FieldId::ServerName,
                    &state.config.server.name,
                ));
                
                Ok(create_success_reply(&transaction, reply_fields))
            } else {
                tracing::warn!("User {} failed authentication - invalid password", user_id);
                Ok(create_error_reply(&transaction, ErrorCode::PermissionDenied))
            }
        }
        None => {
            tracing::warn!("User {} failed authentication - account '{}' not found", user_id, login_str);
            Ok(create_error_reply(&transaction, ErrorCode::PermissionDenied))
        }
    }
}
