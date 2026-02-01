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

/// Chat message type
///
/// Field 109 in the Hotline protocol indicates the chat message type:
/// - false (0): Normal chat message
/// - true (1): Emote/action message (e.g., "/me does something")
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ChatOptions {
    /// True if this is an emote message, false for normal chat
    pub is_emote: bool,
}

impl ChatOptions {
    /// Normal chat message (field 109 = 0)
    pub const NORMAL: Self = Self { is_emote: false };

    /// Emote/action message (field 109 = 1)
    pub const EMOTE: Self = Self { is_emote: true };

    /// Create from protocol field value
    pub fn from_i16(value: i16) -> Self {
        Self {
            is_emote: value == 1,
        }
    }

    /// Convert to protocol field value
    pub fn to_i16(&self) -> i16 {
        if self.is_emote {
            1
        } else {
            0
        }
    }
}
