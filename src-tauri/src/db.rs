use anyhow::{Context, Result};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Mutex;

// ---- 连接配置结构体 ----

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub id: i64,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub group_id: i64,
}

// ---- 分组结构体 ----

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostGroup {
    pub id: i64,
    pub parent_id: i64,
    pub name: String,
    pub remark: String,
}

// ---- 数据库状态 ----

pub struct DbState {
    conn: Mutex<Connection>,
}

impl DbState {
    pub fn new(db_path: &Path) -> Result<Self> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        let conn = Connection::open(db_path).context("无法打开 SQLite 数据库")?;

        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
            .ok();

        // --- connections 表 ---
        conn.execute(
            "CREATE TABLE IF NOT EXISTS connections (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                name        TEXT NOT NULL,
                host        TEXT NOT NULL,
                port        INTEGER NOT NULL DEFAULT 22,
                username    TEXT NOT NULL,
                password    TEXT NOT NULL DEFAULT '',
                group_id    INTEGER NOT NULL DEFAULT 0,
                created_at  TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
            )",
            [],
        ).context("建 connections 表失败")?;

        // 兼容旧库：没有 group_id 列时自动添加
        let has_group_id: bool = conn
            .prepare("SELECT group_id FROM connections LIMIT 0")
            .is_ok();
        if !has_group_id {
            conn.execute("ALTER TABLE connections ADD COLUMN group_id INTEGER NOT NULL DEFAULT 0", [])
                .ok();
        }

        // --- groups 表 ---
        conn.execute(
            "CREATE TABLE IF NOT EXISTS host_groups (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                parent_id   INTEGER NOT NULL DEFAULT 0,
                name        TEXT NOT NULL,
                remark      TEXT NOT NULL DEFAULT '',
                created_at  TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
            )",
            [],
        ).context("建 host_groups 表失败")?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    // ---- Connection CRUD ----

    pub fn list_all(&self) -> Result<Vec<ConnectionConfig>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, name, host, port, username, password, group_id FROM connections ORDER BY updated_at DESC")?;
        let rows = stmt.query_map([], |row| {
            Ok(ConnectionConfig {
                id: row.get(0)?,
                name: row.get(1)?,
                host: row.get(2)?,
                port: row.get::<_, i64>(3)? as u16,
                username: row.get(4)?,
                password: row.get(5)?,
                group_id: row.get(6)?,
            })
        })?;
        let mut list = Vec::new();
        for row in rows { list.push(row?); }
        Ok(list)
    }

    pub fn save(&self, config: &ConnectionConfig) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        if config.id > 0 {
            conn.execute(
                "UPDATE connections SET name=?, host=?, port=?, username=?, password=?, group_id=?, updated_at=datetime('now') WHERE id=?",
                rusqlite::params![config.name, config.host, config.port as i64, config.username, config.password, config.group_id, config.id],
            )?;
            Ok(config.id)
        } else {
            conn.execute(
                "INSERT INTO connections (name, host, port, username, password, group_id) VALUES (?, ?, ?, ?, ?, ?)",
                rusqlite::params![config.name, config.host, config.port as i64, config.username, config.password, config.group_id],
            )?;
            Ok(conn.last_insert_rowid())
        }
    }

    pub fn delete(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM connections WHERE id=?", [id])?;
        Ok(())
    }

    /// 统计某个分组及其子分组下的 host 数量
    pub fn count_hosts_in_group(&self, group_id: i64) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let descendant_ids = self.collect_descendant_ids(&conn, group_id);
        let mut ids = descendant_ids;
        ids.push(group_id);

        let placeholders: Vec<String> = ids.iter().map(|_| "?".to_string()).collect();
        let sql = format!(
            "SELECT COUNT(*) FROM connections WHERE group_id IN ({})",
            placeholders.join(",")
        );
        let params: Vec<rusqlite::types::Value> = ids.iter().map(|&i| rusqlite::types::Value::from(i)).collect();
        let params_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p as &dyn rusqlite::types::ToSql).collect();
        let count: i64 = conn.query_row(&sql, params_refs.as_slice(), |row| row.get(0))?;
        Ok(count)
    }

    // ---- Group CRUD ----

    pub fn list_groups(&self) -> Result<Vec<HostGroup>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, parent_id, name, remark FROM host_groups ORDER BY parent_id, name"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(HostGroup {
                id: row.get(0)?,
                parent_id: row.get(1)?,
                name: row.get(2)?,
                remark: row.get(3)?,
            })
        })?;
        let mut list = Vec::new();
        for row in rows { list.push(row?); }
        Ok(list)
    }

    pub fn save_group(&self, group: &HostGroup) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        if group.id > 0 {
            conn.execute(
                "UPDATE host_groups SET parent_id=?, name=?, remark=?, updated_at=datetime('now') WHERE id=?",
                rusqlite::params![group.parent_id, group.name, group.remark, group.id],
            )?;
            Ok(group.id)
        } else {
            conn.execute(
                "INSERT INTO host_groups (parent_id, name, remark) VALUES (?, ?, ?)",
                rusqlite::params![group.parent_id, group.name, group.remark],
            )?;
            Ok(conn.last_insert_rowid())
        }
    }

    pub fn delete_group(&self, group_id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM host_groups WHERE id=?", [group_id])?;
        Ok(())
    }

    /// 检查分组是否有子分组
    pub fn has_child_groups(&self, group_id: i64) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM host_groups WHERE parent_id=?",
            [group_id],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    /// 递归收集所有后代分组的 id
    fn collect_descendant_ids(&self, conn: &Connection, parent_id: i64) -> Vec<i64> {
        let mut result = Vec::new();
        let mut stmt = match conn.prepare("SELECT id FROM host_groups WHERE parent_id=?") {
            Ok(s) => s,
            Err(_) => return result,
        };
        let children: Vec<i64> = stmt
            .query_map([parent_id], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        for child_id in &children {
            result.push(*child_id);
            result.extend(self.collect_descendant_ids(conn, *child_id));
        }
        result
    }
}
