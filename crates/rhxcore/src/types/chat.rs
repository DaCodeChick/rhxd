//! Chat room types

/// Chat room information
#[derive(Debug, Clone)]
pub struct ChatRoom {
    pub id: u32,
    pub subject: Option<String>,
    pub users: Vec<u16>, // User IDs
}

impl ChatRoom {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            subject: None,
            users: Vec::new(),
        }
    }
}

bitflags::bitflags! {
    /// Chat message options
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ChatOptions: u16 {
        /// Normal chat message (default)
        const NORMAL = 0;
        /// Emote/action message (e.g., "/me does something")
        const EMOTE = 1 << 0;
    }
}

impl Default for ChatOptions {
    fn default() -> Self {
        Self::NORMAL
    }
}

impl ChatOptions {
    /// Check if this is an emote message
    pub fn is_emote(&self) -> bool {
        self.contains(Self::EMOTE)
    }

    /// Check if this is a normal message
    pub fn is_normal(&self) -> bool {
        !self.is_emote()
    }

    /// Create from i16 value (for protocol compatibility)
    pub fn from_i16(value: i16) -> Self {
        Self::from_bits_truncate(value as u16)
    }

    /// Convert to i16 value (for protocol compatibility)
    pub fn to_i16(&self) -> i16 {
        self.bits() as i16
    }
}
