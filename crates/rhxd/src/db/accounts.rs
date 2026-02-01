//! Account management operations

#![allow(dead_code)] // Many functions are for future use

use anyhow::{bail, Result};
use chrono::Utc;
use rhxcore::types::access::AccessPrivileges;
use sqlx::SqlitePool;

/// Account record from database
#[derive(Debug, Clone)]
pub struct Account {
    pub id: i64,
    pub login: String,
    pub password_hash: Vec<u8>,
    pub name: String,
    pub access: i64,
    pub created_at: i64,
    pub modified_at: i64,
}

impl Account {
    /// Get access privileges
    pub fn access_privileges(&self) -> AccessPrivileges {
        AccessPrivileges::from_bits_truncate(self.access as u64)
    }
    
    /// Check if account has specific privilege
    pub fn has_privilege(&self, privilege: AccessPrivileges) -> bool {
        self.access_privileges().contains(privilege)
    }
}

/// Create a new account
pub async fn create_account(
    pool: &SqlitePool,
    login: &str,
    password_hash: &[u8],
    name: &str,
    access: AccessPrivileges,
) -> Result<i64> {
    // Validate input lengths
    if login.len() > 31 {
        bail!("Login must be 31 characters or less");
    }
    if name.len() > 31 {
        bail!("Name must be 31 characters or less");
    }
    
    let now = Utc::now().timestamp();
    let access_bits = access.bits() as i64;
    
    let result = sqlx::query(
        "INSERT INTO accounts (login, password_hash, name, access, created_at, modified_at)
         VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(login)
    .bind(password_hash)
    .bind(name)
    .bind(access_bits)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;
    
    Ok(result.last_insert_rowid())
}

/// Get account by login
pub async fn get_account_by_login(pool: &SqlitePool, login: &str) -> Result<Option<Account>> {
    let account = sqlx::query_as::<_, (i64, String, Vec<u8>, String, i64, i64, i64)>(
        "SELECT id, login, password_hash, name, access, created_at, modified_at
         FROM accounts WHERE login = ? COLLATE NOCASE"
    )
    .bind(login)
    .fetch_optional(pool)
    .await?;
    
    Ok(account.map(|(id, login, password_hash, name, access, created_at, modified_at)| {
        Account {
            id,
            login,
            password_hash,
            name,
            access,
            created_at,
            modified_at,
        }
    }))
}

/// Get account by ID
pub async fn get_account_by_id(pool: &SqlitePool, id: i64) -> Result<Option<Account>> {
    let account = sqlx::query_as::<_, (i64, String, Vec<u8>, String, i64, i64, i64)>(
        "SELECT id, login, password_hash, name, access, created_at, modified_at
         FROM accounts WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    
    Ok(account.map(|(id, login, password_hash, name, access, created_at, modified_at)| {
        Account {
            id,
            login,
            password_hash,
            name,
            access,
            created_at,
            modified_at,
        }
    }))
}

/// List all accounts
pub async fn list_accounts(pool: &SqlitePool) -> Result<Vec<Account>> {
    let accounts = sqlx::query_as::<_, (i64, String, Vec<u8>, String, i64, i64, i64)>(
        "SELECT id, login, password_hash, name, access, created_at, modified_at
         FROM accounts ORDER BY login"
    )
    .fetch_all(pool)
    .await?;
    
    Ok(accounts
        .into_iter()
        .map(|(id, login, password_hash, name, access, created_at, modified_at)| {
            Account {
                id,
                login,
                password_hash,
                name,
                access,
                created_at,
                modified_at,
            }
        })
        .collect())
}

/// Update account password
pub async fn update_password(
    pool: &SqlitePool,
    account_id: i64,
    new_password_hash: &[u8],
) -> Result<()> {
    let now = Utc::now().timestamp();
    
    sqlx::query(
        "UPDATE accounts SET password_hash = ?, modified_at = ? WHERE id = ?"
    )
    .bind(new_password_hash)
    .bind(now)
    .bind(account_id)
    .execute(pool)
    .await?;
    
    Ok(())
}

/// Update account access privileges
pub async fn update_access(
    pool: &SqlitePool,
    account_id: i64,
    access: AccessPrivileges,
) -> Result<()> {
    let now = Utc::now().timestamp();
    let access_bits = access.bits() as i64;
    
    sqlx::query(
        "UPDATE accounts SET access = ?, modified_at = ? WHERE id = ?"
    )
    .bind(access_bits)
    .bind(now)
    .bind(account_id)
    .execute(pool)
    .await?;
    
    Ok(())
}

/// Delete an account
pub async fn delete_account(pool: &SqlitePool, account_id: i64) -> Result<()> {
    sqlx::query("DELETE FROM accounts WHERE id = ?")
        .bind(account_id)
        .execute(pool)
        .await?;
    
    Ok(())
}

/// Check if an account exists
pub async fn account_exists(pool: &SqlitePool, login: &str) -> Result<bool> {
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM accounts WHERE login = ? COLLATE NOCASE"
    )
    .bind(login)
    .fetch_one(pool)
    .await?;
    
    Ok(count.0 > 0)
}

/// Count total accounts
pub async fn count_accounts(pool: &SqlitePool) -> Result<i64> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM accounts")
        .fetch_one(pool)
        .await?;
    
    Ok(count.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    
    async fn test_db(name: &str) -> (Database, String) {
        let path = format!("/tmp/test_rhxd_accounts_{}_{}.db", name, std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
        let db = Database::new(&path).await.unwrap();
        db.init_schema().await.unwrap();
        (db, path)
    }
    
    #[tokio::test]
    async fn test_create_and_get_account() {
        let (db, path) = test_db("create").await;
        let pool = db.pool();
        
        // Create account
        let password = b"scrambled_password";
        let account_id = create_account(
            pool,
            "admin",
            password,
            "Administrator",
            AccessPrivileges::admin(),
        )
        .await
        .unwrap();
        
        assert!(account_id > 0);
        
        // Get by login
        let account = get_account_by_login(pool, "admin")
            .await
            .unwrap()
            .unwrap();
        
        assert_eq!(account.login, "admin");
        assert_eq!(account.name, "Administrator");
        assert_eq!(account.password_hash, password);
        assert!(account.has_privilege(AccessPrivileges::SEND_CHAT));
        
        // Get by ID
        let account2 = get_account_by_id(pool, account_id)
            .await
            .unwrap()
            .unwrap();
        
        assert_eq!(account2.login, account.login);
        
        std::fs::remove_file(&path).ok();
    }
    
    #[tokio::test]
    async fn test_account_exists() {
        let (db, path) = test_db("exists").await;
        let pool = db.pool();
        
        assert!(!account_exists(pool, "test").await.unwrap());
        
        create_account(
            pool,
            "test",
            b"password",
            "Test User",
            AccessPrivileges::user(),
        )
        .await
        .unwrap();
        
        assert!(account_exists(pool, "test").await.unwrap());
        assert!(account_exists(pool, "TEST").await.unwrap()); // Case insensitive
        
        std::fs::remove_file(&path).ok();
    }
    
    #[tokio::test]
    async fn test_list_accounts() {
        let (db, path) = test_db("list").await;
        let pool = db.pool();
        
        create_account(pool, "user1", b"pass1", "User 1", AccessPrivileges::user())
            .await
            .unwrap();
        create_account(pool, "user2", b"pass2", "User 2", AccessPrivileges::user())
            .await
            .unwrap();
        
        let accounts = list_accounts(pool).await.unwrap();
        assert_eq!(accounts.len(), 2);
        
        std::fs::remove_file(&path).ok();
    }
    
    #[tokio::test]
    async fn test_delete_account() {
        let (db, path) = test_db("delete").await;
        let pool = db.pool();
        
        let id = create_account(
            pool,
            "deleteme",
            b"password",
            "Delete Me",
            AccessPrivileges::user(),
        )
        .await
        .unwrap();
        
        assert!(account_exists(pool, "deleteme").await.unwrap());
        
        delete_account(pool, id).await.unwrap();
        
        assert!(!account_exists(pool, "deleteme").await.unwrap());
        
        std::fs::remove_file(&path).ok();
    }
}
