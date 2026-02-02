//! Interactive console for server management

mod commands;

pub use commands::{Command, execute_command};

use anyhow::Result;
use std::sync::Arc;
use tokio::io::{self, AsyncBufReadExt, BufReader};

use crate::ServerState;

/// Run the interactive console loop
pub async fn run_console(state: Arc<ServerState>) -> Result<()> {
    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut line = String::new();
    
    println!("\n=== Hotline Server Console ===");
    println!("Type 'help' for available commands");
    println!();
    
    loop {
        // Print prompt
        print!("> ");
        // Manual flush since we're using print! not println!
        use std::io::Write;
        std::io::stdout().flush()?;
        
        // Read line
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => {
                // EOF (Ctrl+D)
                println!("\nEOF detected, shutting down...");
                break;
            }
            Ok(_) => {
                let input = line.trim();
                
                // Skip empty lines
                if input.is_empty() {
                    continue;
                }
                
                // Parse and execute command
                match Command::parse(input) {
                    Ok(Command::Stop) => {
                        println!("Shutting down server...");
                        break;
                    }
                    Ok(cmd) => {
                        if let Err(e) = execute_command(cmd, state.clone()).await {
                            eprintln!("Error: {}", e);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        println!("Type 'help' for available commands");
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                break;
            }
        }
    }
    
    Ok(())
}
