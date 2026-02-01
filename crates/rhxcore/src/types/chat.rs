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
