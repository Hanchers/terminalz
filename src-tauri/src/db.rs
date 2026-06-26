use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::Path;
use std::sync::Mutex;

use crate::models::{ConnectionConfig, HostGroup, Tag, SshKey, PortForward, Snippet};

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

    /// List all connections.  Passwords are **masked** — frontend never sees
    /// the raw password or vault reference.
    pub fn list_all(&self) -> Result<Vec<ConnectionConfig>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, name, host, port, username, password, group_id, remark, auto_snippet_id FROM connections ORDER BY updated_at DESC")?;
        let rows = stmt.query_map([], |row| {
            let raw: String = row.get(5)?;
            Ok(ConnectionConfig {
                id: row.get(0)?,
                name: row.get(1)?,
                host: row.get(2)?,
                port: row.get::<_, i64>(3)? as u16,
                username: row.get(4)?,
                password: if raw.is_empty() {
                    String::new()
                } else {
                    "••••••••".to_string()
                },
                group_id: row.get(6)?,
                remark: row.get(7)?,
                auto_snippet_id: row.get(8)?,
            })
        })?;
        let mut list = Vec::new();
        for row in rows { list.push(row?); }
        Ok(list)
    }

    /// Get the raw `password` column value (vault reference or legacy plaintext).
    /// Used internally by vault operations.
    pub fn get_raw_password(&self, id: i64) -> Result<String> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT password FROM connections WHERE id = ?",
            [id],
            |row| row.get(0),
        )
        .map_err(|e| e.into())
    }

    /// Load a single connection with **real** password column value (for internal
    /// use only — never exposed to frontend).
    pub fn get_connection_internal(&self, id: i64) -> Result<ConnectionConfig> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT id, name, host, port, username, password, group_id, remark, auto_snippet_id FROM connections WHERE id = ?",
            [id],
            |row| {
                Ok(ConnectionConfig {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    host: row.get(2)?,
                    port: row.get::<_, i64>(3)? as u16,
                    username: row.get(4)?,
                    password: row.get(5)?,
                    group_id: row.get(6)?,
                    remark: row.get(7)?,
                    auto_snippet_id: row.get(8)?,
                })
            },
        ).map_err(|e| e.into())
    }

    /// Update only the password column — used during vault migration.
    pub fn update_password(&self, id: i64, new_value: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE connections SET password = ?, updated_at = datetime('now') WHERE id = ?",
            rusqlite::params![new_value, id],
        )?;
        Ok(())
    }

    pub fn save(&self, config: &ConnectionConfig) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        if config.id > 0 {
            conn.execute(
                "UPDATE connections SET name=?, host=?, port=?, username=?, password=?, group_id=?, remark=?, auto_snippet_id=?, updated_at=datetime('now') WHERE id=?",
                rusqlite::params![config.name, config.host, config.port as i64, config.username, config.password, config.group_id, config.remark, config.auto_snippet_id, config.id],
            )?;
            Ok(config.id)
        } else {
            conn.execute(
                "INSERT INTO connections (name, host, port, username, password, group_id, remark, auto_snippet_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                rusqlite::params![config.name, config.host, config.port as i64, config.username, config.password, config.group_id, config.remark, config.auto_snippet_id],
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

    // ---- Keychain CRUD ----

    pub fn list_ssh_keys(&self) -> Result<Vec<SshKey>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, key_type, username, password, private_key, host, remark FROM ssh_keys ORDER BY updated_at DESC"
        )?;
        let rows = stmt.query_map([], |row| {
            let pw: String = row.get(4)?;
            let pk: String = row.get(5)?;
            Ok(SshKey {
                id: row.get(0)?,
                name: row.get(1)?,
                key_type: row.get(2)?,
                username: row.get(3)?,
                password: if pw.is_empty() { String::new() } else { "••••••••".to_string() },
                private_key: if pk.is_empty() { String::new() } else { "••••••••".to_string() },
                host: row.get(6)?,
                remark: row.get(7)?,
            })
        })?;
        let mut list = Vec::new();
        for row in rows { list.push(row?); }
        Ok(list)
    }

    pub fn save_ssh_key(&self, key: &SshKey) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        if key.id > 0 {
            conn.execute(
                "UPDATE ssh_keys SET name=?, key_type=?, username=?, password=?, private_key=?, host=?, remark=?, updated_at=datetime('now') WHERE id=?",
                rusqlite::params![key.name, key.key_type, key.username, key.password, key.private_key, key.host, key.remark, key.id],
            )?;
            Ok(key.id)
        } else {
            conn.execute(
                "INSERT INTO ssh_keys (name, key_type, username, password, private_key, host, remark) VALUES (?, ?, ?, ?, ?, ?, ?)",
                rusqlite::params![key.name, key.key_type, key.username, key.password, key.private_key, key.host, key.remark],
            )?;
            Ok(conn.last_insert_rowid())
        }
    }

    /// Get raw ssh_key password/private_key values for internal use.
    pub fn get_ssh_key_internal(&self, id: i64) -> Result<SshKey> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT id, name, key_type, username, password, private_key, host, remark FROM ssh_keys WHERE id = ?",
            [id],
            |row| Ok(SshKey {
                id: row.get(0)?,
                name: row.get(1)?,
                key_type: row.get(2)?,
                username: row.get(3)?,
                password: row.get(4)?,
                private_key: row.get(5)?,
                host: row.get(6)?,
                remark: row.get(7)?,
            }),
        ).map_err(|e| e.into())
    }

    pub fn delete_ssh_key(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM ssh_keys WHERE id=?", [id])?;
        Ok(())
    }

    // ---- Port Forward CRUD ----

    pub fn list_port_forwards(&self) -> Result<Vec<PortForward>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, connection_id, local_port, remote_host, remote_port, direction, enabled, remark FROM port_forwards ORDER BY updated_at DESC"
        )?;
        let rows = stmt.query_map([], |row| {
            let enabled_int: i64 = row.get(7)?;
            Ok(PortForward {
                id: row.get(0)?,
                name: row.get(1)?,
                connection_id: row.get(2)?,
                local_port: row.get::<_, i64>(3)? as u16,
                remote_host: row.get(4)?,
                remote_port: row.get::<_, i64>(5)? as u16,
                direction: row.get(6)?,
                enabled: enabled_int != 0,
                remark: row.get(8)?,
            })
        })?;
        let mut list = Vec::new();
        for row in rows { list.push(row?); }
        Ok(list)
    }

    pub fn save_port_forward(&self, pf: &PortForward) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        if pf.id > 0 {
            conn.execute(
                "UPDATE port_forwards SET name=?, connection_id=?, local_port=?, remote_host=?, remote_port=?, direction=?, enabled=?, remark=?, updated_at=datetime('now') WHERE id=?",
                rusqlite::params![pf.name, pf.connection_id, pf.local_port as i64, pf.remote_host, pf.remote_port as i64, pf.direction, pf.enabled as i64, pf.remark, pf.id],
            )?;
            Ok(pf.id)
        } else {
            conn.execute(
                "INSERT INTO port_forwards (name, connection_id, local_port, remote_host, remote_port, direction, enabled, remark) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                rusqlite::params![pf.name, pf.connection_id, pf.local_port as i64, pf.remote_host, pf.remote_port as i64, pf.direction, pf.enabled as i64, pf.remark],
            )?;
            Ok(conn.last_insert_rowid())
        }
    }

    pub fn delete_port_forward(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM port_forwards WHERE id=?", [id])?;
        Ok(())
    }

    // ---- Snippet CRUD ----

    pub fn list_snippets(&self) -> Result<Vec<Snippet>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, content, language, is_favorite, remark FROM snippets ORDER BY is_favorite DESC, updated_at DESC"
        )?;
        let rows = stmt.query_map([], |row| {
            let fav_int: i64 = row.get(4)?;
            Ok(Snippet {
                id: row.get(0)?,
                name: row.get(1)?,
                content: row.get(2)?,
                language: row.get(3)?,
                is_favorite: fav_int != 0,
                remark: row.get(5)?,
            })
        })?;
        let mut list = Vec::new();
        for row in rows { list.push(row?); }
        Ok(list)
    }

    pub fn save_snippet(&self, snippet: &Snippet) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        if snippet.id > 0 {
            conn.execute(
                "UPDATE snippets SET name=?, content=?, language=?, is_favorite=?, remark=?, updated_at=datetime('now') WHERE id=?",
                rusqlite::params![snippet.name, snippet.content, snippet.language, snippet.is_favorite as i64, snippet.remark, snippet.id],
            )?;
            Ok(snippet.id)
        } else {
            conn.execute(
                "INSERT INTO snippets (name, content, language, is_favorite, remark) VALUES (?, ?, ?, ?, ?)",
                rusqlite::params![snippet.name, snippet.content, snippet.language, snippet.is_favorite as i64, snippet.remark],
            )?;
            Ok(conn.last_insert_rowid())
        }
    }

    pub fn delete_snippet(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM snippets WHERE id=?", [id])?;
        Ok(())
    }

    /// Load snippet content by id (for auto-execute on connect).
    pub fn get_snippet_content(&self, id: i64) -> Result<String> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT content FROM snippets WHERE id = ?",
            [id],
            |row| row.get(0),
        ).map_err(|e| e.into())
    }

    /// List all connections as (id, name) pairs for use in port-forward selection.
    pub fn list_connections_compact(&self) -> Result<Vec<(i64, String)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name FROM connections ORDER BY name")?;
        let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
        let mut list = Vec::new();
        for row in rows { list.push(row?); }
        Ok(list)
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
    vault: tauri::State<'_, crate::vault::Vault>,
    config: ConnectionConfig,
) -> Result<ConnectionConfig, String> {
    let password_value = if config.password.is_empty() && config.id > 0 {
        // Editing without changing password — keep existing vault reference.
        let old = db.get_raw_password(config.id).map_err(|e| e.to_string())?;
        // If the old password is also empty (e.g. keychain entry was lost and the
        // stale reference was cleared), reject the save so the user must enter one.
        if old.is_empty() {
            return Err(
                "Password is required.  The previous credential was lost from the OS keychain.\n\
                 Please enter a new password for this host before saving."
                    .to_string(),
            );
        }
        old
    } else {
        // New or changed password — encrypt and store.
        if config.id > 0 {
            vault
                .store(config.id, &config.password)
                .map_err(|e| format!("vault store: {}", e))?
        } else {
            let encrypted = vault
                .encrypt_aes(&config.password)
                .map_err(|e| format!("encrypt: {}", e))?;
            format!("__AES__:{}", encrypted)
        }
    };

    let mut config_with_pw = config.clone();
    config_with_pw.password = password_value;
    let new_id = db.save(&config_with_pw).map_err(|e| e.to_string())?;

    Ok(ConnectionConfig {
        id: new_id,
        password: if config_with_pw.password.is_empty() {
            String::new()
        } else {
            "••••••••".to_string()
        },
        ..config
    })
}

