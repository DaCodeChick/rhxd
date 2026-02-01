//! Error types

use thiserror::Error;

/// Protocol errors
#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("Invalid transaction type: {0}")]
    InvalidTransactionType(u16),

    #[error("Invalid field ID: {0}")]
    InvalidFieldId(u16),

    #[error("Protocol version mismatch")]
    VersionMismatch,

    #[error("Invalid handshake")]
    InvalidHandshake,

    #[error("Transaction too large: {size} bytes (max: {max})")]
    TransactionTooLarge { size: usize, max: usize },

    #[error("Invalid field data")]
    InvalidFieldData,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}

/// Result type for protocol operations
pub type Result<T> = std::result::Result<T, ProtocolError>;
