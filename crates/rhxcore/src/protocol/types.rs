//! Transaction and field type enumerations

/// Transaction types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum TransactionType {
    // Error
    Error = 100,

    // Messages
    GetMessages = 101,
    NewMessage = 102,
    OldPostNews = 103,
    ServerMessage = 104,

    // Chat
    SendChat = 105,
    ChatMessage = 106,

    // Login and session
    Login = 107,
    SendInstantMsg = 108,
    ShowAgreement = 109,
    DisconnectUser = 110,
    DisconnectMsg = 111,

    // Private chat
    InviteNewChat = 112,
    InviteToChat = 113,
    RejectChatInvite = 114,
    JoinChat = 115,
    LeaveChat = 116,
    NotifyChatChangeUser = 117,
    NotifyChatDeleteUser = 118,
    NotifyChatSubject = 119,
    SetChatSubject = 120,

    // Agreement
    Agreed = 121,
    ServerBanner = 122,

    // Files
    GetFileNameList = 200,
    DownloadFile = 202,
    UploadFile = 203,
    DeleteFile = 204,
    NewFolder = 205,
    GetFileInfo = 206,
    SetFileInfo = 207,
    MoveFile = 208,
    MakeFileAlias = 209,
    DownloadFolder = 210,
    DownloadInfo = 211,
    DownloadBanner = 212,
    UploadFolder = 213,

    // Users
    GetUserNameList = 300,
    NotifyChangeUser = 301,
    NotifyDeleteUser = 302,
    GetClientInfoText = 303,
    SetClientUserInfo = 304,

    // User management
    NewUser = 350,
    DeleteUser = 351,
    GetUser = 352,
    SetUser = 353,
    UserAccess = 354,
    UserBroadcast = 355,

    // News
    GetNewsCategoryNameList = 370,
    GetNewsArticleNameList = 371,
    DeleteNewsItem = 380,
    NewNewsFolder = 381,
    NewNewsCategory = 382,
    GetNewsArticleData = 400,
    PostNewsArticle = 410,
    DeleteNewsArticle = 411,

    // Keep alive
    KeepConnectionAlive = 500,
}

impl TransactionType {
    /// Convert from u16
    pub const fn from_u16(value: u16) -> Option<Self> {
        match value {
            100 => Some(Self::Error),
            101 => Some(Self::GetMessages),
            102 => Some(Self::NewMessage),
            103 => Some(Self::OldPostNews),
            104 => Some(Self::ServerMessage),
            105 => Some(Self::SendChat),
            106 => Some(Self::ChatMessage),
            107 => Some(Self::Login),
            108 => Some(Self::SendInstantMsg),
            109 => Some(Self::ShowAgreement),
            110 => Some(Self::DisconnectUser),
            111 => Some(Self::DisconnectMsg),
            112 => Some(Self::InviteNewChat),
            113 => Some(Self::InviteToChat),
            114 => Some(Self::RejectChatInvite),
            115 => Some(Self::JoinChat),
            116 => Some(Self::LeaveChat),
            117 => Some(Self::NotifyChatChangeUser),
            118 => Some(Self::NotifyChatDeleteUser),
            119 => Some(Self::NotifyChatSubject),
            120 => Some(Self::SetChatSubject),
            121 => Some(Self::Agreed),
            122 => Some(Self::ServerBanner),
            200 => Some(Self::GetFileNameList),
            202 => Some(Self::DownloadFile),
            203 => Some(Self::UploadFile),
            204 => Some(Self::DeleteFile),
            205 => Some(Self::NewFolder),
            206 => Some(Self::GetFileInfo),
            207 => Some(Self::SetFileInfo),
            208 => Some(Self::MoveFile),
            209 => Some(Self::MakeFileAlias),
            210 => Some(Self::DownloadFolder),
            211 => Some(Self::DownloadInfo),
            212 => Some(Self::DownloadBanner),
            213 => Some(Self::UploadFolder),
            300 => Some(Self::GetUserNameList),
            301 => Some(Self::NotifyChangeUser),
            302 => Some(Self::NotifyDeleteUser),
            303 => Some(Self::GetClientInfoText),
            304 => Some(Self::SetClientUserInfo),
            350 => Some(Self::NewUser),
            351 => Some(Self::DeleteUser),
            352 => Some(Self::GetUser),
            353 => Some(Self::SetUser),
            354 => Some(Self::UserAccess),
            355 => Some(Self::UserBroadcast),
            370 => Some(Self::GetNewsCategoryNameList),
            371 => Some(Self::GetNewsArticleNameList),
            380 => Some(Self::DeleteNewsItem),
            381 => Some(Self::NewNewsFolder),
            382 => Some(Self::NewNewsCategory),
            400 => Some(Self::GetNewsArticleData),
            410 => Some(Self::PostNewsArticle),
            411 => Some(Self::DeleteNewsArticle),
            500 => Some(Self::KeepConnectionAlive),
            _ => None,
        }
    }

    /// Convert to u16
    #[inline]
    pub const fn to_u16(self) -> u16 {
        self as u16
    }
}

impl From<TransactionType> for u16 {
    #[inline]
    fn from(value: TransactionType) -> Self {
        value.to_u16()
    }
}

/// Error codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ErrorCode {
    NoError = 0,
    UnknownError = 1,
    PermissionDenied = 2,
    NotFound = 3,
    AlreadyExists = 4,
    InvalidParameter = 5,
}

impl ErrorCode {
    /// Convert from u32
    pub const fn from_u32(value: u32) -> Self {
        match value {
            0 => Self::NoError,
            2 => Self::PermissionDenied,
            3 => Self::NotFound,
            4 => Self::AlreadyExists,
            5 => Self::InvalidParameter,
            _ => Self::UnknownError,
        }
    }

    /// Convert to u32
    #[inline]
    pub const fn to_u32(self) -> u32 {
        self as u32
    }
}

impl From<ErrorCode> for u32 {
    #[inline]
    fn from(value: ErrorCode) -> Self {
        value.to_u32()
    }
}
