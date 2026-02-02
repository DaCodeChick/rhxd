//! Access privileges

use serde::{Deserialize, Deserializer, Serialize, Serializer};

bitflags::bitflags! {
    /// Access privileges bitflags (64-bit)
    /// Bit positions match GLoarbLine/mhxd for compatibility
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct AccessPrivileges: u64 {
        // File operations (bits 0-8)
        const DELETE_FILES = 1 << 0;           // myAcc_DeleteFile
        const UPLOAD_FILES = 1 << 1;           // myAcc_UploadFile
        const DOWNLOAD_FILES = 1 << 2;         // myAcc_DownloadFile
        const RENAME_FILES = 1 << 3;           // myAcc_RenameFile
        const MOVE_FILES = 1 << 4;             // myAcc_MoveFile
        const CREATE_FOLDERS = 1 << 5;         // myAcc_CreateFolder
        const DELETE_FOLDERS = 1 << 6;         // myAcc_DeleteFolder
        const RENAME_FOLDERS = 1 << 7;         // myAcc_RenameFolder
        const MOVE_FOLDERS = 1 << 8;           // myAcc_MoveFolder

        // Chat operations (bits 9-12)
        const READ_CHAT = 1 << 9;              // myAcc_ReadChat
        const SEND_CHAT = 1 << 10;             // myAcc_SendChat
        const CREATE_PRIVATE_CHAT = 1 << 11;   // myAcc_CreateChat
        const CLOSE_CHAT = 1 << 12;            // myAcc_CloseChat

        // User visibility (bit 13)
        const SHOW_IN_LIST = 1 << 13;          // myAcc_ShowInList

        // User management (bits 14-19)
        const CREATE_USERS = 1 << 14;          // myAcc_CreateUser
        const DELETE_USERS = 1 << 15;          // myAcc_DeleteUser
        const OPEN_USER = 1 << 16;             // myAcc_OpenUser
        const MODIFY_USERS = 1 << 17;          // myAcc_ModifyUser
        const CHANGE_OWN_PASSWORD = 1 << 18;   // myAcc_ChangeOwnPass
        const SEND_PRIVATE_MESSAGES = 1 << 19; // myAcc_SendPrivMsg

        // News operations (bits 20-21, 33-37)
        const READ_NEWS = 1 << 20;             // myAcc_NewsReadArt
        const POST_NEWS = 1 << 21;             // myAcc_NewsPostArt
        const DELETE_NEWS = 1 << 33;           // myAcc_NewsDeleteArt
        const CREATE_NEWS_CATEGORY = 1 << 34;  // myAcc_NewsCreateCat
        const DELETE_NEWS_CATEGORY = 1 << 35;  // myAcc_NewsDeleteCat
        const CREATE_NEWS_BUNDLE = 1 << 36;    // myAcc_NewsCreateFldr
        const DELETE_NEWS_BUNDLE = 1 << 37;    // myAcc_NewsDeleteFldr

        // Admin operations (bits 22-24)
        const DISCONNECT_USERS = 1 << 22;      // myAcc_DisconUser
        const CANT_BE_DISCONNECTED = 1 << 23;  // myAcc_CannotBeDiscon
        const GET_USER_INFO = 1 << 24;         // myAcc_GetClientInfo

        // Special file privileges (bits 25-31)
        const UPLOAD_ANYWHERE = 1 << 25;       // myAcc_UploadAnywhere
        const ANY_NAME = 1 << 26;              // myAcc_AnyName
        const NO_AGREEMENT = 1 << 27;          // myAcc_NoAgreement
        const SET_FILE_COMMENT = 1 << 28;      // myAcc_SetFileComment
        const SET_FOLDER_COMMENT = 1 << 29;    // myAcc_SetFolderComment
        const VIEW_DROP_BOXES = 1 << 30;       // myAcc_ViewDropBoxes
        const MAKE_ALIASES = 1 << 31;          // myAcc_MakeAlias

        // More operations (bits 32+)
        const BROADCAST = 1 << 32;             // myAcc_Broadcast
        const UPLOAD_FOLDERS = 1 << 38;        // myAcc_UploadFolder
        const DOWNLOAD_FOLDERS = 1 << 39;      // myAcc_DownloadFolder
        const SEND_MESSAGES = 1 << 40;         // myAcc_SendMessage

        // Extended features (bits 41+)
        const FAKE_RED = 1 << 41;              // myAcc_FakeRed
        const AWAY = 1 << 42;                  // myAcc_Away
        const CHANGE_NICK = 1 << 43;           // myAcc_ChangeNick
        const CHANGE_ICON = 1 << 44;           // myAcc_ChangeIcon
        const SPEAK_BEFORE = 1 << 45;          // myAcc_SpeakBefore
        const REFUSE_CHAT = 1 << 46;           // myAcc_RefuseChat
        const BLOCK_DOWNLOAD = 1 << 47;        // myAcc_BlockDownload
        const VISIBLE = 1 << 48;               // myAcc_Visible
        const CAN_VIEW_INVISIBLE = 1 << 49;    // myAcc_Canviewinvisible
    }
}

impl AccessPrivileges {
    /// System operator access (highest level - all privileges including can't be disconnected)
    /// Sysops have complete control and cannot be disconnected by anyone
    pub fn sysop() -> Self {
        Self::all()
    }

