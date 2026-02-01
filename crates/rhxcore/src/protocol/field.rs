//! Field types and structures

use bytes::{Buf, BufMut};

/// Field identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum FieldId {
    // Data
    Data = 101,
    UserName = 102,
    UserId = 103,
    UserIconId = 104,
    UserLogin = 105,
    UserPassword = 106,
    ReferenceNumber = 107,
    TransferSize = 108,
    ChatOptions = 109,

    // User access
    UserAccess = 110,
    UserAlias = 111,
    UserFlags = 112,
    Options = 113,
    ChatId = 114,
    ChatSubject = 115,
    WaitingCount = 116,

    // File fields
    FileName = 201,
    FilePath = 202,
    FileResumeData = 203,
    FileTransferOptions = 204,
    FileTypeString = 205,
    FileCreatorString = 206,
    FileSize = 207,
    FileCreateDate = 208,
    FileModifyDate = 209,
    FileComment = 210,
    FileNewName = 211,
    FileNewPath = 212,
    FileType = 213,

    // Quoting
    QuotingMsg = 214,
    AutomaticResponse = 215,

    // Server info
    ServerAgreement = 151,
    ServerBanner = 152,
    ServerBannerType = 153,
    ServerBannerUrl = 154,
    NoServerAgreement = 155,

    // Version and protocol
    Version = 160,
    BannerId = 161,
    ServerName = 162,

    // File name with info (special compound field)
    FileNameWithInfo = 200,
    UserNameWithInfo = 300,

    // News fields
    NewsArticleId = 320,
    NewsArticleDataFlavor = 321,
    NewsArticleTitle = 322,
    NewsArticlePoster = 323,
    NewsArticleDate = 324,
    NewsArticlePrevArt = 325,
    NewsArticleNextArt = 326,
    NewsArticleData = 327,
    NewsArticleFlags = 328,
    NewsArticleParentArt = 329,
    NewsArticle1stChildArt = 330,

    NewsCategoryGuid = 331,
    NewsCategoryListData = 332,
    NewsCategoryName = 333,
    NewsPath = 335,

    // HOPE extensions (for future use)
    SessionKey = 3587,
    MacAlg = 3588,
    ServerCipherAlg = 3771,
    ClientCipherAlg = 3772,
}

impl FieldId {
    /// Convert from u16
    pub fn from_u16(value: u16) -> Option<Self> {
        match value {
            101 => Some(Self::Data),
            102 => Some(Self::UserName),
            103 => Some(Self::UserId),
            104 => Some(Self::UserIconId),
            105 => Some(Self::UserLogin),
            106 => Some(Self::UserPassword),
            107 => Some(Self::ReferenceNumber),
            108 => Some(Self::TransferSize),
            109 => Some(Self::ChatOptions),
            110 => Some(Self::UserAccess),
            111 => Some(Self::UserAlias),
            112 => Some(Self::UserFlags),
            113 => Some(Self::Options),
            114 => Some(Self::ChatId),
            115 => Some(Self::ChatSubject),
            116 => Some(Self::WaitingCount),
            151 => Some(Self::ServerAgreement),
            152 => Some(Self::ServerBanner),
            153 => Some(Self::ServerBannerType),
            154 => Some(Self::ServerBannerUrl),
            155 => Some(Self::NoServerAgreement),
            160 => Some(Self::Version),
            161 => Some(Self::BannerId),
            162 => Some(Self::ServerName),
            200 => Some(Self::FileNameWithInfo),
            201 => Some(Self::FileName),
            202 => Some(Self::FilePath),
            203 => Some(Self::FileResumeData),
            204 => Some(Self::FileTransferOptions),
            205 => Some(Self::FileTypeString),
            206 => Some(Self::FileCreatorString),
            207 => Some(Self::FileSize),
            208 => Some(Self::FileCreateDate),
            209 => Some(Self::FileModifyDate),
            210 => Some(Self::FileComment),
            211 => Some(Self::FileNewName),
            212 => Some(Self::FileNewPath),
            213 => Some(Self::FileType),
            214 => Some(Self::QuotingMsg),
            215 => Some(Self::AutomaticResponse),
            300 => Some(Self::UserNameWithInfo),
            320 => Some(Self::NewsArticleId),
            321 => Some(Self::NewsArticleDataFlavor),
            322 => Some(Self::NewsArticleTitle),
            323 => Some(Self::NewsArticlePoster),
            324 => Some(Self::NewsArticleDate),
            325 => Some(Self::NewsArticlePrevArt),
            326 => Some(Self::NewsArticleNextArt),
            327 => Some(Self::NewsArticleData),
            328 => Some(Self::NewsArticleFlags),
            329 => Some(Self::NewsArticleParentArt),
            330 => Some(Self::NewsArticle1stChildArt),
            331 => Some(Self::NewsCategoryGuid),
            332 => Some(Self::NewsCategoryListData),
            333 => Some(Self::NewsCategoryName),
            335 => Some(Self::NewsPath),
            3587 => Some(Self::SessionKey),
            3588 => Some(Self::MacAlg),
            3771 => Some(Self::ServerCipherAlg),
            3772 => Some(Self::ClientCipherAlg),
            _ => None,
        }
    }

    /// Convert to u16
    pub fn to_u16(self) -> u16 {
        self as u16
    }
}

/// Field data types
#[derive(Debug, Clone, PartialEq)]
pub enum FieldData {
    /// Integer data (signed 32-bit for now)
    Integer(i32),
    /// String data (UTF-8)
    String(String),
    /// Binary data
    Binary(Vec<u8>),
}

/// A field in a transaction
#[derive(Debug, Clone)]
pub struct Field {
    pub id: FieldId,
    pub data: FieldData,
}

impl Field {
    /// Create a new integer field
    pub fn integer(id: FieldId, value: i32) -> Self {
        Self {
            id,
            data: FieldData::Integer(value),
        }
    }

    /// Create a new string field
    pub fn string(id: FieldId, value: impl Into<String>) -> Self {
        Self {
            id,
            data: FieldData::String(value.into()),
        }
    }

    /// Create a new binary field
    pub fn binary(id: FieldId, value: impl Into<Vec<u8>>) -> Self {
        Self {
            id,
            data: FieldData::Binary(value.into()),
        }
    }

    /// Get as integer
    pub fn as_integer(&self) -> Option<i32> {
        match &self.data {
            FieldData::Integer(v) => Some(*v),
            _ => None,
        }
    }

    /// Get as string
    pub fn as_string(&self) -> Option<&str> {
        match &self.data {
            FieldData::String(s) => Some(s),
            _ => None,
        }
    }

    /// Get as binary
    pub fn as_binary(&self) -> Option<&[u8]> {
        match &self.data {
            FieldData::Binary(b) => Some(b),
            _ => None,
        }
    }
}

/// Field header (4 bytes: 2 for ID, 2 for size)
#[derive(Debug, Clone, Copy)]
pub struct FieldHeader {
    pub id: u16,
    pub size: u16,
}

impl FieldHeader {
    pub const SIZE: usize = 4;

    /// Parse from bytes
    pub fn from_bytes(mut buf: &[u8]) -> Result<Self, std::io::Error> {
        if buf.len() < Self::SIZE {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "Not enough bytes for field header",
            ));
        }

        Ok(Self {
            id: buf.get_u16(),
            size: buf.get_u16(),
        })
    }

    /// Encode to bytes
    pub fn to_bytes(&self, buf: &mut impl BufMut) {
        buf.put_u16(self.id);
        buf.put_u16(self.size);
    }
}
