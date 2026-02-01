//! Configuration management

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub files: FilesConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
    pub security: SecurityConfig,
    pub features: FeaturesConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub name: String,
    pub description: String,
    pub address: String,
    pub port: u16,
    pub max_connections: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesConfig {
    pub root_path: PathBuf,
    pub max_download_size: u64,
    pub enable_uploads: bool,
    pub enable_downloads: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub require_login: bool,
    pub allow_guest: bool,
    pub ban_list_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeaturesConfig {
    pub enable_news: bool,
    pub enable_private_chat: bool,
    pub enable_file_transfers: bool,
}

impl Config {
    /// Load configuration from a file
    pub fn load(path: impl AsRef<std::path::Path>) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// Save configuration to a file
    pub fn save(&self, path: impl AsRef<std::path::Path>) -> anyhow::Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Create default configuration
    pub fn default() -> Self {
        Self {
            server: ServerConfig {
                name: "My Hotline Server".to_string(),
                description: "A modern Rust Hotline server".to_string(),
                address: "0.0.0.0".to_string(),
                port: 5500,
                max_connections: 100,
            },
            files: FilesConfig {
                root_path: PathBuf::from("./files"),
                max_download_size: 104857600, // 100 MB
                enable_uploads: true,
                enable_downloads: true,
            },
            database: DatabaseConfig {
                path: PathBuf::from("./rhxd.db"),
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file: PathBuf::from("./logs/rhxd.log"),
            },
            security: SecurityConfig {
                require_login: true,
                allow_guest: false,
                ban_list_path: PathBuf::from("./banlist.txt"),
            },
            features: FeaturesConfig {
                enable_news: false,
                enable_private_chat: true,
                enable_file_transfers: false,
            },
        }
    }
}