    /// Admin access (all privileges except can't be disconnected)
    /// Admins can manage users, files, news, and moderate chat, but sysops can disconnect them
    pub fn admin() -> Self {
        Self::all() & !Self::CANT_BE_DISCONNECTED
    }

    /// Guest access (read-only with chat)
    pub fn guest() -> Self {
        Self::READ_CHAT | Self::SEND_CHAT | Self::READ_NEWS | Self::DOWNLOAD_FILES
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

    /// Parse a preset name into AccessPrivileges
    pub fn from_preset(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "sysop" => Some(Self::sysop()),
            "admin" => Some(Self::admin()),
            "user" => Some(Self::user()),
            "guest" => Some(Self::guest()),
            _ => None,
        }
    }

    /// Get the preset name for these privileges (if it matches exactly)
    pub fn preset_name(&self) -> Option<&'static str> {
        if *self == Self::sysop() {
            Some("sysop")
        } else if *self == Self::admin() {
            Some("admin")
        } else if *self == Self::user() {
            Some("user")
        } else if *self == Self::guest() {
            Some("guest")
        } else {
            None
        }
    }

    /// Encode access privileges to wire format (8 bytes)
    ///
    /// Note: The Hotline protocol transmits access privileges in the system's
    /// native byte order (little-endian on x86/x86_64), with bits reversed within
    /// each byte on little-endian systems. See docs/ACCESS_BITS.md for details.
    pub fn to_wire_format(&self) -> [u8; 8] {
        #[cfg(target_endian = "little")]
        {
            // On little-endian: use native byte order and reverse bits within each byte
            let mut bytes = self.bits().to_le_bytes();
            for byte in &mut bytes {
                *byte = byte.reverse_bits();
            }
            bytes
        }
        #[cfg(target_endian = "big")]
        {
            // Big-endian systems: direct conversion to bytes (no reversal needed)
            self.bits().to_be_bytes()
        }
    }

    /// Decode access privileges from wire format (8 bytes)
    ///
    /// Reverses the bit-reversal applied during encoding on little-endian systems.
    pub fn from_wire_format(bytes: [u8; 8]) -> Self {
        #[cfg(target_endian = "little")]
        {
            let mut bytes = bytes;
            // Reverse bits within each byte for little-endian systems
            for byte in &mut bytes {
                *byte = byte.reverse_bits();
            }
            let bits = u64::from_le_bytes(bytes);
            Self::from_bits_truncate(bits)
        }
        #[cfg(target_endian = "big")]
        {
            let bits = u64::from_be_bytes(bytes);
            Self::from_bits_truncate(bits)
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wire_format_roundtrip() {
        // Test that encoding and decoding preserves the access bits
        let original = AccessPrivileges::admin();
        let wire = original.to_wire_format();
        let decoded = AccessPrivileges::from_wire_format(wire);
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_guest_access() {
        let guest = AccessPrivileges::guest();
        assert!(guest.contains(AccessPrivileges::READ_CHAT));
        assert!(guest.contains(AccessPrivileges::SEND_CHAT));
        assert!(guest.contains(AccessPrivileges::READ_NEWS));
        assert!(guest.contains(AccessPrivileges::DOWNLOAD_FILES));
        assert!(!guest.contains(AccessPrivileges::UPLOAD_FILES));
    }

    #[test]
    #[cfg(target_endian = "little")]
    fn test_bit_reversal_little_endian() {
        // On little-endian systems, test that bits are reversed within each byte
        // Bit 0 (delete_files) should end up in bit 7 of byte 0
        let access = AccessPrivileges::DELETE_FILES;
        let wire = access.to_wire_format();

        // Debug: print the wire format
        println!("Wire format for DELETE_FILES: {:02X?}", wire);
        println!("Bits value: 0x{:016X}", access.bits());

        // Bit 0 in little-endian byte order goes to byte 0
        // After bit reversal: bit 0 becomes bit 7
        // Result: byte 0 = 0x80
        assert_eq!(wire[0], 0x80, "Bit 0 should be in byte 0, bit 7 (0x80)");
    }

    #[test]
    #[cfg(target_endian = "little")]
    fn test_multiple_bits_little_endian() {
        // Test multiple bits in the first byte
        let access = AccessPrivileges::DELETE_FILES
            | AccessPrivileges::UPLOAD_FILES
            | AccessPrivileges::DOWNLOAD_FILES;
        // Bits 0, 1, 2 in little-endian go to byte 0
        // After bit reversal: bits 7, 6, 5
        // Wire format: 0xE0 (11100000) in byte 0
        let wire = access.to_wire_format();
        assert_eq!(wire[0], 0xE0, "Bits 0,1,2 should be in byte 0 as 0xE0");
    }

    #[test]
    #[cfg(target_endian = "big")]
    fn test_no_reversal_big_endian() {
        // On big-endian systems, no bit reversal should occur
        let access = AccessPrivileges::DELETE_FILES;
        let wire = access.to_wire_format();

        // Bit 0 should remain in position 0 of byte 0 (LSB of first byte)
        // In big-endian wire format, this is: 0x01 00 00 00 00 00 00 00
        assert_eq!(
            wire[0], 0x01,
            "Bit 0 should remain at position 0 on big-endian"
        );
    }
}
