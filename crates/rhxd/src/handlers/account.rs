//! Account management transaction handlers
//!
//! Implements user account CRUD operations for admin users:
//! - NewUser (350): Create new account
//! - GetUser (352): Get account details
//! - SetUser (353): Modify account  
//! - DeleteUser (351): Delete account

use crate::connection::transaction_helpers::{create_error_reply, create_success_reply};
use crate::state::ServerState;
use anyhow::{Context, Result};
use rhxcore::password::xor_password;
use rhxcore::protocol::{ErrorCode, Field, FieldId, Transaction};
use rhxcore::types::AccessPrivileges;
use std::sync::Arc;

/// Check if the user has a specific privilege
async fn check_privilege(
    state: &ServerState,
    user_id: u16,
    required: AccessPrivileges,
) -> Result<bool> {
    // Get user's session and account
    let account_id = match state.get_session(user_id) {
        Some(session) => session.account_id,
        None => return Ok(false),
    };
    
    let account_id = match account_id {
        Some(id) => id,
        None => return Ok(false), // Guests don't have privileges
    };
    
    // Get account from database
    let account = crate::db::accounts::get_account_by_id(state.database.pool(), account_id)
        .await
        .context("Database error")?;
    
    match account {
        Some(account) => Ok(account.has_privilege(required)),
        None => Ok(false),
    }
}

/// Handle NewUser transaction (350) - Create new account
///
/// Client sends:
/// - Field 105: Login name (binary, scrambled)
/// - Field 106: Password (binary, scrambled)
/// - Field 102: Display name (string)
/// - Field 110: Access privileges (8 bytes, i64)
///
/// Server replies with:
/// - Empty success or error code
pub async fn handle_new_user(
    transaction: Transaction,
    user_id: u16,
    state: Arc<ServerState>,
) -> Result<Transaction> {
    tracing::debug!("User {} attempting to create new account", user_id);
    
    // Check if user has CREATE_USERS permission
    if !check_privilege(&state, user_id, AccessPrivileges::CREATE_USERS).await? {
        tracing::warn!("User {} tried to create account without permission", user_id);
        return Ok(create_error_reply(&transaction, ErrorCode::PermissionDenied));
    }
    
    // Extract fields
    let mut login: Option<Vec<u8>> = None;
    let mut password: Option<Vec<u8>> = None;
    let mut name: Option<String> = None;
    let mut access: Option<i64> = None;
    
    for field in &transaction.fields {
        match field.id {
            FieldId::UserLogin => {
                login = field.as_binary().map(|b| b.to_vec());
            }
            FieldId::UserPassword => {
                password = field.as_binary().map(|b| b.to_vec());
            }
            FieldId::UserName => {
                name = field.as_string().map(|s| s.to_string());
            }
            FieldId::UserAccess => {
                access = field.as_binary().and_then(|bytes| {
                    if bytes.len() == 8 {
                        // Read as big-endian i64 from wire format
                        let mut arr = [0u8; 8];
                        arr.copy_from_slice(bytes);
                        Some(i64::from_be_bytes(arr))
                    } else {
                        None
                    }
                });
            }
            _ => {}
        }
    }
    
    // Validate required fields
    let login = login.context("Missing login field")?;
    let password = password.context("Missing password field")?;
    let name = name.context("Missing name field")?;
    let access = access.unwrap_or(0);
    
    // Unscramble login and password
    let login_bytes = xor_password(&login);
    let password_bytes = xor_password(&password);
    
    let login_str = String::from_utf8(login_bytes)
        .context("Invalid UTF-8 in login")?;
    
    tracing::info!(
        "User {} creating account '{}' with name '{}' and access 0x{:016X}",
        user_id,
        login_str,
        name,
        access
    );
    
    // Validate input
    if login_str.is_empty() {
        tracing::warn!("User {} tried to create account with empty login", user_id);
        return Ok(create_error_reply(&transaction, ErrorCode::InvalidParameter));
    }
    
    // Check if account already exists
    if crate::db::accounts::account_exists(state.database.pool(), &login_str).await? {
        tracing::warn!("User {} tried to create duplicate account '{}'", user_id, login_str);
        return Ok(create_error_reply(&transaction, ErrorCode::AlreadyExists));
    }
    
    // Store the password (it's already scrambled from the client)
    // We store it as-is for compatibility with Hotline password verification
    let password_storage = &password_bytes;
    
    // Convert access to AccessPrivileges
    let access_privileges = AccessPrivileges::from_bits_truncate(access as u64);
    
    // Create account in database
    let account_id = crate::db::accounts::create_account(
        state.database.pool(),
        &login_str,
        password_storage,
        &name,
        access_privileges,
    )
    .await
    .context("Failed to create account")?;
    
    tracing::info!(
        "User {} successfully created account '{}' (id={})",
        user_id,
        login_str,
        account_id
    );
    
    // Return success
    Ok(create_success_reply(&transaction, vec![]))
}

