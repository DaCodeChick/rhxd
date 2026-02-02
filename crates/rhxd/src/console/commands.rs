//! Console command definitions and execution

use anyhow::{anyhow, bail, Result};
use std::sync::Arc;

use crate::db::accounts::{create_account, delete_account, get_account_by_login, list_accounts};
use crate::state::{BroadcastMessage, ServerState};
use rhxcore::password::xor_password;
use rhxcore::types::AccessPrivileges;

/// Console commands
#[derive(Debug, Clone)]
pub enum Command {
    /// Create a new account with admin privileges
    CreateAccount { login: String, password: String },
    
    /// Delete an account by login
    DeleteAccount { login: String },
    
    /// List all accounts
    ListAccounts,
    
    /// Disconnect a user by ID or nickname
    Kick { target: String },
    
    /// Broadcast a message to all connected users
    Broadcast { message: String },
    
    /// List currently connected users
    ListUsers,
    
    /// Show help
    Help,
    
    /// Stop the server
    Stop,
}

impl Command {
    /// Parse a command from user input
    pub fn parse(input: &str) -> Result<Self> {
        let parts: Vec<&str> = input.split_whitespace().collect();
        
        if parts.is_empty() {
            bail!("Empty command");
        }
        
        match parts[0] {
            "create-account" => {
                if parts.len() < 3 {
                    bail!("Usage: create-account <login> <password>");
                }
                Ok(Command::CreateAccount {
                    login: parts[1].to_string(),
                    password: parts[2].to_string(),
                })
            }
            
            "delete-account" => {
                if parts.len() < 2 {
                    bail!("Usage: delete-account <login>");
                }
                Ok(Command::DeleteAccount {
                    login: parts[1].to_string(),
                })
            }
            
            "list-accounts" => {
                Ok(Command::ListAccounts)
            }
            
            "kick" => {
                if parts.len() < 2 {
                    bail!("Usage: kick <user_id|nickname>");
                }
                Ok(Command::Kick {
                    target: parts[1].to_string(),
                })
            }
            
            "broadcast" => {
                if parts.len() < 2 {
                    bail!("Usage: broadcast <message>");
                }
                // Join all parts after the command
                let message = parts[1..].join(" ");
                Ok(Command::Broadcast { message })
            }
            
            "list-users" => {
                Ok(Command::ListUsers)
            }
            
            "help" => {
                Ok(Command::Help)
            }
            
            "stop" | "shutdown" | "quit" | "exit" => {
                Ok(Command::Stop)
            }
            
            _ => {
                bail!("Unknown command: '{}'", parts[0]);
            }
        }
    }
}

/// Execute a console command
pub async fn execute_command(cmd: Command, state: Arc<ServerState>) -> Result<()> {
    match cmd {
        Command::CreateAccount { login, password } => {
            cmd_create_account(&state, &login, &password).await
        }
        
        Command::DeleteAccount { login } => {
            cmd_delete_account(&state, &login).await
        }
        
        Command::ListAccounts => {
            cmd_list_accounts(&state).await
        }
        
        Command::Kick { target } => {
            cmd_kick(&state, &target).await
        }
        
        Command::Broadcast { message } => {
            cmd_broadcast(&state, &message).await
        }
        
        Command::ListUsers => {
            cmd_list_users(&state).await
        }
        
        Command::Help => {
            cmd_help();
            Ok(())
        }
        
        Command::Stop => {
            // Handled in console loop
            Ok(())
        }
    }
}

/// Create a new account with admin privileges
async fn cmd_create_account(state: &ServerState, login: &str, password: &str) -> Result<()> {
    // Check if account already exists
    if get_account_by_login(state.database.pool(), login).await?.is_some() {
        bail!("Account '{}' already exists", login);
    }
    
    // Hash password
    let password_hash = xor_password(password.as_bytes());
    
    // Create account with full admin privileges
    let account_id = create_account(
        state.database.pool(),
        login,
        &password_hash,
        login, // Use login as name
        AccessPrivileges::admin(),
    ).await?;
    
    println!("Created admin account: {} (ID: {})", login, account_id);
    println!("Privileges: 0x{:016X}", AccessPrivileges::admin().bits());
    
    Ok(())
}

