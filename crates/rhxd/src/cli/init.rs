//! Server initialization command

use crate::Config;
use anyhow::Result;
use rand::Rng;
use std::path::Path;

pub async fn run(config_path: &str, _non_interactive: bool) -> Result<()> {
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
    
    // Create default admin account
    let admin_password = generate_password(16);
    let scrambled_password = rhxcore::password::scramble_password(admin_password.as_bytes());
    let access_bits = rhxcore::types::AccessPrivileges::admin().bits() as i64;
    
    // Use raw SQL to avoid sqlx macro issues during initial setup
    sqlx::query(
        "INSERT INTO accounts (login, password, name, icon_id, access_privileges) VALUES (?, ?, ?, ?, ?)"
    )
    .bind("admin")
    .bind(scrambled_password)
    .bind("Administrator")
    .bind(0)
    .bind(access_bits)
    .execute(&db)
    .await?;
    
    println!("✓ Admin account created\n");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("IMPORTANT: Save these credentials!");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("  Login:    admin");
    println!("  Password: {}", admin_password);
    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("You can change the password with:");
    println!("  rhxd account set-password admin <new-password>");
    println!();
    println!("To start the server:");
    println!("  rhxd serve");
    
    Ok(())
}

fn generate_password(length: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*";
    let mut rng = rand::rng();
    
    (0..length)
        .map(|_| {
            let idx = rng.random_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}
