use anyhow::{Context, Result};
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use std::io::{Read, Write};
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;
use tokio::sync::mpsc;

// ---- 数据结构 ----

#[derive(serde::Serialize, Clone)]
pub struct LocalOutput {
    pub data: Vec<u8>,
}

enum LocalCommand {
    Write(Vec<u8>),
    Resize(u32, u32),
}

// ---- 状态 ----

pub struct LocalTermState {
    cmd_tx: Mutex<Option<mpsc::UnboundedSender<LocalCommand>>>,
}

impl LocalTermState {
    pub fn new() -> Self {
        Self {
            cmd_tx: Mutex::new(None),
        }
    }
}

// ---- 获取用户 Shell ----

fn get_user_shell() -> String {
    if cfg!(target_os = "windows") {
        std::env::var("COMSPEC")
            .unwrap_or_else(|_| "powershell.exe".to_string())
    } else {
        std::env::var("SHELL")
            .unwrap_or_else(|_| "/bin/bash".to_string())
    }
}

// ---- 启动本地终端 ----

pub async fn start(
    state: &LocalTermState,
    app_handle: AppHandle,
    rows: u32,
    cols: u32,
) -> Result<()> {
    // 如果已有连接，先关闭
    close(state).await.ok();

    let pty_system = native_pty_system();
    let pty_size = PtySize {
        rows: rows as u16,
        cols: cols as u16,
        pixel_width: 0,
        pixel_height: 0,
    };
    let pair = pty_system.openpty(pty_size)
        .context("无法创建 PTY")?;

    let shell = get_user_shell();
    let mut cmd = CommandBuilder::new(&shell);
    cmd.cwd(
        std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("/")),
    );
    cmd.env("TERM", "xterm-256color");

    let master = pair.master;
    let mut reader = master.try_clone_reader()
        .context("无法克隆 PTY 读取端")?;
    let mut writer = master.take_writer()
        .context("无法获取 PTY 写入端")?;

    let _child = pair.slave.spawn_command(cmd)
        .context(format!("无法启动 Shell: {}", shell))?;
    drop(pair.slave);

    let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel::<LocalCommand>();
    *state.cmd_tx.lock().await = Some(cmd_tx);

    // 读取线程：从 PTY master 读取数据并发送到前端
    let app = app_handle.clone();
    std::thread::spawn(move || {
        let mut buf = [0u8; 4096];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => {
                    eprintln!("[Local] PTY read EOF");
                    break;
                }
                Ok(n) => {
                    let output = LocalOutput { data: buf[..n].to_vec() };
                    let _ = app.emit("local-output", output);
                }
                Err(e) => {
                    eprintln!("[Local] PTY read error: {}", e);
                    break;
                }
            }
        }
    });

    // 命令处理线程：处理来自前端的写和 resize 命令
    std::thread::spawn(move || {
        loop {
            match cmd_rx.blocking_recv() {
                Some(LocalCommand::Write(data)) => {
                    if writer.write_all(&data).is_err() {
                        eprintln!("[Local] PTY write failed");
                        break;
                    }
                }
                Some(LocalCommand::Resize(rows, cols)) => {
                    let size = PtySize {
                        rows: rows as u16,
                        cols: cols as u16,
                        pixel_width: 0,
                        pixel_height: 0,
                    };
                    if master.resize(size).is_err() {
                        eprintln!("[Local] PTY resize failed");
                        break;
                    }
                }
                None => {
                    eprintln!("[Local] command channel closed");
                    break;
                }
            }
        }
        // 线程退出时，master、writer 和 _child 会被 drop
        // master drop → PTY 关闭 → child 进程收到 SIGHUP → 进程退出
        eprintln!("[Local] PTY command handler ended");
    });

    Ok(())
}

// ---- 写入 ----

pub async fn write(state: &LocalTermState, data: &[u8]) -> Result<()> {
    let tx = state.cmd_tx.lock().await;
    if let Some(ref tx) = *tx {
        tx.send(LocalCommand::Write(data.to_vec()))
            .map_err(|_| anyhow::anyhow!("本地终端已关闭"))?;
        Ok(())
    } else {
        Err(anyhow::anyhow!("尚未启动本地终端"))
    }
}

// ---- 调整终端大小 ----

pub async fn resize(state: &LocalTermState, rows: u32, cols: u32) -> Result<()> {
    let tx = state.cmd_tx.lock().await;
    if let Some(ref tx) = *tx {
        tx.send(LocalCommand::Resize(rows, cols))
            .map_err(|_| anyhow::anyhow!("本地终端已关闭"))?;
        Ok(())
    } else {
        Err(anyhow::anyhow!("尚未启动本地终端"))
    }
}

// ---- 关闭 ----

pub async fn close(state: &LocalTermState) -> Result<()> {
    let mut tx = state.cmd_tx.lock().await;
    *tx = None; // 丢弃 sender，命令处理线程的 recv 返回 None，线程退出
    Ok(())
}
