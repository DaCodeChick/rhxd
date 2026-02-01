//! Hotline Server Daemon (rhxd)

use anyhow::Result;
use clap::{Parser, Subcommand};

mod cli;
mod config;
mod server;
mod state;
mod connection;
mod handlers;
mod db;
mod broadcast;

pub use config::Config;
pub use server::Server;
pub use state::ServerState;

#[derive(Parser)]
#[command(name = "rhxd")]
#[command(about = "Hotline Server Daemon", long_about = None)]
#[command(version)]
struct Cli {
    /// Path to configuration file
    #[arg(short, long, default_value = "rhxd.json")]
    config: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new server (first-time setup)
    Init {
        /// Skip interactive prompts
        #[arg(long)]
        non_interactive: bool,
    },
    
    /// Run the Hotline server
    Serve,
    
    /// Account management
    Account {
        #[command(subcommand)]
        command: cli::account::AccountCommands,
    },
    
    /// Database operations
    Db {
        #[command(subcommand)]
        command: cli::db::DbCommands,
    },
    
    /// Show server information
    Info,
    
    /// Show version information
    Version,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(true)
        .with_level(true)
        .init();
    
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Init { non_interactive } => {
            cli::init::run(&cli.config, non_interactive).await
        }
        Commands::Serve => {
            cli::serve::run(&cli.config).await
        }
        Commands::Account { command } => {
            cli::account::run(&cli.config, command).await
        }
        Commands::Db { command } => {
            cli::db::run(&cli.config, command).await
        }
        Commands::Info => {
            cli::info::run(&cli.config).await
        }
        Commands::Version => {
            println!("rhxd version {}", env!("CARGO_PKG_VERSION"));
            println!("Protocol version: {}", rhxcore::protocol::PROTOCOL_VERSION);
            Ok(())
        }
    }
}
