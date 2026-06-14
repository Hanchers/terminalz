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

// ---- 标签结构体 ----

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub color: String,
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

        // --- tags 表 ---
        conn.execute(
            "CREATE TABLE IF NOT EXISTS tags (
                id         INTEGER PRIMARY KEY AUTOINCREMENT,
                name       TEXT NOT NULL,
                color      TEXT NOT NULL DEFAULT '#3fb950',
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            )",
            [],
        ).context("建 tags 表失败")?;

        // --- host_tags 关联表 ---
        conn.execute(
            "CREATE TABLE IF NOT EXISTS host_tags (
                host_id INTEGER NOT NULL,
                tag_id  INTEGER NOT NULL,
                PRIMARY KEY (host_id, tag_id),
                FOREIGN KEY (host_id) REFERENCES connections(id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id)  REFERENCES tags(id)       ON DELETE CASCADE
            )",
            [],
        ).context("建 host_tags 表失败")?;

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

    // ---- Tag CRUD ----

    pub fn list_tags(&self) -> Result<Vec<Tag>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, color FROM tags ORDER BY name"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
            })
        })?;
        let mut list = Vec::new();
        for row in rows { list.push(row?); }
        Ok(list)
    }

    pub fn save_tag(&self, name: &str, color: &str) -> Result<Tag> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO tags (name, color) VALUES (?, ?)",
            rusqlite::params![name, color],
        )?;
        let id = conn.last_insert_rowid();
        Ok(Tag { id, name: name.to_string(), color: color.to_string() })
    }

    pub fn delete_tag(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        // host_tags 由 ON DELETE CASCADE 自动清理
        conn.execute("DELETE FROM tags WHERE id=?", [id])?;
        Ok(())
    }

    pub fn update_tag(&self, id: i64, name: &str, color: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE tags SET name=?, color=? WHERE id=?",
            rusqlite::params![name, color, id],
        )?;
        Ok(())
    }

    // ---- Host-Tag 关联 ----

    /// 获取某个 host 的所有标签
    pub fn get_host_tags(&self, host_id: i64) -> Result<Vec<Tag>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT t.id, t.name, t.color FROM tags t
             INNER JOIN host_tags ht ON t.id = ht.tag_id
             WHERE ht.host_id = ?
             ORDER BY t.name"
        )?;
        let rows = stmt.query_map([host_id], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
                color: row.get(2)?,
            })
        })?;
        let mut list = Vec::new();
        for row in rows { list.push(row?); }
        Ok(list)
    }

    /// 批量获取多个 host 的标签（返回 host_id → Vec<Tag> 映射）
    #[allow(dead_code)]
    pub fn get_hosts_tags(&self, host_ids: &[i64]) -> Result<std::collections::HashMap<i64, Vec<Tag>>> {
        if host_ids.is_empty() {
            return Ok(std::collections::HashMap::new());
        }
        let conn = self.conn.lock().unwrap();
        let placeholders: Vec<String> = host_ids.iter().map(|_| "?".to_string()).collect();
        let sql = format!(
            "SELECT ht.host_id, t.id, t.name, t.color FROM tags t
             INNER JOIN host_tags ht ON t.id = ht.tag_id
             WHERE ht.host_id IN ({})
             ORDER BY t.name",
            placeholders.join(",")
        );
        let params: Vec<rusqlite::types::Value> = host_ids
            .iter()
            .map(|&i| rusqlite::types::Value::from(i))
            .collect();
        let params_refs: Vec<&dyn rusqlite::types::ToSql> = params
            .iter()
            .map(|p| p as &dyn rusqlite::types::ToSql)
            .collect();
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(params_refs.as_slice(), |row| {
            Ok((row.get::<_, i64>(0)?, Tag {
                id: row.get(1)?,
                name: row.get(2)?,
                color: row.get(3)?,
            }))
        })?;
        let mut map: std::collections::HashMap<i64, Vec<Tag>> = std::collections::HashMap::new();
        for row in rows {
            let (host_id, tag) = row?;
            map.entry(host_id).or_default().push(tag);
        }
        Ok(map)
    }

    /// 设置 host 的标签列表（全量替换）
    pub fn set_host_tags(&self, host_id: i64, tag_ids: &[i64]) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM host_tags WHERE host_id=?", [host_id])?;
        for tag_id in tag_ids {
            conn.execute(
                "INSERT OR IGNORE INTO host_tags (host_id, tag_id) VALUES (?, ?)",
                rusqlite::params![host_id, tag_id],
            )?;
        }
        Ok(())
    }
}
