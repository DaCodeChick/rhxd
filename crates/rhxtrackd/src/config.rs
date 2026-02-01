//! Tracker configuration

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub http: HttpConfig,
    pub database: DatabaseConfig,
    pub registry: RegistryConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub name: String,
    pub address: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpConfig {
    pub enabled: bool,
    pub address: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    pub server_ttl_seconds: u64,
    pub cleanup_interval_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file: PathBuf,
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
                name: "My Hotline Tracker".to_string(),
                address: "0.0.0.0".to_string(),
                port: 5498,
            },
            http: HttpConfig {
                enabled: true,
                address: "0.0.0.0".to_string(),
                port: 8080,
            },
            database: DatabaseConfig {
                path: PathBuf::from("./rhxtrackd.db"),
            },
            registry: RegistryConfig {
                server_ttl_seconds: 3600,
                cleanup_interval_seconds: 300,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file: PathBuf::from("./logs/rhxtrackd.log"),
            },
        }
    }
}
