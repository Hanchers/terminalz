use anyhow::{Context, Result};
use log::{debug, info, warn};
use russh::{client, ChannelMsg};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::{mpsc, Mutex};

use crate::db;
use crate::terminal;
use crate::vault;

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

#[derive(serde::Serialize, Clone)]
struct ConnectStep { step: String, detail: String }

fn emit(s: &str, d: &str, app: &AppHandle) {
    debug!("connect-step: [{}] {}", s, d);
    let _ = app.emit("connect-step", ConnectStep { step: s.to_string(), detail: d.to_string() });
}

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
    disconnect(state).await.ok();

    debug!("SSH connect: {}@{}:{}  (pty {}x{})", username, host, port, cols, rows);
    emit("tcp", &format!("Connecting to {}:{}...", host, port), &app_handle);

    let config = Arc::new(client::Config::default());
    let handler = ClientHandler;

    const CONNECT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);
    const MAX_RETRIES: u32 = 3;

    let mut conn = None;
    for attempt in 1..=MAX_RETRIES {
        debug!("SSH TCP connect attempt {}/{}", attempt, MAX_RETRIES);
        let msg = format!("TCP attempt {}/{}", attempt, MAX_RETRIES);
        emit("tcp", &msg, &app_handle);
        match tokio::time::timeout(
            CONNECT_TIMEOUT,
            client::connect(config.clone(), (host.to_string(), port), handler.clone()),
        )
        .await
        {
            Ok(Ok(s)) => {
                debug!("SSH TCP connected (attempt {})", attempt);
                emit("tcp", "Connected", &app_handle);
                conn = Some(s);
                break;
            }
            Ok(Err(ref e)) if attempt < MAX_RETRIES => {
                warn!("SSH TCP attempt {}/{} failed: {} — retrying...", attempt, MAX_RETRIES, e);
                emit("tcp", &format!("Failed, retrying..."), &app_handle);
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
            Err(_) if attempt < MAX_RETRIES => {
                warn!("SSH TCP attempt {}/{} timed out — retrying...", attempt, MAX_RETRIES);
                emit("tcp", "Timed out, retrying...", &app_handle);
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
            Ok(Err(e)) => {
                warn!("SSH TCP all {} attempts failed: {}", MAX_RETRIES, e);
                return Err(e).context(format!("SSH connection failed after {} retries", MAX_RETRIES));
            }
            Err(_) => {
                warn!("SSH TCP all {} attempts timed out", MAX_RETRIES);
                return Err(anyhow::anyhow!("SSH connection timed out after {} retries", MAX_RETRIES));
            }
        }
    }

    let mut session = conn.context("SSH connection failed")?;

    debug!("SSH authenticating as {}...", username);
    emit("auth", &format!("Authenticating as {}...", username), &app_handle);

    // Try keyboard-interactive auth first (more compatible with PAM-based servers).
    // Falls back gracefully to simple password auth when the server doesn't use prompts.
    let none: Option<String> = None;
    match session
        .authenticate_keyboard_interactive_start(username, none)
        .await
    {
        Ok(client::KeyboardInteractiveAuthResponse::Success) => {
            debug!("SSH authenticated (keyboard-interactive, no prompts)");
        }
        Ok(client::KeyboardInteractiveAuthResponse::InfoRequest { prompts, .. }) => {
            debug!(
                "SSH keyboard-interactive: {} prompt(s), responding with password",
                prompts.len()
            );
            let responses: Vec<String> = prompts.iter().map(|_| password.to_owned()).collect();
            match session
                .authenticate_keyboard_interactive_respond(responses)
                .await
            {
                Ok(client::KeyboardInteractiveAuthResponse::Success) => {
                    debug!("SSH authenticated (keyboard-interactive, responded)");
                }
                Ok(client::KeyboardInteractiveAuthResponse::InfoRequest { prompts: prompts2, .. }) => {
                    debug!(
                        "SSH keyboard-interactive: {} more prompt(s)",
                        prompts2.len()
                    );
                    let r2: Vec<String> = prompts2.iter().map(|_| password.to_owned()).collect();
                    let final_resp =
                        session.authenticate_keyboard_interactive_respond(r2).await;
                    match final_resp {
                        Ok(client::KeyboardInteractiveAuthResponse::Success) => {
                            debug!("SSH authenticated (keyboard-interactive, 2nd response)");
                        }
                        other => {
                            warn!("SSH keyboard-interactive final response: {:?}", other);
                            return Err(anyhow::anyhow!(
                                "Authentication failed, check username and password"
                            ));
                        }
                    }
                }
                other => {
                    warn!("SSH keyboard-interactive response: {:?}", other);
                    return Err(anyhow::anyhow!(
                        "Authentication failed, check username and password"
                    ));
                }
            }
        }
        Ok(client::KeyboardInteractiveAuthResponse::Failure { .. }) => {
            // Keyboard-interactive failed, fall back to simple password auth
            debug!("SSH keyboard-interactive not available, trying password auth...");
            session
                .authenticate_password(username, password)
                .await
                .context("Authentication failed, check username and password")?;
            debug!("SSH authenticated (password)");
        }
        Err(e) => {
            return Err(e).context("Authentication failed, check username and password");
        }
    }

    emit("auth", "Authenticated ✓", &app_handle);

    debug!("SSH opening channel...");
    emit("channel", "Opening session channel...", &app_handle);
    let mut channel = session
        .channel_open_session()
        .await
        .context("Failed to open SSH channel")?;
    debug!("SSH channel opened");

    debug!("SSH requesting PTY {}x{}", cols, rows);
    emit("channel", "Allocating PTY...", &app_handle);
    channel
        .request_pty(true, "xterm-256color", cols, rows, 0, 0, &[])
        .await
        .context("Failed to request PTY")?;
    debug!("SSH PTY allocated");

    debug!("SSH starting shell...");
    emit("channel", "Starting shell...", &app_handle);
    channel
        .request_shell(false)
        .await
        .context("Failed to start shell")?;
    debug!("SSH shell started");
    emit("done", "Connected ✓", &app_handle);

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
        debug!("SSH IO loop started");
        loop {
            tokio::select! {
                msg = channel.wait() => {
                    match msg {
                        None => {
                            debug!("SSH IO loop: channel closed (None)");
                            break;
                        }
                        Some(ref m) => match m {
                            ChannelMsg::Data { data } => {
                                debug!("SSH recv: {} bytes", data.len());
                                let output = SshOutput { data: data.to_vec() };
                                let _ = app_handle.emit("ssh-output", output);
                            }
                            ChannelMsg::ExtendedData { data, .. } => {
                                debug!("SSH recv (stderr): {} bytes", data.len());
                                let output = SshOutput { data: data.to_vec() };
                                let _ = app_handle.emit("ssh-output", output);
                            }
                            ChannelMsg::Eof => {
                                debug!("SSH IO loop: EOF");
                                break;
                            }
                            ChannelMsg::Close => {
                                debug!("SSH IO loop: Close");
                                break;
                            }
                            ChannelMsg::ExitStatus { exit_status, .. } => {
                                debug!("SSH IO loop: exit status {}", exit_status);
                                break;
                            }
                            ChannelMsg::ExitSignal { signal_name, .. } => {
                                debug!("SSH IO loop: exit signal {:?}", signal_name);
                                break;
                            }
                            ChannelMsg::Success => {
                                debug!("SSH IO loop: Success ack");
                            }
                            ChannelMsg::Failure => {
                                debug!("SSH IO loop: Failure ack");
                            }
                            _ => {
                                debug!("SSH IO loop: unhandled message, ignoring");
                            }
                        },
                    }
                }
                cmd = cmd_rx.recv() => {
                    match cmd {
                        Some(terminal::Command::Write(data)) => {
                            debug!("SSH send: {} bytes", data.len());
                            if channel.data(&data[..]).await.is_err() {
                                debug!("SSH IO loop: write failed, exiting");
                                break;
                            }
                        }
                        Some(terminal::Command::Resize(r, c)) => {
                            debug!("SSH resize to {}x{}", c, r);
                            if channel.window_change(c, r, 0, 0).await.is_err() {
                                debug!("SSH IO loop: resize failed, exiting");
                                break;
                            }
                        }
                        None => {
                            debug!("SSH IO loop: command channel closed");
                            break;
                        }
                    }
                }
            }
        }
        debug!("SSH IO loop ended");
    });

    info!("SSH session established: {}@{}:{}", username, host, port);
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
    debug!("SSH disconnect requested");
    state.channel.close().await;
    *state.session.lock().await = None;
    *state.credentials.lock().await = None;
    debug!("SSH disconnected");
    Ok(())
}

