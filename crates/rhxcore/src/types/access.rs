//! Access privileges

use serde::{Deserialize, Deserializer, Serialize, Serializer};

bitflags::bitflags! {
    /// Access privileges bitflags (64-bit)
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct AccessPrivileges: u64 {
        // File operations (0-9)
        const UPLOAD_FILES = 1 << 0;
        const DOWNLOAD_FILES = 1 << 1;
        const DELETE_FILES = 1 << 2;
        const RENAME_FILES = 1 << 3;
        const MOVE_FILES = 1 << 4;
        const CREATE_FOLDERS = 1 << 5;
        const DELETE_FOLDERS = 1 << 6;
        const RENAME_FOLDERS = 1 << 7;
        const MOVE_FOLDERS = 1 << 8;
        const READ_FILE_INFO = 1 << 9;

        // Chat operations (10-19)
        const READ_CHAT = 1 << 10;
        const SEND_CHAT = 1 << 11;
        const CREATE_PRIVATE_CHAT = 1 << 12;

        // News operations (20-29)
        const READ_NEWS = 1 << 13;
        const POST_NEWS = 1 << 14;
        const DELETE_NEWS = 1 << 15;
        const CREATE_NEWS_CATEGORY = 1 << 16;
        const DELETE_NEWS_CATEGORY = 1 << 17;
        const CREATE_NEWS_BUNDLE = 1 << 18;
        const DELETE_NEWS_BUNDLE = 1 << 19;

        // User management (20-29)
        const DISCONNECT_USERS = 1 << 20;
        const CANT_BE_DISCONNECTED = 1 << 21;
        const GET_USER_INFO = 1 << 22;
        const MODIFY_USERS = 1 << 23;
        const CREATE_USERS = 1 << 24;
        const DELETE_USERS = 1 << 25;
        const READ_USERS = 1 << 26;
        const SEND_MESSAGES = 1 << 27;

        // Special privileges (30-39)
        const BROADCAST = 1 << 28;
        const INVISIBILITY = 1 << 29;
        const CHANGE_OWN_PASSWORD = 1 << 30;
        const SEND_PRIVATE_MESSAGES = 1 << 31;

        // Download/upload management (32-39)
        const UPLOAD_ANYWHERE = 1 << 32;
        const ANY_NAME = 1 << 33;
        const NO_AGREEMENT = 1 << 34;
        const SET_FILE_COMMENT = 1 << 35;
        const SET_FOLDER_COMMENT = 1 << 36;
        const VIEW_DROP_BOXES = 1 << 37;
        const MAKE_ALIASES = 1 << 38;
        const DOWNLOAD_FOLDERS = 1 << 39;

        // Account management (40-49)
        const MODIFY_ACCOUNT = 1 << 40;
        const DELETE_ACCOUNT = 1 << 41;
        const CREATE_ACCOUNT = 1 << 42;
        const READ_ACCOUNTS = 1 << 43;

        // News extras (44-49)
        const READ_NEWS_ARTICLES = 1 << 44;
        const DELETE_NEWS_ARTICLES = 1 << 45;

        // File extras (50-59)
        const UPLOAD_FOLDERS = 1 << 50;
    }
}

impl AccessPrivileges {
    /// Full admin access
    pub fn admin() -> Self {
        Self::all()
    }

    /// Guest access (read-only)
    pub fn guest() -> Self {
        Self::READ_CHAT | Self::READ_NEWS | Self::DOWNLOAD_FILES
    }

    /// Regular user access
    pub fn user() -> Self {
        Self::READ_CHAT
            | Self::SEND_CHAT
            | Self::CREATE_PRIVATE_CHAT
            | Self::READ_NEWS
            | Self::DOWNLOAD_FILES
            | Self::UPLOAD_FILES
            | Self::SEND_MESSAGES
            | Self::SEND_PRIVATE_MESSAGES
    }
}

impl Default for AccessPrivileges {
    fn default() -> Self {
        Self::user()
    }
}

// Manual serde implementation for bitflags
impl Serialize for AccessPrivileges {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(self.bits())
    }
}

impl<'de> Deserialize<'de> for AccessPrivileges {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bits = u64::deserialize(deserializer)?;
        Ok(AccessPrivileges::from_bits_truncate(bits))
    }
}
