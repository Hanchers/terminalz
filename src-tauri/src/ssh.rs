use anyhow::{Context, Result};
use russh::*;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::{mpsc, Mutex};

// ---- SSH Client Handler ----

struct ClientHandler;

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

enum SshCommand {
    Write(Vec<u8>),
    Resize(u32, u32),
}

// ---- 连接状态 ----

pub struct SshState {
    cmd_tx: Mutex<Option<mpsc::UnboundedSender<SshCommand>>>,
}

impl SshState {
    pub fn new() -> Self {
        Self {
            cmd_tx: Mutex::new(None),
        }
    }
}

// ---- 调试辅助：打印 ChannelMsg 类型 ----

fn channel_msg_name(msg: &ChannelMsg) -> &'static str {
    match msg {
        ChannelMsg::Open { .. } => "Open",
        ChannelMsg::Data { .. } => "Data",
        ChannelMsg::ExtendedData { .. } => "ExtendedData",
        ChannelMsg::Eof => "Eof",
        ChannelMsg::Close => "Close",
        ChannelMsg::RequestPty { .. } => "RequestPty",
        ChannelMsg::RequestShell { .. } => "RequestShell",
        ChannelMsg::Exec { .. } => "Exec",
        ChannelMsg::Signal { .. } => "Signal",
        ChannelMsg::RequestSubsystem { .. } => "RequestSubsystem",
        ChannelMsg::RequestX11 { .. } => "RequestX11",
        ChannelMsg::SetEnv { .. } => "SetEnv",
        ChannelMsg::WindowChange { .. } => "WindowChange",
        ChannelMsg::XonXoff { .. } => "XonXoff",
        _ => "Unknown",
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

    let mut session = client::connect(config, (host.to_owned(), port), handler)
        .await
        .context("SSH 连接失败")?;

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

    // 启动 shell（want_reply=false，rsah 不会等待确认，避免收到确认消息）
    channel
        .request_shell(false)
        .await
        .context("无法启动 Shell")?;

    // 创建命令通道
    let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel::<SshCommand>();
    *state.cmd_tx.lock().await = Some(cmd_tx);

    // 启动 IO 循环
    tokio::spawn(async move {
        eprintln!("[SSH] IO loop started");
        loop {
            tokio::select! {
                // 从 SSH 读取数据
                msg = channel.wait() => {
                    match msg {
                        None => {
                            eprintln!("[SSH] channel closed (None)");
                            break;
                        }
                        Some(ref m) => {
                            let name = channel_msg_name(m);
                            match m {
                                ChannelMsg::Data { data } => {
                                    eprintln!("[SSH] Data: {} bytes", data.len());
                                    let output = SshOutput { data: data.to_vec() };
                                    let _ = app_handle.emit("ssh-output", output);
                                }
                                ChannelMsg::ExtendedData { data, .. } => {
                                    eprintln!("[SSH] ExtendedData: {} bytes", data.len());
                                    let output = SshOutput { data: data.to_vec() };
                                    let _ = app_handle.emit("ssh-output", output);
                                }
                                ChannelMsg::Eof
                                | ChannelMsg::Close
                                | ChannelMsg::ExitStatus { .. }
                                | ChannelMsg::ExitSignal { .. } => {
                                    eprintln!("[SSH] {} - channel closing", name);
                                    break;
                                }
                                ChannelMsg::Success | ChannelMsg::Failure => {
                                    eprintln!("[SSH] {} - server acknowledgement, continuing", name);
                                    // SSH 服务器可能会发 Success/Failure 确认，不退出
                                }
                                _ => {
                                    eprintln!("[SSH] Ignoring message: {}", name);
                                    // 继续循环，不退出
                                }
                            }
                        }
                    }
                }
                // 从前端接收命令
                cmd = cmd_rx.recv() => {
                    match cmd {
                        Some(SshCommand::Write(data)) => {
                            eprintln!("[SSH] writing {} bytes", data.len());
                            if channel.data(&data[..]).await.is_err() {
                                eprintln!("[SSH] write failed");
                                break;
                            }
                        }
                        Some(SshCommand::Resize(r, c)) => {
                            eprintln!("[SSH] resize to {}x{}", c, r);
                            if channel.window_change(c, r, 0, 0).await.is_err() {
                                eprintln!("[SSH] resize failed");
                                break;
                            }
                        }
                        None => {
                            eprintln!("[SSH] command channel closed");
                            break;
                        }
                    }
                }
            }
        }
        eprintln!("[SSH] IO loop ended");
    });

    Ok(())
}

// ---- 写入 ----

pub async fn write(state: &SshState, data: &[u8]) -> Result<()> {
    let tx = state.cmd_tx.lock().await;
    if let Some(ref tx) = *tx {
        tx.send(SshCommand::Write(data.to_vec()))
            .map_err(|_| anyhow::anyhow!("SSH 连接已断开"))?;
        Ok(())
    } else {
        Err(anyhow::anyhow!("尚未建立连接"))
    }
}

// ---- 调整终端大小 ----

pub async fn resize(state: &SshState, rows: u32, cols: u32) -> Result<()> {
    let tx = state.cmd_tx.lock().await;
    if let Some(ref tx) = *tx {
        tx.send(SshCommand::Resize(rows, cols))
            .map_err(|_| anyhow::anyhow!("SSH 连接已断开"))?;
        Ok(())
    } else {
        Err(anyhow::anyhow!("尚未建立连接"))
    }
}

// ---- 断开 ----

pub async fn disconnect(state: &SshState) -> Result<()> {
    let mut tx = state.cmd_tx.lock().await;
    *tx = None; // 丢弃 sender，IO 循环的 recv 会返回 None，从而退出
    Ok(())
}
