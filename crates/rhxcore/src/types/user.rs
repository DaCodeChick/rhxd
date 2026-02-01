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
    /// User flags
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct UserFlags: u16 {
        const AWAY = 1 << 0;
        const ADMIN = 1 << 1;
        const REFUSED_MESSAGES = 1 << 2;
        const REFUSED_CHAT = 1 << 3;
    }
}
