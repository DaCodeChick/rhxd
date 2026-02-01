-- Initial schema for rhxd

-- Accounts table
CREATE TABLE IF NOT EXISTS accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    login TEXT NOT NULL UNIQUE,
    password BLOB NOT NULL,
    name TEXT NOT NULL,
    icon_id INTEGER DEFAULT 0,
    access_privileges INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    modified_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_accounts_login ON accounts(login);

-- Files table (metadata index)
CREATE TABLE IF NOT EXISTS files (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    path TEXT NOT NULL UNIQUE,
    physical_path TEXT NOT NULL,
    name TEXT NOT NULL,
    size INTEGER NOT NULL,
    type_code TEXT,
    creator_code TEXT,
    comment TEXT,
    is_folder INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    modified_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_files_path ON files(path);
