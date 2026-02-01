//! User types

/// User information
#[derive(Debug, Clone)]
pub struct User {
    pub id: u16,
    pub icon_id: i16,
    pub flags: u16,
    pub name: String,
}

impl User {
    pub fn new(id: u16, name: String) -> Self {
        Self {
            id,
            icon_id: 0,
            flags: 0,
            name,
        }
    }
}

bitflags::bitflags! {
    /// User flags (sent in user list broadcasts)
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct UserFlags: u16 {
        const AWAY = 1 << 0;
        const ADMIN = 1 << 1;
        const REFUSED_MESSAGES = 1 << 2;
        const REFUSED_CHAT = 1 << 3;
    }
}

bitflags::bitflags! {
    /// User options (sent with Agreed transaction, field 113)
    ///
    /// These are user preferences that control how they want to interact
    /// with other users on the server.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct UserOptions: u16 {
        /// User refuses private messages (bit 0)
        const REFUSE_PRIVATE_MESSAGE = 1 << 0;
        /// User refuses private chat invitations (bit 1)
        const REFUSE_PRIVATE_CHAT = 1 << 1;
        /// User has automatic response enabled (bit 2)
        const AUTOMATIC_RESPONSE = 1 << 2;
    }
}

impl Default for UserOptions {
    fn default() -> Self {
        Self::empty()
    }
}

impl UserOptions {
    /// Create from i16 value (for protocol compatibility)
    pub fn from_i16(value: i16) -> Self {
        Self::from_bits_truncate(value as u16)
    }

    /// Convert to i16 value (for protocol compatibility)
    pub fn to_i16(&self) -> i16 {
        self.bits() as i16
    }

    /// Convert UserOptions to UserFlags for broadcasting
    ///
    /// Maps the user's preferences to the corresponding flags that
    /// other users see in the user list.
    pub fn to_user_flags(&self) -> u16 {
        let mut flags = 0u16;

        if self.contains(Self::REFUSE_PRIVATE_MESSAGE) {
            flags |= UserFlags::REFUSED_MESSAGES.bits();
        }
        if self.contains(Self::REFUSE_PRIVATE_CHAT) {
            flags |= UserFlags::REFUSED_CHAT.bits();
        }

        flags
    }
}
