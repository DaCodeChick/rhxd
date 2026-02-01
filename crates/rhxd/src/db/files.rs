//! File metadata operations

#![allow(dead_code)] // Many functions are for future use

use anyhow::{bail, Result};
use chrono::Utc;
use sqlx::SqlitePool;
use std::path::PathBuf;

/// File entry record from database
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub id: i64,
    pub path: String,
    pub name: String,
    pub is_folder: bool,
    pub size: i64,
    pub type_code: Option<String>,
    pub creator_code: Option<String>,
    pub comment: Option<String>,
    pub created_at: i64,
    pub modified_at: i64,
    pub physical_path: String,
}

impl FileEntry {
    /// Get the parent path
    pub fn parent_path(&self) -> Option<String> {
        if self.path == "/" {
            return None;
        }
        
        let path = self.path.trim_end_matches('/');
        path.rfind('/')
            .map(|pos| path[..pos].to_string())
            .or(Some("/".to_string()))
    }
}

/// Create a file entry
pub async fn create_file_entry(
    pool: &SqlitePool,
    path: &str,
    name: &str,
    is_folder: bool,
    size: i64,
    type_code: Option<&str>,
    creator_code: Option<&str>,
    comment: Option<&str>,
    physical_path: &str,
) -> Result<i64> {
    // Validate input
    if name.len() > 255 {
        bail!("File name must be 255 characters or less");
    }
    
    if let Some(tc) = type_code {
        if tc.len() != 4 {
            bail!("Type code must be exactly 4 characters");
        }
    }
    
    if let Some(cc) = creator_code {
        if cc.len() != 4 {
            bail!("Creator code must be exactly 4 characters");
        }
    }
    
    let now = Utc::now().timestamp();
    
    let result = sqlx::query(
        "INSERT INTO files (path, name, is_folder, size, type_code, creator_code, comment, 
                           created_at, modified_at, physical_path)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(path)
    .bind(name)
    .bind(is_folder as i32)
    .bind(size)
    .bind(type_code)
    .bind(creator_code)
    .bind(comment)
    .bind(now)
    .bind(now)
    .bind(physical_path)
    .execute(pool)
    .await?;
    
    Ok(result.last_insert_rowid())
}

/// Get file entry by path
pub async fn get_file_by_path(pool: &SqlitePool, path: &str) -> Result<Option<FileEntry>> {
    let entry = sqlx::query_as::<_, (i64, String, String, i32, i64, Option<String>, 
                                     Option<String>, Option<String>, i64, i64, String)>(
        "SELECT id, path, name, is_folder, size, type_code, creator_code, comment,
                created_at, modified_at, physical_path
         FROM files WHERE path = ?"
    )
    .bind(path)
    .fetch_optional(pool)
    .await?;
    
    Ok(entry.map(|(id, path, name, is_folder, size, type_code, creator_code, comment,
                   created_at, modified_at, physical_path)| {
        FileEntry {
            id,
            path,
            name,
            is_folder: is_folder != 0,
            size,
            type_code,
            creator_code,
            comment,
            created_at,
            modified_at,
            physical_path,
        }
    }))
}

/// List files in a directory
pub async fn list_files_in_directory(pool: &SqlitePool, parent_path: &str) -> Result<Vec<FileEntry>> {
    // Normalize parent path
    let parent = if parent_path.is_empty() || parent_path == "/" {
        "/".to_string()
    } else {
        parent_path.trim_end_matches('/').to_string()
    };
    
    // For root, we want entries like "/filename" but not "/"
    // For "/folder", we want entries like "/folder/filename" but not "/folder"
    let pattern = if parent == "/" {
        format!("{}%", parent)
    } else {
        format!("{}/%%", parent)
    };
    
    let entries = sqlx::query_as::<_, (i64, String, String, i32, i64, Option<String>,
                                       Option<String>, Option<String>, i64, i64, String)>(
        "SELECT id, path, name, is_folder, size, type_code, creator_code, comment,
                created_at, modified_at, physical_path
         FROM files 
         WHERE path LIKE ? 
           AND path != ?
           AND path NOT LIKE ?
         ORDER BY is_folder DESC, name ASC"
    )
    .bind(&pattern)
    .bind(&parent)  // Exclude the parent directory itself
    .bind(format!("{}%/%", &pattern.trim_end_matches('%')))
    .fetch_all(pool)
    .await?;
    
    Ok(entries
        .into_iter()
        .map(|(id, path, name, is_folder, size, type_code, creator_code, comment,
               created_at, modified_at, physical_path)| {
            FileEntry {
                id,
                path,
                name,
                is_folder: is_folder != 0,
                size,
                type_code,
                creator_code,
                comment,
                created_at,
                modified_at,
                physical_path,
            }
        })
        .collect())
}

