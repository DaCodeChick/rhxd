//! File types

use std::path::PathBuf;

/// File entry information
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub physical_path: PathBuf,
    pub size: i64,
    pub type_code: Option<[u8; 4]>,
    pub creator_code: Option<[u8; 4]>,
    pub comment: Option<String>,
    pub is_folder: bool,
}

impl FileEntry {
    pub fn new(name: String, path: String) -> Self {
        Self {
            name,
            path,
            physical_path: PathBuf::new(),
            size: 0,
            type_code: None,
            creator_code: None,
            comment: None,
            is_folder: false,
        }
    }
}
