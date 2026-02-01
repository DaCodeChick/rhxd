//! Database schema utilities

/// Current schema version
pub const SCHEMA_VERSION: &str = "1";

/// Schema SQL is embedded from schema.sql file
pub const SCHEMA_SQL: &str = include_str!("schema.sql");
