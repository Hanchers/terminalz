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
