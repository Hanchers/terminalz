use anyhow::{Context, Result};
use russh::*;
use russh_sftp::client::SftpSession;
use std::path::Path;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const CHUNK_SIZE: usize = 256 * 1024; // 256KB

// ---- 数据结构 ----

#[derive(serde::Serialize, Clone, Debug)]
pub struct SftpProgress {
    pub file_name: String,
    pub current: u64,
    pub total: u64,
    pub percentage: f64,
    pub status: String, // "uploading" | "completed" | "error"
}

#[derive(serde::Serialize, Clone, Debug)]
pub struct FileEntry {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: String, // human-readable date
}

#[derive(serde::Serialize, Clone)]
pub struct UploadResult {
    pub success: Vec<String>,
    pub failed: Vec<String>,
}

// ---- 连接辅助 ----

async fn connect_sftp(credentials: &(String, u16, String, String)) -> Result<SftpSession> {
    let (host, port, username, password) = credentials;
    let config = Arc::new(client::Config::default());
    let handler = crate::ssh::ClientHandler;

    let mut session = client::connect(config, (host.clone(), *port), handler)
        .await
        .context("SFTP 连接失败")?;

    session
        .authenticate_password(username, password)
        .await
        .context("SFTP 认证失败")?;

    let channel = session
        .channel_open_session()
        .await
        .context("无法打开通道")?;

    channel
        .request_subsystem(true, "sftp")
        .await
        .context("无法请求 SFTP 子系统")?;

    let stream = channel.into_stream();
    SftpSession::new(stream).await.context("无法初始化 SFTP 会话")
}

// ---- 目录列表 ----

pub async fn list_dir(
    credentials: &(String, u16, String, String),
    remote_path: &str,
) -> Result<Vec<FileEntry>> {
    let sftp = connect_sftp(credentials).await?;
    let path = if remote_path.is_empty() { "/" } else { remote_path };
    let entries = sftp.read_dir(path).await?;

    let mut files: Vec<FileEntry> = Vec::new();
    for entry in entries {
        let meta = entry.metadata();
        let modified = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0)
            .to_string();
        let ft = entry.file_type();
        files.push(FileEntry {
            name: entry.file_name(),
            is_dir: ft.is_dir(),
            size: meta.size.unwrap_or(0),
            modified,
        });
    }

    files.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    Ok(files)
}

// ---- 删除文件/目录 ----

pub async fn delete_path(
    credentials: &(String, u16, String, String),
    remote_path: &str,
) -> Result<()> {
    let sftp = connect_sftp(credentials).await?;

    // 尝试删除文件
    match sftp.remove_file(remote_path).await {
        Ok(()) => return Ok(()),
        Err(_) => {}
    }

    // 尝试删除目录（递归）
    remove_dir_recursive(&sftp, remote_path).await
}

async fn remove_dir_recursive(sftp: &SftpSession, path: &str) -> Result<()> {
    let entries = sftp.read_dir(path).await?;
    for entry in entries {
        let name = entry.file_name();
        if name == "." || name == ".." {
            continue;
        }
        let full = format!("{}/{}", path.trim_end_matches('/'), name);
        if entry.file_type().is_dir() {
            let fut = remove_dir_recursive(sftp, &full);
            Box::pin(fut).await?;
        } else {
            sftp.remove_file(full).await?;
        }
    }
    sftp.remove_dir(path).await?;
    Ok(())
}

// ---- 重命名 ----

pub async fn rename_path(
    credentials: &(String, u16, String, String),
    old_path: &str,
    new_path: &str,
) -> Result<()> {
    let sftp = connect_sftp(credentials).await?;
    sftp.rename(old_path, new_path).await?;
    Ok(())
}

pub async fn create_dir(
    credentials: &(String, u16, String, String),
    remote_path: &str,
) -> Result<()> {
    let sftp = connect_sftp(credentials).await?;
    sftp.create_dir(remote_path).await?;
    Ok(())
}