// ---- Tauri commands ----

#[tauri::command]
pub(crate) async fn ssh_connect(
    state: tauri::State<'_, SshState>,
    app_handle: tauri::AppHandle,
    db: tauri::State<'_, db::DbState>,
    vault: tauri::State<'_, vault::Vault>,
    connection_id: i64,
    rows: u32,
    cols: u32,
) -> Result<(), String> {
    // Load connection config from DB and decrypt password via vault.
    let config = db
        .get_connection_internal(connection_id)
        .map_err(|e| format!("DB load failed: {}", e))?;

    // If keychain is linked, use keychain credentials
    let (username, password) = if config.keychain_id > 0 {
        match db.get_ssh_key_internal(config.keychain_id) {
            Ok(key) => {
                let user = if key.username.is_empty() { config.username.clone() } else { key.username };
                let pw = vault.load(connection_id, &key.password).unwrap_or_default();
                (user, pw)
            }
            Err(_) => {
                // Keychain entry lost, fall back to host's own password
                let pw = match vault.load(connection_id, &config.password) {
                    Ok(p) => p,
                    Err(e) => return Err(format!("Vault load failed: {}", e)),
                };
                (config.username.clone(), pw)
            }
        }
    } else {
        let pw = match vault.load(connection_id, &config.password) {
            Ok(p) => p,
            Err(e) if config.password.starts_with("__KC__:") => {
                db.update_password(connection_id, "").ok();
                return Err(format!(
                    "Keychain entry lost for this host — the stale reference has been cleared.\n\
                     Right-click the host → Edit → enter the password → Save, then reconnect.\n\
                     Original: {}",
                    e
                ));
            }
            Err(e) => return Err(format!("Vault load failed: {}", e)),
        };
        (config.username.clone(), pw)
    };

    connect(
        &state,
        app_handle,
        &config.host,
        config.port,
        &username,
        &password,
        rows,
        cols,
    )
    .await
    .map_err(|e| e.to_string())?;

    // Auto-execute snippet if configured (shell needs ~800ms to initialize)
    if config.auto_snippet_id > 0 {
        match db.get_snippet_content(config.auto_snippet_id) {
            Ok(content) if !content.is_empty() => {
                tokio::time::sleep(std::time::Duration::from_millis(800)).await;
                let _ = write(&state, content.as_bytes()).await;
                let _ = write(&state, b"\n").await;
            }
            Ok(_) => {}
            Err(e) => warn!("Failed to load auto-snippet {}: {}", config.auto_snippet_id, e),
        }
    }

    Ok(())
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
