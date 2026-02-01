//! Login transaction handler

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
    let mut client_version: Option<i32> = None;
    
    for field in &transaction.fields {
        match field.id {
            FieldId::UserLogin => {
                login = field.as_binary().map(|b| b.to_vec());
            }
            FieldId::UserPassword => {
                password = field.as_binary().map(|b| b.to_vec());
            }
            FieldId::Version => {
                client_version = field.as_integer();
            }
            _ => {}
        }
    }
    
    // Check for guest login (empty login/password)
    let is_guest = login.as_ref().map_or(true, |l| l.is_empty())
        || password.as_ref().map_or(true, |p| p.is_empty());
    
    if is_guest && !state.config.security.allow_guest {
        tracing::warn!("User {} attempted guest login but guests not allowed", user_id);
        return Ok(Transaction {
            flags: 0,
            is_reply: true,
            transaction_type: TransactionType::Login,
            id: transaction.id,
            error_code: ErrorCode::PermissionDenied.to_u32(),
            total_size: 0,
            data_size: 0,
            fields: vec![],
        });
    }
    
    // Handle guest login
    if is_guest {
        tracing::info!("User {} logged in as guest", user_id);
        
        // Update session to authenticated
        if let Some(mut session) = state.get_session_mut(user_id) {
            session.authenticate_guest(format!("Guest {}", user_id), 0);
        }
        
        // Create reply
        let mut reply_fields = vec![
            Field::integer(FieldId::Version, SERVER_VERSION as i32),
        ];
        
        // Add server name and banner for version >= 151
        // Note: We always send these for modern clients
        reply_fields.push(Field::integer(FieldId::BannerId, 0));
        reply_fields.push(Field::string(
            FieldId::ServerName,
            &state.config.server.name,
        ));
        
        return Ok(Transaction {
            flags: 0,
            is_reply: true,
            transaction_type: TransactionType::Login,
            id: transaction.id,
            error_code: ErrorCode::NoError.to_u32(),
            total_size: 0,
            data_size: 0,
            fields: reply_fields,
        });
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
                
                // Create reply
                let mut reply_fields = vec![
                    Field::integer(FieldId::Version, SERVER_VERSION as i32),
                ];
                
                // Add server name and banner
                reply_fields.push(Field::integer(FieldId::BannerId, 0));
                reply_fields.push(Field::string(
                    FieldId::ServerName,
                    &state.config.server.name,
                ));
                
                Ok(Transaction {
                    flags: 0,
                    is_reply: true,
                    transaction_type: TransactionType::Login,
                    id: transaction.id,
                    error_code: ErrorCode::NoError.to_u32(),
                    total_size: 0,
                    data_size: 0,
                    fields: reply_fields,
                })
            } else {
                tracing::warn!("User {} failed authentication - invalid password", user_id);
                Ok(Transaction {
                    flags: 0,
                    is_reply: true,
                    transaction_type: TransactionType::Login,
                    id: transaction.id,
                    error_code: ErrorCode::PermissionDenied.to_u32(),
                    total_size: 0,
                    data_size: 0,
                    fields: vec![],
                })
            }
        }
        None => {
            tracing::warn!("User {} failed authentication - account '{}' not found", user_id, login_str);
            Ok(Transaction {
                flags: 0,
                is_reply: true,
                transaction_type: TransactionType::Login,
                id: transaction.id,
                error_code: ErrorCode::PermissionDenied.to_u32(),
                total_size: 0,
                data_size: 0,
                fields: vec![],
            })
        }
    }
}
