//! Server info command

use crate::Config;
use anyhow::Result;

pub async fn run(config_path: &str) -> Result<()> {
    let config = Config::load(config_path)?;
    
    println!("Server Information");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Name:        {}", config.server.name);
    println!("Description: {}", config.server.description);
    println!("Address:     {}:{}", config.server.address, config.server.port);
    println!("Max clients: {}", config.server.max_connections);
    println!();
    println!("Files root:  {}", config.files.root_path.display());
    println!("Database:    {}", config.database.path.display());
    println!();
    println!("Features:");
    println!("  News:          {}", if config.features.enable_news { "enabled" } else { "disabled" });
    println!("  Private chat:  {}", if config.features.enable_private_chat { "enabled" } else { "disabled" });
    println!("  File transfers: {}", if config.features.enable_file_transfers { "enabled" } else { "disabled" });
    
    Ok(())
}
