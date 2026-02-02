//! User info transaction handlers

use crate::connection::transaction_helpers::{create_error_reply, create_success_reply};
use crate::db::accounts::get_account_by_id;
use crate::state::ServerState;
use anyhow::Result;
use chrono::{DateTime, Utc};
use rhxcore::protocol::{ErrorCode, Field, FieldId, Transaction};
use rhxcore::types::AccessPrivileges;
use std::sync::Arc;
use std::time::SystemTime;

/// Handle GetClientInfoText (303) transaction
///
/// Allows users with GET_USER_INFO privilege to retrieve detailed information
/// about a connected user.
pub async fn handle_get_client_info_text(
    transaction: Transaction,
    user_id: u16,
    state: Arc<ServerState>,
) -> Result<Option<Transaction>> {
    tracing::debug!("User {} requested client info", user_id);

    // Get the requesting user's session
    let session = match state.get_session(user_id) {
        Some(s) => s,
        None => {
            tracing::warn!(
                "User {} requested client info but session not found",
                user_id
            );
            return Ok(None);
        }
    };

    // Check if requester has GET_USER_INFO privilege
    if let Some(account_id) = session.account_id {
        match get_account_by_id(state.database.pool(), account_id).await? {
            Some(account) => {
                if !account.has_privilege(AccessPrivileges::GET_USER_INFO) {
                    return Ok(Some(create_error_reply(
                        &transaction,
                        ErrorCode::PermissionDenied,
                    )));
                }
            }
            None => {
                return Ok(Some(create_error_reply(
                    &transaction,
                    ErrorCode::PermissionDenied,
                )));
            }
        }
    } else {
        // Guests don't have GET_USER_INFO privilege
        return Ok(Some(create_error_reply(
            &transaction,
            ErrorCode::PermissionDenied,
        )));
    }

    // Extract the requested user ID from the request
    let mut target_user_id = None;
    for field in &transaction.fields {
        if field.id == FieldId::UserId {
            if let Some(id) = field.as_integer() {
                target_user_id = Some(id as u16);
                break;
            }
        }
    }

    let target_user_id = match target_user_id {
        Some(id) => id,
        None => {
            tracing::warn!(
                "User {} sent GetClientInfoText without UserId field",
                user_id
            );
            return Ok(Some(create_error_reply(
                &transaction,
                ErrorCode::InvalidParameter,
            )));
        }
    };

    // Find the target user's session
    let target_session = match state.get_session(target_user_id) {
        Some(s) => s,
        None => {
            return Ok(Some(create_error_reply(&transaction, ErrorCode::NotFound)));
        }
    };

    // Build the user info text
    let info_text = build_user_info_text(&state, &target_session).await?;

    tracing::info!(
        "User {} requested info for user {} ({})",
        user_id,
        target_user_id,
        target_session.nickname
    );

    // Build the response
    Ok(Some(create_success_reply(
        &transaction,
        vec![
            Field::binary(FieldId::Data, info_text.into_bytes()),
            Field::string(FieldId::UserName, target_session.nickname.clone()),
            Field::integer(FieldId::UserIconId, target_session.icon_id as i32),
        ],
    )))
}

/// Build the formatted user info text
async fn build_user_info_text(
    state: &ServerState,
    session: &crate::connection::session::Session,
) -> Result<String> {
    // Calculate "away" time (time since last activity)
    let away_duration = SystemTime::now()
        .duration_since(session.last_activity)
        .unwrap_or_default();
    let away_minutes = away_duration.as_secs() / 60;
    let away_seconds = away_duration.as_secs() % 60;

    // Format connection time
    let connected_datetime: DateTime<Utc> = session.connected_at.into();
    let connected_str = connected_datetime
        .format("%Y-%m-%d %H:%M:%S UTC")
        .to_string();

    // Get account information if not a guest
    let (account_name, account_login) = if let Some(account_id) = session.account_id {
        match get_account_by_id(state.database.pool(), account_id).await? {
            Some(account) => (account.name.clone(), account.login.clone()),
            None => ("Unknown".to_string(), "Unknown".to_string()),
        }
    } else {
        ("Guest".to_string(), "Guest".to_string())
    };

    // Extract IP address
    let ip = session.address.ip().to_string();

    // Build the formatted text following GLoarbLine's exact format
    // Note: GLoarbLine uses \r (carriage return) as line separator, which is Mac Classic convention
    // Format matches GLoarbLine's normal mode (non-bot) output
    let info_text = format!(
        "Nickname:   {}\r\
         User ID:    {}\r\
         Icon:       {}\r\
         Away:       {} min {} sec\r\
         Name:       {}\r\
         Account:    {}\r\
         Address:    {}",
        session.nickname,
        session.user_id,
        session.icon_id,
        away_minutes,
        away_seconds,
        account_name,
        account_login,
        ip
    );

    Ok(info_text)
}
