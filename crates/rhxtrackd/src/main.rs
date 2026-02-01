//! Hotline Tracker Daemon (rhxtrackd)

use anyhow::Result;
use clap::{Parser, Subcommand};

mod cli;
mod config;
mod server;
mod registry;
mod http;
mod db;

pub use config::Config;
pub use server::TrackerServer;

#[derive(Parser)]
#[command(name = "rhxtrackd")]
#[command(about = "Hotline Tracker Daemon", long_about = None)]
#[command(version)]
struct Cli {
    /// Path to configuration file
    #[arg(short, long, default_value = "rhxtrackd.json")]
    config: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new tracker (first-time setup)
    Init,
    
    /// Run the tracker server
    Serve,
    
    /// Server registry management
    Server {
        #[command(subcommand)]
        command: cli::server::ServerCommands,
    },
    
    /// Database operations
    Db {
        #[command(subcommand)]
        command: cli::db::DbCommands,
    },
    
    /// Show tracker information
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
        Commands::Init => {
            cli::init::run(&cli.config).await
        }
        Commands::Serve => {
            cli::serve::run(&cli.config).await
        }
        Commands::Server { command } => {
            cli::server::run(&cli.config, command).await
        }
        Commands::Db { command } => {
            cli::db::run(&cli.config, command).await
        }
        Commands::Info => {
            cli::info::run(&cli.config).await
        }
        Commands::Version => {
            println!("rhxtrackd version {}", env!("CARGO_PKG_VERSION"));
            println!("Protocol version: {}", rhxcore::protocol::PROTOCOL_VERSION);
            Ok(())
        }
    }
}
