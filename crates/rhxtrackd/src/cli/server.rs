//! Server management commands (stub)

use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum ServerCommands {
    List { #[arg(short, long)] verbose: bool },
    Remove { server_id: String },
    Show { server_id: String },
}

pub async fn run(_config_path: &str, _command: ServerCommands) -> Result<()> {
    println!("Server management - Not yet implemented");
    Ok(())
}
