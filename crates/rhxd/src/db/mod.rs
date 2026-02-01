//! Database module
//! 
//! Handles SQLite database operations for accounts, sessions, files, and other data.

use anyhow::Result;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::path::Path;

pub mod accounts;
pub mod files;
pub mod schema;

/// Parse SQL statements from a script, handling comments and semicolons
fn parse_sql_statements(sql: &str) -> Vec<String> {
    let mut statements = Vec::new();
    let mut current = String::new();
    let mut in_string = false;
    let mut string_char = ' ';
    
    for line in sql.lines() {
        let trimmed = line.trim();
        
        // Skip full-line comments
        if trimmed.starts_with("--") {
            continue;
        }
        
        // Process character by character to handle strings properly
        for ch in line.chars() {
            if in_string {
                current.push(ch);
                if ch == string_char {
                    in_string = false;
                }
            } else {
                match ch {
                    '\'' | '"' => {
                        in_string = true;
                        string_char = ch;
                        current.push(ch);
                    }
                    ';' => {
                        let stmt = current.trim().to_string();
                        if !stmt.is_empty() && !stmt.starts_with("--") {
                            statements.push(stmt);
                        }
                        current.clear();
                    }
                    _ => current.push(ch),
                }
            }
        }
        
        // Add newline if we're building a statement
        if !current.trim().is_empty() {
            current.push('\n');
        }
    }
    
    // Add final statement if any
    let stmt = current.trim().to_string();
    if !stmt.is_empty() && !stmt.starts_with("--") {
        statements.push(stmt);
    }
    
    statements
}

/// Database connection pool
#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// Create a new database connection pool
    pub async fn new(path: impl AsRef<Path>) -> Result<Self> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        
        let options = SqliteConnectOptions::new()
            .filename(&path_str)
            .create_if_missing(true)
            .foreign_keys(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);
        
        let pool = SqlitePoolOptions::new()
            .max_connections(32)
            .connect_with(options)
            .await?;
        
        Ok(Self { pool })
    }
    
    /// Initialize the database schema
    pub async fn init_schema(&self) -> Result<()> {
        let schema_sql = include_str!("schema.sql");
        
        // Parse and execute SQL statements manually
        let statements = parse_sql_statements(schema_sql);
        
        for (idx, stmt) in statements.iter().enumerate() {
            let trimmed = stmt.trim();
            if !trimmed.is_empty() {
                sqlx::query(trimmed)
                    .execute(&self.pool)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to execute statement {}: {}\nStatement: {}", idx + 1, e, trimmed))?;
            }
        }
        
        tracing::info!("Database schema initialized ({} statements executed)", statements.len());
        Ok(())
    }
    
    /// Get the underlying connection pool
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
    
    /// Check if the database is healthy
    pub async fn health_check(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await?;
        Ok(())
    }
    
    /// Get the current schema version
    pub async fn schema_version(&self) -> Result<String> {
        let row: (String,) = sqlx::query_as(
            "SELECT value FROM server_metadata WHERE key = 'schema_version'"
        )
        .fetch_one(&self.pool)
        .await?;
        
        Ok(row.0)
    }
    
    /// Close the database connection pool
    pub async fn close(&self) {
        self.pool.close().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_database_init() {
        // Use a temp file instead of :memory: to avoid connection isolation issues
        let temp_path = format!("/tmp/test_rhxd_{}.db", std::process::id());
        
        let db = Database::new(&temp_path).await.unwrap();
        
        println!("Initializing schema...");
        
        // Parse statements first to see what we're dealing with
        let schema_sql = include_str!("schema.sql");
        let statements = parse_sql_statements(schema_sql);
        println!("Parsed {} statements", statements.len());
        for (i, stmt) in statements.iter().take(5).enumerate() {
            println!("Statement {}: {}", i + 1, &stmt[..stmt.len().min(100)]);
        }
        
        db.init_schema().await.unwrap();
        
        println!("Checking tables...");
        let tables: Vec<(String,)> = sqlx::query_as(
            "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name"
        )
        .fetch_all(db.pool())
        .await
        .unwrap();
        
        println!("Tables: {:?}", tables);
        
        // Verify schema version
        println!("Getting schema version...");
        let version = db.schema_version().await.unwrap();
        assert_eq!(version, "1");
        
        // Health check
        db.health_check().await.unwrap();
        
        // Cleanup
        std::fs::remove_file(&temp_path).ok();
    }
}