/// Handle GetUser transaction (352) - Get account details
///
/// Client sends:
/// - Field 105: Login name (binary, scrambled)
///
/// Server replies with:
/// - Field 102: Display name (string)
/// - Field 105: Login name (binary, scrambled)
/// - Field 110: Access privileges (8 bytes)
pub async fn handle_get_user(
    transaction: Transaction,
    user_id: u16,
    state: Arc<ServerState>,
) -> Result<Transaction> {
    tracing::debug!("User {} requesting account details", user_id);
    
    // Check if user has OPEN_USER permission
    if !check_privilege(&state, user_id, AccessPrivileges::OPEN_USER).await? {
        tracing::warn!("User {} tried to get account without permission", user_id);
        return Ok(create_error_reply(&transaction, ErrorCode::PermissionDenied));
    }
    
    // Extract login field
    let login = transaction.fields.iter()
        .find(|f| f.id == FieldId::UserLogin)
        .and_then(|f| f.as_binary())
        .context("Missing login field")?;
    
    // Unscramble login
    let login_bytes = xor_password(login);
    let login_str = String::from_utf8(login_bytes)
        .context("Invalid UTF-8 in login")?;
    
    tracing::debug!("User {} getting account '{}'", user_id, login_str);
    
    // Get account from database
    let account = crate::db::accounts::get_account_by_login(state.database.pool(), &login_str)
        .await
        .context("Database error")?;
    
    let account = match account {
        Some(acc) => acc,
        None => {
            tracing::warn!("User {} requested non-existent account '{}'", user_id, login_str);
            return Ok(create_error_reply(&transaction, ErrorCode::NotFound));
        }
    };
    
    tracing::info!("User {} retrieved account '{}'", user_id, account.login);
    
    // Scramble login for response (keep it scrambled as client expects)
    let scrambled_login = xor_password(account.login.as_bytes());
    
    // Encode access privileges as 8 bytes (big-endian for wire format)
    let access_bytes = (account.access as i64).to_be_bytes().to_vec();
    
    // Return account details
    Ok(create_success_reply(&transaction, vec![
        Field::string(FieldId::UserName, &account.name),
        Field::binary(FieldId::UserLogin, scrambled_login),
        Field::binary(FieldId::UserAccess, access_bytes),
    ]))
}

