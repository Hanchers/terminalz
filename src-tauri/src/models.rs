use serde::{Deserialize, Serialize};

/// 连接配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub id: i64,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub group_id: i64,
    pub remark: String,
    pub auto_snippet_id: i64,
}

/// 主机分组
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostGroup {
    pub id: i64,
    pub parent_id: i64,
    pub name: String,
    pub remark: String,
}

/// 标签
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub color: String,
}

/// SSH keychain entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshKey {
    pub id: i64,
    pub name: String,
    pub key_type: String,
    pub username: String,
    pub password: String,       // masked when listing
    pub private_key: String,    // masked when listing
    pub host: String,
    pub remark: String,
}

/// Port forward rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortForward {
    pub id: i64,
    pub name: String,
    pub connection_id: i64,
    pub local_port: u16,
    pub remote_host: String,
    pub remote_port: u16,
    pub direction: String,
    pub enabled: bool,
    pub remark: String,
}

/// Code snippet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snippet {
    pub id: i64,
    pub name: String,
    pub content: String,
    pub language: String,
    pub is_favorite: bool,
    pub remark: String,
}