/// Update file metadata
pub async fn update_file_metadata(
    pool: &SqlitePool,
    file_id: i64,
    comment: Option<&str>,
    type_code: Option<&str>,
    creator_code: Option<&str>,
) -> Result<()> {
    let now = Utc::now().timestamp();
    
    sqlx::query(
        "UPDATE files SET comment = ?, type_code = ?, creator_code = ?, modified_at = ?
         WHERE id = ?"
    )
    .bind(comment)
    .bind(type_code)
    .bind(creator_code)
    .bind(now)
    .bind(file_id)
    .execute(pool)
    .await?;
    
    Ok(())
}

/// Delete a file entry
pub async fn delete_file_entry(pool: &SqlitePool, path: &str) -> Result<()> {
    sqlx::query("DELETE FROM files WHERE path = ? OR path LIKE ?")
        .bind(path)
        .bind(format!("{}/%", path.trim_end_matches('/')))
        .execute(pool)
        .await?;
    
    Ok(())
}

/// Check if a file exists
pub async fn file_exists(pool: &SqlitePool, path: &str) -> Result<bool> {
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM files WHERE path = ?"
    )
    .bind(path)
    .fetch_one(pool)
    .await?;
    
    Ok(count.0 > 0)
}

/// Index a physical directory into the database
pub async fn index_directory(
    pool: &SqlitePool,
    physical_root: &str,
    virtual_root: &str,
) -> Result<usize> {
    let mut count = 0;
    let physical_path = PathBuf::from(physical_root);
    
    if !physical_path.exists() {
        bail!("Physical path does not exist: {}", physical_root);
    }
    
    fn index_recursive(
        pool: &SqlitePool,
        physical_path: &PathBuf,
        virtual_path: &str,
        count: &mut usize,
    ) -> Result<()> {
        let entries = std::fs::read_dir(physical_path)?;
        
        for entry in entries {
            let entry = entry?;
            let metadata = entry.metadata()?;
            let file_name = entry.file_name().to_string_lossy().to_string();
            
            // Skip hidden files
            if file_name.starts_with('.') {
                continue;
            }
            
            let vpath = if virtual_path == "/" {
                format!("/{}", file_name)
            } else {
                format!("{}/{}", virtual_path, file_name)
            };
            
            let physical = entry.path().to_string_lossy().to_string();
            
            // Use tokio::task::block_in_place for async operation in sync context
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    create_file_entry(
                        pool,
                        &vpath,
                        &file_name,
                        metadata.is_dir(),
                        metadata.len() as i64,
                        None,
                        None,
                        None,
                        &physical,
                    )
                    .await
                })
            })?;
            
            *count += 1;
            
            // Recurse into directories
            if metadata.is_dir() {
                index_recursive(pool, &entry.path(), &vpath, count)?;
            }
        }
        
        Ok(())
    }
    
    index_recursive(pool, &physical_path, virtual_root, &mut count)?;
    
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;
    
    async fn test_db(name: &str) -> (Database, String) {
        let path = format!("/tmp/test_rhxd_files_{}_{}.db", name, std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
        let db = Database::new(&path).await.unwrap();
        db.init_schema().await.unwrap();
        (db, path)
    }
    
    #[tokio::test]
    async fn test_create_and_get_file() {
        let (db, path) = test_db("create").await;
        let pool = db.pool();
        
        let file_id = create_file_entry(
            pool,
            "/test.txt",
            "test.txt",
            false,
            1024,
            Some("TEXT"),
            Some("ttxt"),
            Some("A test file"),
            "/physical/test.txt",
        )
        .await
        .unwrap();
        
        assert!(file_id > 0);
        
        let file = get_file_by_path(pool, "/test.txt")
            .await
            .unwrap()
            .unwrap();
        
        assert_eq!(file.name, "test.txt");
        assert_eq!(file.size, 1024);
        assert!(!file.is_folder);
        assert_eq!(file.type_code, Some("TEXT".to_string()));
        
        std::fs::remove_file(&path).ok();
    }
    
    #[tokio::test]
    async fn test_list_files() {
        let (db, path) = test_db("list").await;
        let pool = db.pool();
        
        // Create some test files
        create_file_entry(pool, "/", "/", true, 0, None, None, None, "/physical")
            .await
            .unwrap();
        create_file_entry(pool, "/file1.txt", "file1.txt", false, 100, None, None, None, "/physical/file1.txt")
            .await
            .unwrap();
        create_file_entry(pool, "/file2.txt", "file2.txt", false, 200, None, None, None, "/physical/file2.txt")
            .await
            .unwrap();
        create_file_entry(pool, "/folder", "folder", true, 0, None, None, None, "/physical/folder")
            .await
            .unwrap();
        create_file_entry(pool, "/folder/nested.txt", "nested.txt", false, 50, None, None, None, "/physical/folder/nested.txt")
            .await
            .unwrap();
        
        // List root
        let root_files = list_files_in_directory(pool, "/").await.unwrap();
        assert_eq!(root_files.len(), 3); // file1, file2, folder (not the nested file)
        
        // List folder
        let folder_files = list_files_in_directory(pool, "/folder").await.unwrap();
        assert_eq!(folder_files.len(), 1); // nested.txt
        
        std::fs::remove_file(&path).ok();
    }
}
