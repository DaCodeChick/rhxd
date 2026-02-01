//! Handshake structures

use super::constants::PROTOCOL_MAGIC;
use bytes::{Buf, BufMut};

/// Client handshake (12 bytes)
#[derive(Debug, Clone)]
pub struct Handshake {
    /// Protocol ID: 'TRTP' (0x54525450)
    pub protocol_id: [u8; 4],

    /// Sub-protocol ID (user defined)
    pub sub_protocol_id: u32,

    /// Version (currently 1)
    pub version: u16,

    /// Sub-version (user defined)
    pub sub_version: u16,
}

impl Handshake {
    pub const SIZE: usize = 12;

    /// Create a new handshake
    pub fn new() -> Self {
        Self {
            protocol_id: PROTOCOL_MAGIC,
            sub_protocol_id: 0,
            version: 1,
            sub_version: 2,
        }
    }

    /// Parse from bytes
    pub fn from_bytes(mut buf: &[u8]) -> Result<Self, std::io::Error> {
        if buf.len() < Self::SIZE {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "Not enough bytes for handshake",
            ));
        }

        let mut protocol_id = [0u8; 4];
        buf.copy_to_slice(&mut protocol_id);

        Ok(Self {
            protocol_id,
            sub_protocol_id: buf.get_u32(),
            version: buf.get_u16(),
            sub_version: buf.get_u16(),
        })
    }

    /// Encode to bytes
    pub fn to_bytes(&self, buf: &mut impl BufMut) {
        buf.put_slice(&self.protocol_id);
        buf.put_u32(self.sub_protocol_id);
        buf.put_u16(self.version);
        buf.put_u16(self.sub_version);
    }

    /// Validate the handshake
    pub fn is_valid(&self) -> bool {
        self.protocol_id == PROTOCOL_MAGIC
    }
}

impl Default for Handshake {
    fn default() -> Self {
        Self::new()
    }
}

/// Server handshake reply (8 bytes)
#[derive(Debug, Clone)]
pub struct HandshakeReply {
    /// Protocol ID: 'TRTP' (0x54525450)
    pub protocol_id: [u8; 4],

    /// Error code (0 = no error)
    pub error_code: u32,
}

impl HandshakeReply {
    pub const SIZE: usize = 8;

    /// Create a successful handshake reply
    pub fn new() -> Self {
        Self {
            protocol_id: PROTOCOL_MAGIC,
            error_code: 0,
        }
    }

    /// Create an error handshake reply
    pub fn error(code: u32) -> Self {
        Self {
            protocol_id: PROTOCOL_MAGIC,
            error_code: code,
        }
    }

    /// Parse from bytes
    pub fn from_bytes(mut buf: &[u8]) -> Result<Self, std::io::Error> {
        if buf.len() < Self::SIZE {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "Not enough bytes for handshake reply",
            ));
        }

        let mut protocol_id = [0u8; 4];
        buf.copy_to_slice(&mut protocol_id);

        Ok(Self {
            protocol_id,
            error_code: buf.get_u32(),
        })
    }

    /// Encode to bytes
    pub fn to_bytes(&self, buf: &mut impl BufMut) {
        buf.put_slice(&self.protocol_id);
        buf.put_u32(self.error_code);
    }

    /// Check if handshake was successful
    pub fn is_success(&self) -> bool {
        self.error_code == 0
    }
}

impl Default for HandshakeReply {
    fn default() -> Self {
        Self::new()
    }
}
