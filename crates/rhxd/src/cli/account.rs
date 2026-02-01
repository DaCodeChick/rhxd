//! Account management commands

use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum AccountCommands {
    /// Create a new account
    Add {
        login: String,
        password: String,
        name: String,
        #[arg(long)]
        admin: bool,
    },
    /// Delete an account
    Delete { login: String },
    /// List all accounts
    List {
        #[arg(short, long)]
        verbose: bool,
    },
    /// Show account details
    Show { login: String },
    /// Change account password
    SetPassword { login: String, new_password: String },
}

pub async fn run(_config_path: &str, _command: AccountCommands) -> Result<()> {
    // TODO: Implement account management
    println!("Account management not yet implemented");
    Ok(())
}
