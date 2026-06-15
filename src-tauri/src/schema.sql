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
