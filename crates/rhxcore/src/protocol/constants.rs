//! Protocol constants

/// Protocol magic bytes: 'TRTP'
pub const PROTOCOL_MAGIC: [u8; 4] = [b'T', b'R', b'T', b'P'];

/// Protocol version
pub const PROTOCOL_VERSION: u16 = 1;

/// Server version we report (Hotline 1.9.x compatible)
/// Version 197 = Hotline 1.9.2
pub const SERVER_VERSION: u16 = 197;

/// HTXF protocol magic bytes: 'HTXF'
pub const HTXF_MAGIC: [u8; 4] = [b'H', b'T', b'X', b'F'];

/// Default base port
pub const DEFAULT_PORT: u16 = 5500;

/// Default tracker port
pub const DEFAULT_TRACKER_PORT: u16 = 5498;

/// Maximum transaction data size (32 KB)
pub const MAX_TRANSACTION_SIZE: usize = 32768;

/// Maximum field data size
pub const MAX_FIELD_SIZE: usize = 32768;

/// Maximum chat message size (8 KB per spec)
pub const MAX_CHAT_SIZE: usize = 8192;

/// Maximum user name size
pub const MAX_USERNAME_SIZE: usize = 31;

/// Maximum login size
pub const MAX_LOGIN_SIZE: usize = 31;

/// Maximum password size
pub const MAX_PASSWORD_SIZE: usize = 31;

/// Maximum file path size
pub const MAX_PATH_SIZE: usize = 2048;
