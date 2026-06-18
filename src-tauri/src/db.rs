use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::Path;
use std::sync::Mutex;

use crate::models::{ConnectionConfig, HostGroup, Tag};

// ---- 数据库状态 ----

pub struct DbState {
    conn: Mutex<Connection>,
}

impl DbState {
    pub fn new(db_path: &Path) -> Result<Self> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        let conn = Connection::open(db_path).context("Failed to open SQLite database")?;

        let schema = include_str!("schema.sql");
        conn.execute_batch(
            &format!("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON; {}", schema),
        )
        .context("Failed to initialize database schema")?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    // ---- Connection CRUD ----

    pub fn list_all(&self) -> Result<Vec<ConnectionConfig>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, name, host, port, username, password, group_id, remark FROM connections ORDER BY updated_at DESC")?;
        let rows = stmt.query_map([], |row| {
            Ok(ConnectionConfig {
                id: row.get(0)?,
                name: row.get(1)?,
                host: row.get(2)?,
                port: row.get::<_, i64>(3)? as u16,
                username: row.get(4)?,
                password: row.get(5)?,
                group_id: row.get(6)?,
                remark: row.get(7)?,
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
                "UPDATE connections SET name=?, host=?, port=?, username=?, password=?, group_id=?, remark=?, updated_at=datetime('now') WHERE id=?",
                rusqlite::params![config.name, config.host, config.port as i64, config.username, config.password, config.group_id, config.remark, config.id],
            )?;
            Ok(config.id)
        } else {
            conn.execute(
                "INSERT INTO connections (name, host, port, username, password, group_id, remark) VALUES (?, ?, ?, ?, ?, ?, ?)",
                rusqlite::params![config.name, config.host, config.port as i64, config.username, config.password, config.group_id, config.remark],
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
        let count: i64 = conn.query_row(
            "WITH RECURSIVE subtree(id) AS (
                SELECT id FROM host_groups WHERE parent_id = ?
                UNION ALL
                SELECT g.id FROM host_groups g JOIN subtree s ON g.parent_id = s.id
            )
            SELECT COUNT(*) FROM connections
            WHERE group_id = ? OR group_id IN (SELECT id FROM subtree)",
            rusqlite::params![group_id, group_id],
            |row| row.get(0),
        )?;
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

    /// 检查分组是否有子分组（单次查询，不需要 CTE）
    pub fn has_child_groups(&self, group_id: i64) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM host_groups WHERE parent_id=?",
            [group_id],
            |row| row.get(0),
        )?;
        Ok(count > 0)
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

// ---- Tauri commands ----

#[tauri::command]
pub(crate) fn list_connections(db: tauri::State<'_, DbState>) -> Result<Vec<ConnectionConfig>, String> {
    db.list_all().map_err(|e| e.to_string())
}

#[tauri::command]
pub(crate) fn save_connection(
    db: tauri::State<'_, DbState>,
    config: ConnectionConfig,
) -> Result<ConnectionConfig, String> {
    let new_id = db.save(&config).map_err(|e| e.to_string())?;
    Ok(ConnectionConfig { id: new_id, ..config })
}

#[tauri::command]
pub(crate) fn delete_connection(db: tauri::State<'_, DbState>, id: i64) -> Result<(), String> {
    db.delete(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub(crate) fn list_groups(db: tauri::State<'_, DbState>) -> Result<Vec<HostGroup>, String> {
    db.list_groups().map_err(|e| e.to_string())
}

#[tauri::command]
pub(crate) fn save_group(db: tauri::State<'_, DbState>, group: HostGroup) -> Result<HostGroup, String> {
    let new_id = db.save_group(&group).map_err(|e| e.to_string())?;
    Ok(HostGroup { id: new_id, ..group })
}

#[tauri::command]
pub(crate) fn delete_group(db: tauri::State<'_, DbState>, id: i64) -> Result<(), String> {
    if db.has_child_groups(id).map_err(|e| e.to_string())? {
        return Err("该分组下存在子分组，请先删除子分组".to_string());
    }
    let count = db.count_hosts_in_group(id).map_err(|e| e.to_string())?;
    if count > 0 {
        return Err(format!("该分组及子分组下存在 {} 个 host，请先移除这些 host", count));
    }
    db.delete_group(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub(crate) fn list_tags(db: tauri::State<'_, DbState>) -> Result<Vec<Tag>, String> {
    db.list_tags().map_err(|e| e.to_string())
}

#[tauri::command]
pub(crate) fn save_tag(
    db: tauri::State<'_, DbState>,
    name: String,
    color: String,
) -> Result<Tag, String> {
    db.save_tag(&name, &color).map_err(|e| e.to_string())
}

#[tauri::command]
pub(crate) fn delete_tag(db: tauri::State<'_, DbState>, id: i64) -> Result<(), String> {
    db.delete_tag(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub(crate) fn update_tag(
    db: tauri::State<'_, DbState>,
    id: i64,
    name: String,
    color: String,
) -> Result<(), String> {
    db.update_tag(id, &name, &color).map_err(|e| e.to_string())
}

#[tauri::command]
pub(crate) fn get_host_tags(db: tauri::State<'_, DbState>, host_id: i64) -> Result<Vec<Tag>, String> {
    db.get_host_tags(host_id).map_err(|e| e.to_string())
}

/// Batch query: returns a map of host_id → tags for all given host IDs.
/// Replaces N individual get_host_tags calls with a single query.
#[tauri::command]
pub(crate) fn list_all_host_tags(
    db: tauri::State<'_, DbState>,
    host_ids: Vec<i64>,
) -> Result<std::collections::HashMap<i64, Vec<Tag>>, String> {
    db.get_hosts_tags(&host_ids).map_err(|e| e.to_string())
}

#[tauri::command]
pub(crate) fn set_host_tags(
    db: tauri::State<'_, DbState>,
    host_id: i64,
    tag_ids: Vec<i64>,
) -> Result<(), String> {
    db.set_host_tags(host_id, &tag_ids).map_err(|e| e.to_string())
}