// ---- 下载文件 ----

pub async fn download_file(
    credentials: &(String, u16, String, String),
    remote_path: &str,
    local_path: &str,
    app_handle: &AppHandle,
) -> Result<()> {
    let sftp = connect_sftp(credentials).await?;
    let file_name = Path::new(remote_path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let mut remote_file = sftp.open(remote_path).await?;
    let metadata = remote_file.metadata().await?;
    let total_size = metadata.size.unwrap_or(0);

    let mut local_file = tokio::fs::File::create(local_path).await?;
    let mut buf = vec![0u8; CHUNK_SIZE];
    let mut downloaded: u64 = 0;

    loop {
        let n = remote_file.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        local_file.write_all(&buf[..n]).await?;
        downloaded += n as u64;

        let percentage = if total_size > 0 {
            (downloaded as f64 / total_size as f64) * 100.0
        } else {
            100.0
        };

        let _ = app_handle.emit(
            "sftp-progress",
            SftpProgress {
                file_name: file_name.clone(),
                current: downloaded,
                total: total_size,
                percentage,
                status: "uploading".to_string(),
            },
        );
    }

    local_file.shutdown().await?;

    let _ = app_handle.emit(
        "sftp-progress",
        SftpProgress {
            file_name: file_name.clone(),
            current: downloaded,
            total: total_size,
            percentage: 100.0,
            status: "completed".to_string(),
        },
    );

    Ok(())
}

// ---- 上传文件（保留原有） ----

pub async fn upload_files(
    credentials: &(String, u16, String, String),
    local_paths: Vec<String>,
    remote_dir: String,
    app_handle: &AppHandle,
) -> Result<UploadResult> {
    let mut sftp = connect_sftp(credentials).await?;
    let mut success = Vec::new();
    let mut failed = Vec::new();

    for local_path in &local_paths {
        let path = Path::new(local_path);
        let file_name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let remote_path = format!("{}/{}", remote_dir.trim_end_matches('/'), file_name);

        match upload_one(&mut sftp, local_path, &remote_path, &file_name, app_handle).await {
            Ok(()) => success.push(file_name),
            Err(e) => {
                let msg = format!("{}: {}", file_name, e);
                failed.push(msg);
                let _ = app_handle.emit(
                    "sftp-progress",
                    SftpProgress {
                        file_name: file_name.clone(),
                        current: 0,
                        total: 0,
                        percentage: 0.0,
                        status: "error".to_string(),
                    },
                );
            }
        }
    }

    Ok(UploadResult { success, failed })
}

async fn upload_one(
    sftp: &mut SftpSession,
    local_path: &str,
    remote_path: &str,
    file_name: &str,
    app_handle: &AppHandle,
) -> Result<()> {
    let metadata = tokio::fs::metadata(local_path).await?;
    let total_size = metadata.len();

    let mut local_file = tokio::fs::File::open(local_path).await?;
    let mut remote_file = sftp.create(remote_path).await?;

    let mut buf = vec![0u8; CHUNK_SIZE];
    let mut uploaded: u64 = 0;

    loop {
        let n = local_file.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        remote_file.write_all(&buf[..n]).await?;
        uploaded += n as u64;

        let percentage = if total_size > 0 {
            (uploaded as f64 / total_size as f64) * 100.0
        } else {
            100.0
        };

        let _ = app_handle.emit(
            "sftp-progress",
            SftpProgress {
                file_name: file_name.to_string(),
                current: uploaded,
                total: total_size,
                percentage,
                status: "uploading".to_string(),
            },
        );
    }

    remote_file.shutdown().await?;

    let _ = app_handle.emit(
        "sftp-progress",
        SftpProgress {
            file_name: file_name.to_string(),
            current: uploaded,
            total: total_size,
            percentage: 100.0,
            status: "completed".to_string(),
        },
    );

    Ok(())
}
