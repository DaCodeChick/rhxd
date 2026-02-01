-- Hotline Server Database Schema
-- SQLite 3.x

-- =============================================================================
-- Accounts Table
-- =============================================================================
-- Stores user account information for authentication and access control
CREATE TABLE IF NOT EXISTS accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    login TEXT NOT NULL UNIQUE COLLATE NOCASE,
    password BLOB NOT NULL,              -- Scrambled password (legacy XOR) or future Blake3
    name TEXT NOT NULL,                  -- Display name
    icon_id INTEGER DEFAULT 0,
    access_privileges INTEGER NOT NULL,  -- Access privileges bitfield (i64)
    created_at INTEGER NOT NULL,         -- Unix timestamp
    modified_at INTEGER NOT NULL,        -- Unix timestamp
    
    CHECK(length(login) <= 31),
    CHECK(length(name) <= 31)
);

CREATE INDEX IF NOT EXISTS idx_accounts_login ON accounts(login);

-- =============================================================================
-- Sessions Table
-- =============================================================================
-- Tracks active user sessions (connections)
CREATE TABLE IF NOT EXISTS sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,            -- Session ID used in protocol (1-65535)
    account_id INTEGER,                  -- NULL for guests, FK to accounts
    nickname TEXT NOT NULL,              -- Display name shown to other users
    icon_id INTEGER NOT NULL DEFAULT 0,  -- User icon (0-999)
    flags INTEGER NOT NULL DEFAULT 0,    -- User status flags
    ip_address TEXT NOT NULL,            -- Client IP address
    connected_at INTEGER NOT NULL,       -- Unix timestamp
    last_activity INTEGER NOT NULL,      -- Unix timestamp for keepalive
    
    FOREIGN KEY(account_id) REFERENCES accounts(id) ON DELETE CASCADE,
    CHECK(user_id > 0 AND user_id <= 65535),
    CHECK(length(nickname) <= 31)
);

CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_account_id ON sessions(account_id);
CREATE INDEX IF NOT EXISTS idx_sessions_active ON sessions(last_activity);

-- =============================================================================
-- Files Table
-- =============================================================================
-- File metadata cache for quick listing (optional, can also read filesystem directly)
CREATE TABLE IF NOT EXISTS files (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    path TEXT NOT NULL UNIQUE,        -- Virtual path in Hotline filesystem
    name TEXT NOT NULL,                -- File/folder name
    is_folder INTEGER NOT NULL,        -- 0 = file, 1 = folder
    size INTEGER NOT NULL DEFAULT 0,   -- File size in bytes (0 for folders)
    type_code TEXT,                    -- MacOS type code (4 chars, e.g., 'TEXT')
    creator_code TEXT,                 -- MacOS creator code (4 chars)
    comment TEXT,                      -- File comment
    created_at INTEGER NOT NULL,       -- Unix timestamp
    modified_at INTEGER NOT NULL,      -- Unix timestamp
    physical_path TEXT NOT NULL,       -- Actual filesystem path
    
    CHECK(is_folder IN (0, 1)),
    CHECK(length(name) <= 255),
    CHECK(type_code IS NULL OR length(type_code) = 4),
    CHECK(creator_code IS NULL OR length(creator_code) = 4)
);

CREATE INDEX IF NOT EXISTS idx_files_path ON files(path);
CREATE INDEX IF NOT EXISTS idx_files_parent ON files(
    substr(path, 1, length(path) - length(name) - 1)
);
CREATE INDEX IF NOT EXISTS idx_files_name ON files(name);

-- =============================================================================
-- Messages Table
-- =============================================================================
-- Stores server messages (similar to email inbox)
CREATE TABLE IF NOT EXISTS messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    recipient_account_id INTEGER NOT NULL,  -- Who receives the message
    sender_name TEXT NOT NULL,              -- Who sent it (name, not account)
    subject TEXT,
    message TEXT NOT NULL,
    sent_at INTEGER NOT NULL,               -- Unix timestamp
    read_at INTEGER,                        -- NULL if unread, timestamp if read
    
    FOREIGN KEY(recipient_account_id) REFERENCES accounts(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_messages_recipient ON messages(recipient_account_id);
CREATE INDEX IF NOT EXISTS idx_messages_unread ON messages(recipient_account_id, read_at);

-- =============================================================================
-- Ban List Table
-- =============================================================================
-- Stores banned IP addresses or account logins
CREATE TABLE IF NOT EXISTS bans (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ban_type TEXT NOT NULL,         -- 'ip' or 'account'
    ban_value TEXT NOT NULL UNIQUE, -- IP address or account login
    reason TEXT,
    banned_by TEXT NOT NULL,        -- Admin who created the ban
    banned_at INTEGER NOT NULL,     -- Unix timestamp
    expires_at INTEGER,             -- NULL = permanent, timestamp = temporary
    
    CHECK(ban_type IN ('ip', 'account'))
);

CREATE INDEX IF NOT EXISTS idx_bans_type_value ON bans(ban_type, ban_value);
CREATE INDEX IF NOT EXISTS idx_bans_expires ON bans(expires_at);

-- =============================================================================
-- News Categories Table
-- =============================================================================
-- Hierarchical news categories (for future news system)
CREATE TABLE IF NOT EXISTS news_categories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    parent_id INTEGER,                      -- NULL for top-level categories
    name TEXT NOT NULL,
    guid TEXT NOT NULL UNIQUE,              -- Unique identifier
    created_at INTEGER NOT NULL,
    
    FOREIGN KEY(parent_id) REFERENCES news_categories(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_news_categories_parent ON news_categories(parent_id);
CREATE INDEX IF NOT EXISTS idx_news_categories_guid ON news_categories(guid);

-- =============================================================================
-- News Articles Table
-- =============================================================================
-- News articles within categories (for future news system)
CREATE TABLE IF NOT EXISTS news_articles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    category_id INTEGER NOT NULL,
    parent_article_id INTEGER,              -- For threaded discussions
    title TEXT NOT NULL,
    poster TEXT NOT NULL,                   -- Author name
    data_flavor TEXT NOT NULL DEFAULT 'text/plain',
    data BLOB NOT NULL,                     -- Article content
    flags INTEGER NOT NULL DEFAULT 0,
    posted_at INTEGER NOT NULL,
    
    FOREIGN KEY(category_id) REFERENCES news_categories(id) ON DELETE CASCADE,
    FOREIGN KEY(parent_article_id) REFERENCES news_articles(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_news_articles_category ON news_articles(category_id);
CREATE INDEX IF NOT EXISTS idx_news_articles_parent ON news_articles(parent_article_id);
CREATE INDEX IF NOT EXISTS idx_news_articles_posted ON news_articles(posted_at);

-- =============================================================================
-- Server Metadata Table
-- =============================================================================
-- Stores server configuration and state
CREATE TABLE IF NOT EXISTS server_metadata (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- Initialize with schema version
INSERT OR IGNORE INTO server_metadata (key, value) VALUES ('schema_version', '1');
INSERT OR IGNORE INTO server_metadata (key, value) VALUES ('created_at', strftime('%s', 'now'));
