//! Server initialization command

use crate::Config;
use anyhow::{Context, Result};
use std::io::{self, Write};
use std::path::Path;

pub async fn run(config_path: &str, non_interactive: bool) -> Result<()> {
    println!("Initializing rhxd server...\n");
    
    // Check if config already exists
    if Path::new(config_path).exists() {
        println!("Error: Configuration file already exists: {}", config_path);
        println!("Remove it first or use a different path.");
        return Ok(());
    }
    
    // Create default config
    let config = Config::default();
    
    // Create necessary directories
    std::fs::create_dir_all(&config.files.root_path)?;
    if let Some(parent) = config.logging.file.parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    // Save config
    config.save(config_path)?;
    println!("✓ Configuration created: {}", config_path);
    
    // Initialize database
    let db_path = config.database.path.to_str().unwrap();
    let db = sqlx::SqlitePool::connect(&format!("sqlite:{}", db_path)).await?;
    
    // Run migrations
    sqlx::migrate!("./migrations").run(&db).await?;
    println!("✓ Database initialized: {}", db_path);
    
    // Prompt for admin credentials
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Admin Account Setup");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    let (admin_login, admin_password) = if non_interactive {
        // Non-interactive mode: use defaults
        ("admin".to_string(), "admin".to_string())
    } else {
        // Interactive mode: prompt user
        let login = prompt_input("Enter admin login name: ")?;
        let password = prompt_password("Enter admin password: ")?;
        let password_confirm = prompt_password("Confirm admin password: ")?;
        
        if password != password_confirm {
            anyhow::bail!("Passwords do not match");
        }
        
        if login.is_empty() {
            anyhow::bail!("Login cannot be empty");
        }
        
        if password.is_empty() {
            anyhow::bail!("Password cannot be empty");
        }
        
        (login, password)
    };
    
    // Create admin account
    let scrambled_password = rhxcore::password::xor_password(admin_password.as_bytes());
    let admin_access = rhxcore::types::AccessPrivileges::admin().bits() as i64;
    
    sqlx::query(
        "INSERT INTO accounts (login, password, name, icon_id, access_privileges) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&admin_login)
    .bind(scrambled_password)
    .bind("Administrator")
    .bind(0)
    .bind(admin_access)
    .execute(&db)
    .await
    .context("Failed to create admin account")?;
    
    println!("✓ Admin account created: {}", admin_login);
    
    // Create guest account
    let guest_password = rhxcore::password::xor_password(b"");
    let guest_access = rhxcore::types::AccessPrivileges::guest().bits() as i64;
    
    sqlx::query(
        "INSERT INTO accounts (login, password, name, icon_id, access_privileges) VALUES (?, ?, ?, ?, ?)"
    )
    .bind("guest")
    .bind(guest_password)
    .bind("Guest")
    .bind(0)
    .bind(guest_access)
    .execute(&db)
    .await
    .context("Failed to create guest account")?;
    
    println!("✓ Guest account created\n");
    
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Setup Complete!");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("Admin credentials:");
    println!("  Login:    {}", admin_login);
    println!("  Password: {}", admin_password);
    println!();
    println!("Guest credentials:");
    println!("  Login:    guest");
    println!("  Password: (empty)");
    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("To start the server:");
    println!("  rhxd serve");
    
    Ok(())
}

/// Prompt for text input
fn prompt_input(prompt: &str) -> Result<String> {
    print!("{}", prompt);
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    Ok(input.trim().to_string())
}

/// Prompt for password (uses rpassword if available, falls back to visible input)
fn prompt_password(prompt: &str) -> Result<String> {
    // Try to use rpassword for hidden input
    match rpassword::prompt_password(prompt) {
        Ok(pass) => Ok(pass),
        Err(_) => {
            // Fallback to visible input
            print!("{}", prompt);
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            
            Ok(input.trim().to_string())
        }
    }
}
