-- ============================================================
-- Terminalz Database Schema
-- All CREATE TABLE statements in one place.
-- Loaded at startup via include_str!() in db.rs.
-- ============================================================

-- Host connection configs
CREATE TABLE IF NOT EXISTS connections (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,  -- unique ID
    name        TEXT NOT NULL,                       -- display name
    host        TEXT NOT NULL,                       -- IP / hostname
    port        INTEGER NOT NULL DEFAULT 22,         -- SSH port
    username    TEXT NOT NULL,                       -- login user
    password    TEXT NOT NULL DEFAULT '',            -- login password
    group_id    INTEGER NOT NULL DEFAULT 0,          -- FK -> host_groups.id
    remark      TEXT NOT NULL DEFAULT '',            -- user notes
    auto_snippet_id INTEGER NOT NULL DEFAULT 0,      -- FK -> snippets.id (0=off)
    keychain_id  INTEGER NOT NULL DEFAULT 0,          -- FK -> ssh_keys.id (0=none)
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Tree-style host groups (supports nesting via parent_id)
CREATE TABLE IF NOT EXISTS host_groups (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    parent_id   INTEGER NOT NULL DEFAULT 0,          -- 0 = root level
    name        TEXT NOT NULL,
    remark      TEXT NOT NULL DEFAULT '',
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Tags that can be attached to hosts
CREATE TABLE IF NOT EXISTS tags (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    name       TEXT NOT NULL,
    color      TEXT NOT NULL DEFAULT '#3fb950',       -- hex color for badge
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Many-to-many: host <-> tag
CREATE TABLE IF NOT EXISTS host_tags (
    host_id INTEGER NOT NULL,
    tag_id  INTEGER NOT NULL,
    PRIMARY KEY (host_id, tag_id),
    FOREIGN KEY (host_id) REFERENCES connections(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id)  REFERENCES tags(id)        ON DELETE CASCADE
);

-- SSH keychain entries
CREATE TABLE IF NOT EXISTS ssh_keys (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    name            TEXT NOT NULL,
    key_type        TEXT NOT NULL DEFAULT 'password',   -- 'password' | 'private_key'
    username        TEXT NOT NULL DEFAULT '',
    password        TEXT NOT NULL DEFAULT '',           -- encrypted password or passphrase
    private_key     TEXT NOT NULL DEFAULT '',           -- encrypted private key content
    host            TEXT NOT NULL DEFAULT '',           -- optional target host filter
    remark          TEXT NOT NULL DEFAULT '',
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Port forward rules
CREATE TABLE IF NOT EXISTS port_forwards (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    name            TEXT NOT NULL,
    connection_id   INTEGER NOT NULL DEFAULT 0,        -- FK -> connections.id (0=manual)
    local_port      INTEGER NOT NULL,
    remote_host     TEXT NOT NULL DEFAULT 'localhost',
    remote_port     INTEGER NOT NULL,
    direction       TEXT NOT NULL DEFAULT 'local',      -- 'local' | 'remote'
    enabled         INTEGER NOT NULL DEFAULT 0,         -- 0=stopped, 1=running
    remark          TEXT NOT NULL DEFAULT '',
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Code snippets
CREATE TABLE IF NOT EXISTS snippets (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    name            TEXT NOT NULL,
    content         TEXT NOT NULL,
    language        TEXT NOT NULL DEFAULT 'shell',
    is_favorite     INTEGER NOT NULL DEFAULT 0,
    remark          TEXT NOT NULL DEFAULT '',
    created_at      TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
);