/// Handle SetUser transaction (353) - Modify account
///
/// Client sends:
/// - Field 105: Login name (binary, scrambled) - identifies which account to modify
/// - Field 102: New display name (string, optional)
/// - Field 106: New password (binary, scrambled, optional)
/// - Field 110: New access privileges (8 bytes, optional)
///
/// Server replies with:
/// - Empty success or error code
pub async fn handle_set_user(
    transaction: Transaction,
    user_id: u16,
    state: Arc<ServerState>,
) -> Result<Transaction> {
    tracing::debug!("User {} attempting to modify account", user_id);
    
    // Check if user has MODIFY_USERS permission
    if !check_privilege(&state, user_id, AccessPrivileges::MODIFY_USERS).await? {
        tracing::warn!("User {} tried to modify account without permission", user_id);
        return Ok(create_error_reply(&transaction, ErrorCode::PermissionDenied));
    }
    
    // Extract fields
    let mut login: Option<Vec<u8>> = None;
    let mut password: Option<Vec<u8>> = None;
    let mut name: Option<String> = None;
    let mut access: Option<i64> = None;
    
    for field in &transaction.fields {
        match field.id {
            FieldId::UserLogin => {
                login = field.as_binary().map(|b| b.to_vec());
            }
            FieldId::UserPassword => {
                password = field.as_binary().map(|b| b.to_vec());
            }
            FieldId::UserName => {
                name = field.as_string().map(|s| s.to_string());
            }
            FieldId::UserAccess => {
                access = field.as_binary().and_then(|bytes| {
                    if bytes.len() == 8 {
                        let mut arr = [0u8; 8];
                        arr.copy_from_slice(bytes);
                        Some(i64::from_be_bytes(arr))
                    } else {
                        None
                    }
                });
            }
            _ => {}
        }
    }
    
    // Login is required to identify the account
    let login = login.context("Missing login field")?;
    let login_bytes = xor_password(&login);
    let login_str = String::from_utf8(login_bytes)
        .context("Invalid UTF-8 in login")?;
    
    tracing::debug!("User {} modifying account '{}'", user_id, login_str);
    
    // Get account from database
    let account = crate::db::accounts::get_account_by_login(state.database.pool(), &login_str)
        .await
        .context("Database error")?;
    
    let account = match account {
        Some(acc) => acc,
        None => {
            tracing::warn!("User {} tried to modify non-existent account '{}'", user_id, login_str);
            return Ok(create_error_reply(&transaction, ErrorCode::NotFound));
        }
    };
    
    // Update password if provided
    if let Some(password_data) = password {
        let password_bytes = xor_password(&password_data);
        
        crate::db::accounts::update_password(state.database.pool(), account.id, &password_bytes)
            .await
            .context("Failed to update password")?;
        
        tracing::info!("User {} updated password for account '{}'", user_id, login_str);
    }
    
    // Update access if provided
    if let Some(access_bits) = access {
        let access_privileges = AccessPrivileges::from_bits_truncate(access_bits as u64);
        
        crate::db::accounts::update_access(state.database.pool(), account.id, access_privileges)
            .await
            .context("Failed to update access")?;
        
        tracing::info!(
            "User {} updated access for account '{}' to 0x{:016X}",
            user_id,
            login_str,
            access_bits
        );
    }
    
    // Note: Name updates would require a new function in db/accounts.rs
    // For now, we'll log but not implement it
    if let Some(new_name) = name {
        tracing::warn!(
            "User {} tried to update name for '{}' to '{}', but name updates not yet implemented",
            user_id,
            login_str,
            new_name
        );
    }
    
    tracing::info!("User {} successfully modified account '{}'", user_id, login_str);
    
    // Return success
    Ok(create_success_reply(&transaction, vec![]))
}

/// Handle DeleteUser transaction (351) - Delete account
///
/// Client sends:
/// - Field 105: Login name (binary, scrambled)
///
/// Server replies with:
/// - Empty success or error code
pub async fn handle_delete_user(
    transaction: Transaction,
    user_id: u16,
    state: Arc<ServerState>,
) -> Result<Transaction> {
    tracing::debug!("User {} attempting to delete account", user_id);
    
    // Check if user has DELETE_USERS permission
    if !check_privilege(&state, user_id, AccessPrivileges::DELETE_USERS).await? {
        tracing::warn!("User {} tried to delete account without permission", user_id);
        return Ok(create_error_reply(&transaction, ErrorCode::PermissionDenied));
    }
    
    // Extract login field
    let login = transaction.fields.iter()
        .find(|f| f.id == FieldId::UserLogin)
        .and_then(|f| f.as_binary())
        .context("Missing login field")?;
    
    // Unscramble login
    let login_bytes = xor_password(login);
    let login_str = String::from_utf8(login_bytes)
        .context("Invalid UTF-8 in login")?;
    
    tracing::debug!("User {} deleting account '{}'", user_id, login_str);
    
    // Get account from database
    let account = crate::db::accounts::get_account_by_login(state.database.pool(), &login_str)
        .await
        .context("Database error")?;
    
    let account = match account {
        Some(acc) => acc,
        None => {
            tracing::warn!("User {} tried to delete non-existent account '{}'", user_id, login_str);
            return Ok(create_error_reply(&transaction, ErrorCode::NotFound));
        }
    };
    
    // Delete the account
    crate::db::accounts::delete_account(state.database.pool(), account.id)
        .await
        .context("Failed to delete account")?;
    
    tracing::info!("User {} successfully deleted account '{}' (id={})", user_id, login_str, account.id);
    
    // Return success
    Ok(create_success_reply(&transaction, vec![]))
}