#[tauri::command]
pub(crate) fn delete_connection(
    db: tauri::State<'_, DbState>,
    vault: tauri::State<'_, crate::vault::Vault>,
    id: i64,
) -> Result<(), String> {
    // Clean up keychain entry before deleting the row.
    if let Ok(raw) = db.get_raw_password(id) {
        vault.delete(id, &raw);
    }
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
        return Err("Cannot delete group: it still contains subgroups".to_string());
    }
    let count = db.count_hosts_in_group(id).map_err(|e| e.to_string())?;
    if count > 0 {
        return Err(format!(
            "Cannot delete group: {} host(s) still exist in this group and its subgroups",
            count
        ));
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

// ---- Keychain commands ----

#[tauri::command]
pub(crate) fn list_ssh_keys(db: tauri::State<'_, DbState>) -> Result<Vec<SshKey>, String> {
    db.list_ssh_keys().map_err(|e| e.to_string())
}

#[tauri::command]
pub(crate) fn save_ssh_key(
    db: tauri::State<'_, DbState>,
    vault: tauri::State<'_, crate::vault::Vault>,
    key: SshKey,
) -> Result<SshKey, String> {
    let mut k = key.clone();
    // If editing and password/private_key is empty, keep existing values
    if k.id > 0 {
        let old = db.get_ssh_key_internal(k.id).map_err(|e| e.to_string())?;
        if k.password.is_empty() { k.password = old.password; }
        if k.private_key.is_empty() { k.private_key = old.private_key; }
    }
    // Encrypt password / private_key if newly provided
    if !k.password.is_empty() && !k.password.starts_with("__AES__:") {
        let encrypted = vault.encrypt_aes(&k.password).map_err(|e| format!("encrypt: {}", e))?;
        k.password = format!("__AES__:{}", encrypted);
    }
    if !k.private_key.is_empty() && !k.private_key.starts_with("__AES__:") {
        let encrypted = vault.encrypt_aes(&k.private_key).map_err(|e| format!("encrypt: {}", e))?;
        k.private_key = format!("__AES__:{}", encrypted);
    }
    let new_id = db.save_ssh_key(&k).map_err(|e| e.to_string())?;
    // Return masked version
    Ok(SshKey {
        id: new_id,
        password: if key.password.is_empty() { String::new() } else { "••••••••".to_string() },
        private_key: if key.private_key.is_empty() { String::new() } else { "••••••••".to_string() },
        ..key
    })
}

#[tauri::command]
pub(crate) fn delete_ssh_key(db: tauri::State<'_, DbState>, id: i64) -> Result<(), String> {
    db.delete_ssh_key(id).map_err(|e| e.to_string())
}

// ---- Port Forward commands ----

#[tauri::command]
pub(crate) fn list_port_forwards(db: tauri::State<'_, DbState>) -> Result<Vec<PortForward>, String> {
    db.list_port_forwards().map_err(|e| e.to_string())
}

#[tauri::command]
pub(crate) fn save_port_forward(
    db: tauri::State<'_, DbState>,
    config: PortForward,
) -> Result<PortForward, String> {
    let new_id = db.save_port_forward(&config).map_err(|e| e.to_string())?;
    Ok(PortForward { id: new_id, ..config })
}

#[tauri::command]
pub(crate) fn delete_port_forward(db: tauri::State<'_, DbState>, id: i64) -> Result<(), String> {
    db.delete_port_forward(id).map_err(|e| e.to_string())
}

// ---- Snippet commands ----

#[tauri::command]
pub(crate) fn list_snippets(db: tauri::State<'_, DbState>) -> Result<Vec<Snippet>, String> {
    db.list_snippets().map_err(|e| e.to_string())
}

#[tauri::command]
pub(crate) fn save_snippet(
    db: tauri::State<'_, DbState>,
    snippet: Snippet,
) -> Result<Snippet, String> {
    let new_id = db.save_snippet(&snippet).map_err(|e| e.to_string())?;
    Ok(Snippet { id: new_id, ..snippet })
}

#[tauri::command]
pub(crate) fn delete_snippet(db: tauri::State<'_, DbState>, id: i64) -> Result<(), String> {
    db.delete_snippet(id).map_err(|e| e.to_string())
}

// ---- Connection list helper (compact) ----

#[tauri::command]
pub(crate) fn list_connections_compact(
    db: tauri::State<'_, DbState>,
) -> Result<Vec<(i64, String)>, String> {
    db.list_connections_compact().map_err(|e| e.to_string())
}