/// Delete an account by login
async fn cmd_delete_account(state: &ServerState, login: &str) -> Result<()> {
    // Check if account exists
    let account = get_account_by_login(state.database.pool(), login)
        .await?
        .ok_or_else(|| anyhow!("Account '{}' not found", login))?;
    
    // Delete the account
    delete_account(state.database.pool(), account.id).await?;
    
    println!("Deleted account: {} (ID: {})", login, account.id);
    
    Ok(())
}

/// List all accounts
async fn cmd_list_accounts(state: &ServerState) -> Result<()> {
    let accounts = list_accounts(state.database.pool()).await?;
    
    if accounts.is_empty() {
        println!("No accounts found");
        return Ok(());
    }
    
    println!("\n{:<5} {:<20} {:<20} {:<18}", "ID", "Login", "Name", "Privileges");
    println!("{}", "-".repeat(68));
    
    for account in accounts {
        println!(
            "{:<5} {:<20} {:<20} 0x{:016X}",
            account.id,
            account.login,
            account.name,
            account.access_privileges().bits()
        );
    }
    println!();
    
    Ok(())
}

/// Kick a user by ID or nickname
async fn cmd_kick(state: &ServerState, target: &str) -> Result<()> {
    // Try to parse as user ID first
    let user_id = if let Ok(id) = target.parse::<u16>() {
        Some(id)
    } else {
        // Search by nickname
        state.sessions.iter()
            .find(|s| s.nickname.eq_ignore_ascii_case(target))
            .map(|s| s.user_id)
    };
    
    let user_id = user_id.ok_or_else(|| anyhow!("User '{}' not found", target))?;
    
    // Get session info before disconnecting
    let (nickname, addr) = {
        let session = state.get_session(user_id)
            .ok_or_else(|| anyhow!("User {} not found", user_id))?;
        (session.nickname.clone(), session.address)
    };
    
    // Unregister the session (this will trigger disconnection)
    state.unregister_session(user_id);
    
    // Broadcast user left
    state.broadcast(BroadcastMessage::UserLeft { user_id });
    
    println!("Kicked user {} ({}) from {}", user_id, nickname, addr);
    
    Ok(())
}

/// Broadcast a message to all connected users
async fn cmd_broadcast(state: &ServerState, message: &str) -> Result<()> {
    let user_count = state.session_count();
    
    if user_count == 0 {
        println!("No users connected");
        return Ok(());
    }
    
    state.broadcast(BroadcastMessage::ServerMessage {
        message: message.to_string(),
    });
    
    println!("Broadcast message to {} user(s): {}", user_count, message);
    
    Ok(())
}

/// List currently connected users
async fn cmd_list_users(state: &ServerState) -> Result<()> {
    let sessions: Vec<_> = state.sessions.iter().map(|s| s.clone()).collect();
    
    if sessions.is_empty() {
        println!("No users connected");
        return Ok(());
    }
    
    println!("\n{:<6} {:<20} {:<20} {:<12}", "ID", "Nickname", "Address", "Auth State");
    println!("{}", "-".repeat(63));
    
    for session in sessions {
        println!(
            "{:<6} {:<20} {:<20} {:<12}",
            session.user_id,
            session.nickname,
            session.address,
            format!("{:?}", session.auth_state)
        );
    }
    println!();
    
    Ok(())
}

/// Show help
fn cmd_help() {
    println!("\nAvailable commands:");
    println!("  create-account <login> <password>  Create admin account");
    println!("  delete-account <login>             Delete an account");
    println!("  list-accounts                      Show all accounts");
    println!("  kick <user_id|nickname>            Disconnect a user");
    println!("  broadcast <message>                Send message to all users");
    println!("  list-users                         Show connected users");
    println!("  help                               Show this help");
    println!("  stop                               Shut down the server");
    println!();
}
