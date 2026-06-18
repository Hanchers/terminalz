use anyhow::{Context, Result};
use russh::*;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::{mpsc, Mutex};

use crate::terminal;

// ---- SSH Client Handler ----

#[derive(Clone)]
pub(crate) struct ClientHandler;

impl client::Handler for ClientHandler {
    type Error = anyhow::Error;

    fn check_server_key(
        &mut self,
        _server_public_key: &ssh_key::PublicKey,
    ) -> impl std::future::Future<Output = Result<bool, Self::Error>> + Send {
        async { Ok(true) }
    }
}

// ---- 数据结构 ----

#[derive(serde::Serialize, Clone)]
pub struct SshOutput {
    pub data: Vec<u8>,
}

// ---- 连接状态 ----

pub struct SshState {
    channel: terminal::Channel,
    /// Saved credentials for SFTP / sysinfo operations.
    pub(crate) credentials: Mutex<Option<(String, u16, String, String)>>,
    /// Cached SSH session handle so SFTP ops can open channels without
    /// re-establishing TCP + key-exchange + auth.
    /// Wrapped in Arc so the lock is held only for the clone, not across I/O.
    pub(crate) session: Mutex<Option<std::sync::Arc<client::Handle<ClientHandler>>>>,
}

impl SshState {
    pub fn new() -> Self {
        Self {
            channel: terminal::Channel::new(),
            credentials: Mutex::new(None),
            session: Mutex::new(None),
        }
    }
}

// ---- 连接 ----

pub async fn connect(
    state: &SshState,
    app_handle: AppHandle,
    host: &str,
    port: u16,
    username: &str,
    password: &str,
    rows: u32,
    cols: u32,
) -> Result<()> {
    // 如果已有连接，先断开
    disconnect(state).await.ok();

    let config = Arc::new(client::Config::default());
    let handler = ClientHandler;

    const CONNECT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);
    const MAX_RETRIES: u32 = 3;

    let mut conn = None;
    for attempt in 1..=MAX_RETRIES {
        match tokio::time::timeout(
            CONNECT_TIMEOUT,
            client::connect(config.clone(), (host.to_string(), port), handler.clone()),
        )
        .await
        {
            Ok(Ok(s)) => {
                conn = Some(s);
                break;
            }
            Ok(Err(_)) | Err(_) if attempt < MAX_RETRIES => {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
            Ok(Err(e)) => {
                return Err(e).context(format!("SSH 连接失败（已重试 {} 次）", MAX_RETRIES));
            }
            Err(_) => {
                return Err(anyhow::anyhow!("SSH 连接超时（已重试 {} 次）", MAX_RETRIES));
            }
        }
    }

    let mut session = conn.context("SSH 连接失败")?;

    session
        .authenticate_password(username, password)
        .await
        .context("认证失败，请检查用户名和密码")?;

    let mut channel = session
        .channel_open_session()
        .await
        .context("无法打开 SSH 通道")?;

    // 请求 PTY（want_reply=true 确保 PTY 分配成功）
    channel
        .request_pty(true, "xterm-256color", cols, rows, 0, 0, &[])
        .await
        .context("无法请求 PTY")?;

    // 启动 shell
    channel
        .request_shell(false)
        .await
        .context("无法启动 Shell")?;

    // 创建命令通道
    let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel::<terminal::Command>();
    state.channel.set(cmd_tx).await;

    // 保存凭据和会话句柄（供 SFTP / sysinfo 复用）
    *state.credentials.lock().await = Some((
        host.to_owned(),
        port,
        username.to_owned(),
        password.to_owned(),
    ));
    *state.session.lock().await = Some(std::sync::Arc::new(session));

    // 启动 IO 循环
    tokio::spawn(async move {
        loop {
            tokio::select! {
                msg = channel.wait() => {
                    match msg {
                        None => break,
                        Some(ref m) => match m {
                            ChannelMsg::Data { data } => {
                                let output = SshOutput { data: data.to_vec() };
                                let _ = app_handle.emit("ssh-output", output);
                            }
                            ChannelMsg::ExtendedData { data, .. } => {
                                let output = SshOutput { data: data.to_vec() };
                                let _ = app_handle.emit("ssh-output", output);
                            }
                            ChannelMsg::Eof
                            | ChannelMsg::Close
                            | ChannelMsg::ExitStatus { .. }
                            | ChannelMsg::ExitSignal { .. } => break,
                            ChannelMsg::Success | ChannelMsg::Failure => {
                                // server acknowledgement — keep listening
                            }
                            _ => {
                                // unknown message — keep listening
                            }
                        },
                    }
                }
                cmd = cmd_rx.recv() => {
                    match cmd {
                        Some(terminal::Command::Write(data)) => {
                            if channel.data(&data[..]).await.is_err() {
                                break;
                            }
                        }
                        Some(terminal::Command::Resize(r, c)) => {
                            if channel.window_change(c, r, 0, 0).await.is_err() {
                                break;
                            }
                        }
                        None => break,
                    }
                }
            }
        }
    });

    Ok(())
}

// ---- 写入 / 调整大小 / 断开 ----

pub async fn write(state: &SshState, data: &[u8]) -> Result<()> {
    state.channel.write(data).await
}

pub async fn resize(state: &SshState, rows: u32, cols: u32) -> Result<()> {
    state.channel.resize(rows, cols).await
}

pub async fn disconnect(state: &SshState) -> Result<()> {
    state.channel.close().await;
    *state.session.lock().await = None;
    *state.credentials.lock().await = None;
    Ok(())
}

// ---- Tauri commands ----

#[tauri::command]
pub(crate) async fn ssh_connect(
    state: tauri::State<'_, SshState>,
    app_handle: tauri::AppHandle,
    host: String,
    port: u16,
    username: String,
    password: String,
    rows: u32,
    cols: u32,
) -> Result<(), String> {
    connect(&state, app_handle, &host, port, &username, &password, rows, cols)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub(crate) async fn ssh_write(
    state: tauri::State<'_, SshState>,
    data: Vec<u8>,
) -> Result<(), String> {
    write(&state, &data).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub(crate) async fn ssh_resize(
    state: tauri::State<'_, SshState>,
    rows: u32,
    cols: u32,
) -> Result<(), String> {
    resize(&state, rows, cols).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub(crate) async fn ssh_disconnect(
    state: tauri::State<'_, SshState>,
) -> Result<(), String> {
    disconnect(&state).await.map_err(|e| e.to_string())
}
