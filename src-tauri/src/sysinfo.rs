use anyhow::{Context, Result};
use russh::*;
use std::sync::Arc;

use crate::ssh::ClientHandler;

/// 服务器实时状态
#[derive(serde::Serialize, Clone, Debug)]
pub struct SystemInfo {
    pub hostname: String,
    /// CPU 负载 (1min / 5min / 15min)
    pub load_1min: f64,
    pub load_5min: f64,
    pub load_15min: f64,
    /// CPU 使用率百分比 (0-100)
    pub cpu_pct: f64,
    /// 总内存 (MB)
    pub mem_total: u64,
    /// 已用内存 (MB)
    pub mem_used: u64,
    /// 可用内存 (MB)
    pub mem_avail: u64,
    /// 内存使用百分比
    pub mem_pct: f64,
    /// 磁盘挂载信息（每个挂载点一条）
    pub disks: Vec<DiskInfo>,
    /// 操作系统名称 (如 "Ubuntu 22.04.5 LTS")
    pub os_name: String,
    /// 内核版本
    pub kernel: String,
    /// 系统运行时间
    pub uptime: String,
}

/// 单个挂载点的磁盘信息
#[derive(serde::Serialize, Clone, Debug)]
pub struct DiskInfo {
    /// 挂载点路径 (如 "/", "/data")
    pub mount: String,
    /// 总容量 (MB)
    pub total: u64,
    /// 已用 (MB)
    pub used: u64,
    /// 使用百分比 (0-100)
    pub pct: f64,
}

/// 远程命令，用 <<<SEP>>> 分隔各段输出
const SYS_CMD: &str = r#"
echo '<<<LOADAVG>>>' && cat /proc/loadavg && \
echo '<<<MEM>>>' && free -m 2>/dev/null | grep '^Mem:' || echo 'N/A' && \
echo '<<<CPUCOUNT>>>' && grep -c processor /proc/cpuinfo 2>/dev/null || echo 1 && \
echo '<<<DISK>>>' && (df -BM -x tmpfs -x devtmpfs -x overlay -x squashfs 2>/dev/null | tail -n +2 || echo 'N/A') && \
echo '<<<OS>>>' && (cat /etc/os-release 2>/dev/null | grep -E '^NAME=|^VERSION=' || echo 'N/A') && \
echo '<<<KERNEL>>>' && uname -r && \
echo '<<<HOST>>>' && hostname && \
echo '<<<UPTIME>>>' && (uptime -p 2>/dev/null || uptime 2>/dev/null || echo 'N/A') && \
echo '<<<END>>>'
"#;

pub async fn get_system_info(credentials: &(String, u16, String, String)) -> Result<SystemInfo> {
    let (host, port, username, password) = credentials;
    let config = Arc::new(client::Config::default());
    let handler = crate::ssh::ClientHandler;

    let mut session = client::connect(config, (host.clone(), *port), handler)
        .await
        .context("系统信息查询连接失败")?;

    session
        .authenticate_password(username, password)
        .await
        .context("认证失败")?;

    let output = exec(&mut session, SYS_CMD)
        .await
        .context("获取系统信息失败")?;

    parse_system_info(&output)
}

/// 执行远程命令并收集输出
async fn exec(session: &mut client::Handle<ClientHandler>, cmd: &str) -> Result<String> {
    let mut channel = session
        .channel_open_session()
        .await
        .context("无法打开通道")?;

    channel.exec(true, cmd).await.context("执行命令失败")?;

    let mut output = Vec::new();
    loop {
        match channel.wait().await {
            None => break,
            Some(msg) => match msg {
                ChannelMsg::Data { data } => output.extend_from_slice(&data),
                ChannelMsg::ExtendedData { data, .. } => output.extend_from_slice(&data),
                ChannelMsg::Eof | ChannelMsg::Close | ChannelMsg::ExitStatus { .. } => break,
                _ => {}
            },
        }
    }

    String::from_utf8(output).context("输出不是有效的 UTF-8")
}

