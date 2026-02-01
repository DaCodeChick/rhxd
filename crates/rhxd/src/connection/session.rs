//! Session management

use rhxcore::types::UserOptions;
use std::net::SocketAddr;
use std::time::SystemTime;

/// Authentication state for a session
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthState {
    /// Connection established, waiting for handshake
    Handshake,
    /// Handshake completed, waiting for login
    LoginPending,
    /// Authenticated (either logged in or guest)
    Authenticated,
}

/// Represents a connected client session
#[derive(Debug, Clone)]
pub struct Session {
    /// Protocol user ID (1-65535, unique during connection)
    pub user_id: u16,

    /// Database account ID (None for guests)
    pub account_id: Option<i64>,

    /// Display nickname
    pub nickname: String,

    /// User icon ID
    pub icon_id: u16,

    /// User flags (idle, admin, etc.)
    pub flags: u16,

    /// User options (refuse private messages, refuse private chat, automatic response)
    pub options: UserOptions,

    /// Client IP address
    pub address: SocketAddr,

    /// Connection timestamp
    pub connected_at: SystemTime,

    /// Last activity timestamp
    pub last_activity: SystemTime,

    /// Authentication state
    pub auth_state: AuthState,
}

impl Session {
    /// Create a new session in handshake state
    pub fn new(user_id: u16, address: SocketAddr) -> Self {
        let now = SystemTime::now();
        Self {
            user_id,
            account_id: None,
            nickname: format!("Guest {}", user_id),
            icon_id: 0,
            flags: 0,
            options: UserOptions::default(),
            address,
            connected_at: now,
            last_activity: now,
            auth_state: AuthState::Handshake,
        }
    }

    /// Authenticate as a guest
    pub fn authenticate_guest(&mut self, nickname: String, icon_id: u16) {
        self.nickname = nickname;
        self.icon_id = icon_id;
        self.auth_state = AuthState::Authenticated;
    }

    /// Authenticate as a logged-in user
    pub fn authenticate_user(&mut self, account_id: i64, nickname: String, icon_id: u16) {
        self.account_id = Some(account_id);
        self.nickname = nickname;
        self.icon_id = icon_id;
        self.auth_state = AuthState::Authenticated;
    }

    /// Mark handshake as complete
    pub fn complete_handshake(&mut self) {
        self.auth_state = AuthState::LoginPending;
    }

    /// Update last activity timestamp
    pub fn touch(&mut self) {
        self.last_activity = SystemTime::now();
    }

    /// Check if the session is authenticated
    pub fn is_authenticated(&self) -> bool {
        self.auth_state == AuthState::Authenticated
    }

    /// Check if the session is a guest
    pub fn is_guest(&self) -> bool {
        self.account_id.is_none()
    }
}
