-- Initial schema for rhxtrackd

-- Servers table
CREATE TABLE IF NOT EXISTS servers (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    address TEXT NOT NULL,
    port INTEGER NOT NULL,
    user_count INTEGER DEFAULT 0,
    registered_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_update TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_servers_last_update ON servers(last_update);
