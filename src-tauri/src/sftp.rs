use anyhow::{Context, Result};
use russh::*;
use russh_sftp::client::SftpSession;
use std::path::Path;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const CHUNK_SIZE: usize = 256 * 1024; // 256KB

/// 上传进度事件
#[derive(serde::Serialize, Clone, Debug)]
pub struct SftpProgress {
    pub file_name: String,
    pub current: u64,
    pub total: u64,
    pub percentage: f64,
    pub status: String, // "uploading" | "completed" | "error"
}

#[derive(serde::Serialize, Clone)]
pub struct UploadResult {
    pub success: Vec<String>,
    pub failed: Vec<String>,
}

/// 上传多个文件到远程服务器
pub async fn upload_files(
    credentials: &(String, u16, String, String),
    local_paths: Vec<String>,
    remote_dir: String,
    app_handle: &AppHandle,
) -> Result<UploadResult> {
    let (host, port, username, password) = credentials;
    let mut success = Vec::new();
    let mut failed = Vec::new();

    // 建立 SSH 连接
    let config = Arc::new(client::Config::default());
    let handler = crate::ssh::ClientHandler;

    let mut session = client::connect(config, (host.clone(), *port), handler)
        .await
        .context("SFTP 连接失败")?;

    session
        .authenticate_password(username, password)
        .await
        .context("SFTP 认证失败")?;

    // 打开通道并请求 SFTP 子系统
    let channel = session
        .channel_open_session()
        .await
        .context("无法打开通道")?;

    channel
        .request_subsystem(true, "sftp")
        .await
        .context("无法请求 SFTP 子系统")?;

    let stream = channel.into_stream();
    let mut sftp = SftpSession::new(stream)
        .await
        .context("无法初始化 SFTP 会话")?;

    // 逐个上传文件
    for local_path in &local_paths {
        let path = Path::new(local_path);
        let file_name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let remote_path = format!("{}/{}", remote_dir.trim_end_matches('/'), file_name);

        eprintln!("[SFTP] Uploading: {} -> {}", local_path, remote_path);

        match upload_one(&mut sftp, local_path, &remote_path, &file_name, app_handle).await {
            Ok(()) => success.push(file_name),
            Err(e) => {
                let msg = format!("{}: {}", file_name, e);
                eprintln!("[SFTP] Failed: {}", msg);
                failed.push(msg);
                // 通知前端上传失败
                let progress = SftpProgress {
                    file_name: file_name.clone(),
                    current: 0,
                    total: 0,
                    percentage: 0.0,
                    status: "error".to_string(),
                };
                let _ = app_handle.emit("sftp-progress", progress);
            }
        }
    }

    Ok(UploadResult { success, failed })
}

/// 上传单个文件，带进度回调
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

    // 创建远程文件（截断模式）
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

        let progress = SftpProgress {
            file_name: file_name.to_string(),
            current: uploaded,
            total: total_size,
            percentage,
            status: "uploading".to_string(),
        };
        let _ = app_handle.emit("sftp-progress", progress);
    }

    // 关闭远程文件
    remote_file.shutdown().await?;

    let progress = SftpProgress {
        file_name: file_name.to_string(),
        current: uploaded,
        total: total_size,
        percentage: 100.0,
        status: "completed".to_string(),
    };
    let _ = app_handle.emit("sftp-progress", progress);

    eprintln!(
        "[SFTP] Completed: {} ({}/{})",
        file_name,
        uploaded,
        total_size
    );

    Ok(())
}
