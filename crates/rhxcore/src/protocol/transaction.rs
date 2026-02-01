//! Transaction types and structures

use super::field::Field;
use super::types::TransactionType;
use bytes::{Buf, BufMut};

/// A Hotline protocol transaction
#[derive(Debug, Clone)]
pub struct Transaction {
    /// Reserved flags (should be 0)
    pub flags: u8,

    /// Request (false) or reply (true)
    pub is_reply: bool,

    /// Transaction type
    pub transaction_type: TransactionType,

    /// Unique transaction ID (must not be 0)
    pub id: u32,

    /// Error code (used in replies, 0 = no error)
    pub error_code: u32,

    /// Total size of all transaction parts
    pub total_size: u32,

    /// Size of data in this transaction part
    pub data_size: u32,

    /// Transaction fields
    pub fields: Vec<Field>,
}

impl Transaction {
    /// Create a new request transaction
    pub fn new(transaction_type: TransactionType) -> Self {
        Self {
            flags: 0,
            is_reply: false,
            transaction_type,
            id: 0, // Will be set by caller
            error_code: 0,
            total_size: 0,
            data_size: 0,
            fields: Vec::new(),
        }
    }

    /// Create a new reply transaction
    pub fn new_reply(transaction_type: TransactionType, id: u32) -> Self {
        Self {
            flags: 0,
            is_reply: true,
            transaction_type,
            id,
            error_code: 0,
            total_size: 0,
            data_size: 0,
            fields: Vec::new(),
        }
    }

    /// Create an error reply
    pub fn new_error(id: u32, error_code: u32) -> Self {
        Self {
            flags: 0,
            is_reply: true,
            transaction_type: TransactionType::Error,
            id,
            error_code,
            total_size: 0,
            data_size: 0,
            fields: Vec::new(),
        }
    }

    /// Add a field to the transaction
    pub fn add_field(&mut self, field: Field) {
        self.fields.push(field);
    }

    /// Get a field by ID
    pub fn get_field(&self, id: super::field::FieldId) -> Option<&Field> {
        self.fields.iter().find(|f| f.id == id)
    }

    /// Check if transaction has a specific field
    pub fn has_field(&self, id: super::field::FieldId) -> bool {
        self.get_field(id).is_some()
    }
}

/// Transaction header structure (20 bytes)
#[derive(Debug, Clone, Copy)]
pub struct TransactionHeader {
    pub flags: u8,
    pub is_reply: u8,
    pub transaction_type: u16,
    pub id: u32,
    pub error_code: u32,
    pub total_size: u32,
    pub data_size: u32,
}

impl TransactionHeader {
    /// Size of the transaction header in bytes
    pub const SIZE: usize = 20;

    /// Parse a transaction header from bytes
    pub fn from_bytes(mut buf: &[u8]) -> Result<Self, std::io::Error> {
        if buf.len() < Self::SIZE {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "Not enough bytes for transaction header",
            ));
        }

        Ok(Self {
            flags: buf.get_u8(),
            is_reply: buf.get_u8(),
            transaction_type: buf.get_u16(),
            id: buf.get_u32(),
            error_code: buf.get_u32(),
            total_size: buf.get_u32(),
            data_size: buf.get_u32(),
        })
    }

    /// Encode the transaction header to bytes
    pub fn to_bytes(&self, buf: &mut impl BufMut) {
        buf.put_u8(self.flags);
        buf.put_u8(self.is_reply);
        buf.put_u16(self.transaction_type);
        buf.put_u32(self.id);
        buf.put_u32(self.error_code);
        buf.put_u32(self.total_size);
        buf.put_u32(self.data_size);
    }
}