/// 按 <<<SECTION>>> 标记解析所有信息
fn parse_system_info(text: &str) -> Result<SystemInfo> {
    let section = |marker: &str| -> String {
        let start = text.find(marker).map(|i| i + marker.len()).unwrap_or(0);
        let end = text[start..]
            .find("<<<")
            .map(|i| start + i)
            .unwrap_or(text.len());
        text[start..end].trim().to_string()
    };

    let loadavg_raw = section("<<<LOADAVG>>>");
    let mem_raw = section("<<<MEM>>>");
    let cpu_count_raw = section("<<<CPUCOUNT>>>");
    let disk_raw = section("<<<DISK>>>");
    let os_raw = section("<<<OS>>>");
    let kernel_raw = section("<<<KERNEL>>>");
    let host_raw = section("<<<HOST>>>");
    let uptime_raw = section("<<<UPTIME>>>");

    // --- Load Average ---
    // 格式: "0.16 0.07 0.06 1/345 12345"
    let la: Vec<f64> = loadavg_raw
        .split_whitespace()
        .filter_map(|s| s.parse().ok())
        .collect();
    let (load_1min, load_5min, load_15min) = if la.len() >= 3 {
        (la[0], la[1], la[2])
    } else {
        (0.0, 0.0, 0.0)
    };

    // --- CPU Count ---
    let cpu_count: f64 = cpu_count_raw
        .split_whitespace()
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(2.0_f64)
        .max(1.0_f64);
    let cpu_pct = (load_1min / cpu_count * 100.0).min(100.0).max(0.0);

    // --- Memory ---
    // free -m: "Mem:      7821    1381     469      11    5970    5617"
    let mem_parts: Vec<u64> = mem_raw
        .split_whitespace()
        .skip(1)
        .filter_map(|s| s.parse().ok())
        .collect();

    let (mem_total, mem_used, mem_avail) = if mem_parts.len() >= 2 {
        let total = mem_parts[0];
        let used = mem_parts[1];
        // available is the 6th column (index 5 after skipping "Mem:")
        let avail = if mem_parts.len() >= 6 {
            mem_parts[5]
        } else {
            total.saturating_sub(used)
        };
        (total, used, avail)
    } else {
        (0, 0, 0)
    };
    let mem_pct = if mem_total > 0 {
        (mem_used as f64 / mem_total as f64 * 100.0).min(100.0)
    } else {
        0.0
    };

    // --- Disk ---
    // 格式每行: /dev/sda1  51200M 20480M 28672M  42% /
    let mut disks: Vec<DiskInfo> = Vec::new();
    for line in disk_raw.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 6 {
            continue;
        }
        let total = parse_mb(parts[1]);
        let used = parse_mb(parts[2]);
        let pct = parts[4]
            .trim_end_matches('%')
            .parse::<f64>()
            .unwrap_or(0.0)
            .min(100.0)
            .max(0.0);
        let mount = parts[5].to_string();
        if total == 0 && used == 0 {
            continue;
        }
        disks.push(DiskInfo { mount, total, used, pct });
    }
    // 至少保证有一个根分区
    if disks.is_empty() {
        disks.push(DiskInfo {
            mount: "/".into(),
            total: 0,
            used: 0,
            pct: 0.0,
        });
    }

    // --- OS Name ---
    let mut os_name = String::new();
    let mut os_version = String::new();
    for line in os_raw.lines() {
        let line = line.trim();
        if line.starts_with("NAME=") {
            os_name = line[5..].trim_matches('"').to_string();
        } else if line.starts_with("VERSION=") {
            os_version = line[8..].trim_matches('"').to_string();
        } else if line.starts_with("VERSION_ID=") && os_version.is_empty() {
            os_version = line[11..].trim_matches('"').to_string();
        }
    }
    let os_name = if os_name.is_empty() {
        "Unknown".to_string()
    } else if os_version.is_empty() {
        os_name
    } else {
        format!("{} {}", os_name, os_version)
    };

    // --- Kernel ---
    let kernel = kernel_raw.lines().next().unwrap_or("unknown").to_string();

    // --- Hostname ---
    let hostname = host_raw.lines().next().unwrap_or("unknown").to_string();

    // --- Uptime ---
    let uptime = uptime_raw.lines().next().unwrap_or("unknown").trim().to_string();
    // 如果不是 "up ..." 格式，尝试从 raw 中提取
    let uptime = if uptime.starts_with("up ") {
        uptime
    } else {
        // uptime 没有 -p 标志时输出是 "14:23:01 up 5 days,  3:42,  1 user,  load average: ..."
        uptime_raw
            .split("load average")
            .next()
            .unwrap_or(&uptime)
            .trim()
            .to_string()
    };

    Ok(SystemInfo {
        hostname,
        load_1min,
        load_5min,
        load_15min,
        cpu_pct,
        mem_total,
        mem_used,
        mem_avail,
        mem_pct,
        disks,
        os_name,
        kernel,
        uptime,
    })
}

/// 解析 "51200M" 或 "51200" 这样的值为 MB 数字
fn parse_mb(s: &str) -> u64 {
    let cleaned = s.trim_end_matches(|c: char| !c.is_ascii_digit());
    cleaned.parse().unwrap_or(0)
}
