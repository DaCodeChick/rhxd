//! # rhxcore
//!
//! Core protocol library for Hotline Connect implementations.
//!
//! This library provides the fundamental building blocks for implementing
//! Hotline Connect clients, servers, bots, and other tools:
//!
//! - Protocol types and constants (transactions, fields, error codes)
//! - Encoding/decoding (codec implementation for transactions and fields)
//! - Type-safe protocol structures
//! - Legacy password handling
//!
//! ## Example
//!
//! ```rust,no_run
//! use rhxcore::protocol::{Transaction, TransactionType, Field, FieldId};
//!
//! // Create a login transaction
//! let mut transaction = Transaction::new(TransactionType::Login);
//! transaction.add_field(Field::string(FieldId::UserLogin, "mylogin"));
//! transaction.add_field(Field::binary(FieldId::UserPassword, b"scrambled_password"));
//! ```

pub mod protocol;
pub mod codec;
pub mod types;
pub mod password;
pub mod error;

// Re-export commonly used types
pub use error::{ProtocolError, Result};
pub use protocol::{Transaction, TransactionType, Field, FieldId};
pub use types::access::AccessPrivileges;
