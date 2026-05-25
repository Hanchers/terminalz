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
}

// ---- 数据库状态 ----

pub struct DbState {
    conn: Mutex<Connection>,
}

impl DbState {
    /// 初始化数据库：打开/创建 db 文件，建表
    pub fn new(db_path: &Path) -> Result<Self> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        let conn = Connection::open(db_path).context("无法打开 SQLite 数据库")?;

        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
            .ok();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS connections (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                name        TEXT NOT NULL,
                host        TEXT NOT NULL,
                port        INTEGER NOT NULL DEFAULT 22,
                username    TEXT NOT NULL,
                password    TEXT NOT NULL DEFAULT '',
                created_at  TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
            )",
            [],
        )
        .context("建表失败")?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    // ---- CRUD ----

    /// 查询所有连接配置
    pub fn list_all(&self) -> Result<Vec<ConnectionConfig>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, name, host, port, username, password FROM connections ORDER BY updated_at DESC")?;
        let rows = stmt.query_map([], |row| {
            Ok(ConnectionConfig {
                id: row.get(0)?,
                name: row.get(1)?,
                host: row.get(2)?,
                port: row.get::<_, i64>(3)? as u16,
                username: row.get(4)?,
                password: row.get(5)?,
            })
        })?;

        let mut list = Vec::new();
        for row in rows {
            list.push(row?);
        }
        Ok(list)
    }

    /// 保存（新增或更新）连接配置
    /// 返回值：插入/更新的 id
    pub fn save(&self, config: &ConnectionConfig) -> Result<i64> {
        let conn = self.conn.lock().unwrap();

        if config.id > 0 {
            // 更新已有记录
            conn.execute(
                "UPDATE connections SET name=?, host=?, port=?, username=?, password=?, updated_at=datetime('now') WHERE id=?",
                rusqlite::params![config.name, config.host, config.port as i64, config.username, config.password, config.id],
            )?;
            Ok(config.id)
        } else {
            // 新增
            conn.execute(
                "INSERT INTO connections (name, host, port, username, password) VALUES (?, ?, ?, ?, ?)",
                rusqlite::params![config.name, config.host, config.port as i64, config.username, config.password],
            )?;
            Ok(conn.last_insert_rowid())
        }
    }

    /// 删除指定连接
    pub fn delete(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM connections WHERE id=?", [id])?;
        Ok(())
    }
}
