//! Transaction codec for encoding and decoding transactions

use crate::error::{ProtocolError, Result};
use crate::protocol::{Transaction, TransactionHeader, TransactionType};
use bytes::{Buf, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

/// Codec for encoding and decoding Hotline transactions
pub struct TransactionCodec {
    // Maximum transaction size to prevent DoS
    max_size: usize,
}

impl TransactionCodec {
    /// Create a new transaction codec
    pub fn new() -> Self {
        Self {
            max_size: crate::protocol::constants::MAX_TRANSACTION_SIZE,
        }
    }

    /// Create a new transaction codec with a custom max size
    pub fn with_max_size(max_size: usize) -> Self {
        Self { max_size }
    }
}

impl Default for TransactionCodec {
    fn default() -> Self {
        Self::new()
    }
}

impl Decoder for TransactionCodec {
    type Item = Transaction;
    type Error = ProtocolError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>> {
        // Need at least the header
        if src.len() < TransactionHeader::SIZE {
            return Ok(None);
        }

        // Parse header without consuming bytes
        let header = TransactionHeader::from_bytes(&src[..TransactionHeader::SIZE])?;

        // Check if we have the full transaction
        let total_needed = TransactionHeader::SIZE + header.data_size as usize;
        if src.len() < total_needed {
            // Reserve space for the full transaction
            src.reserve(total_needed - src.len());
            return Ok(None);
        }

        // Check max size
        if header.data_size as usize > self.max_size {
            return Err(ProtocolError::TransactionTooLarge {
                size: header.data_size as usize,
                max: self.max_size,
            });
        }

        // Now consume the bytes
        src.advance(TransactionHeader::SIZE);

        // Parse transaction type
        let transaction_type = TransactionType::from_u16(header.transaction_type).ok_or(
            ProtocolError::InvalidTransactionType(header.transaction_type),
        )?;

        // Parse fields
        let fields = if header.data_size > 0 {
            super::field_codec::decode_fields(&mut src.split_to(header.data_size as usize))?
        } else {
            Vec::new()
        };

        Ok(Some(Transaction {
            flags: header.flags,
            is_reply: header.is_reply != 0,
            transaction_type,
            id: header.id,
            error_code: header.error_code,
            total_size: header.total_size,
            data_size: header.data_size,
            fields,
        }))
    }
}

impl Encoder<Transaction> for TransactionCodec {
    type Error = ProtocolError;

    fn encode(&mut self, item: Transaction, dst: &mut BytesMut) -> Result<()> {
        // Encode fields first to know the size
        let mut fields_buf = BytesMut::new();
        super::field_codec::encode_fields(&item.fields, &mut fields_buf)?;

        let data_size = fields_buf.len() as u32;

        // Create header
        let header = TransactionHeader {
            flags: item.flags,
            is_reply: if item.is_reply { 1 } else { 0 },
            transaction_type: item.transaction_type.to_u16(),
            id: item.id,
            error_code: item.error_code,
            total_size: data_size, // For single-part transactions
            data_size,
        };

        // Encode header
        dst.reserve(TransactionHeader::SIZE + data_size as usize);
        header.to_bytes(dst);

        // Append fields
        dst.extend_from_slice(&fields_buf);

        Ok(())
    }
}
