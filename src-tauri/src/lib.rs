mod db;
mod local_term;
mod ssh;
mod sftp;
mod sysinfo;

use db::{ConnectionConfig, DbState, HostGroup, Tag};
use local_term::LocalTermState;
use ssh::SshState;
use sftp::{FileEntry, UploadResult};
use sysinfo::SystemInfo;
use tauri::Manager;

// ---- SSH 命令 ----

#[tauri::command]
async fn ssh_connect(
    state: tauri::State<'_, SshState>,
    app_handle: tauri::AppHandle,
    host: String,
    port: u16,
    username: String,
    password: String,
    rows: u32,
    cols: u32,
) -> Result<(), String> {
    ssh::connect(&state, app_handle, &host, port, &username, &password, rows, cols)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn ssh_write(
    state: tauri::State<'_, SshState>,
    data: Vec<u8>,
) -> Result<(), String> {
    ssh::write(&state, &data).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn ssh_resize(
    state: tauri::State<'_, SshState>,
    rows: u32,
    cols: u32,
) -> Result<(), String> {
    ssh::resize(&state, rows, cols).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn ssh_disconnect(
    state: tauri::State<'_, SshState>,
) -> Result<(), String> {
    ssh::disconnect(&state).await.map_err(|e| e.to_string())
}

// ---- 本地终端命令 ----

#[tauri::command]
async fn local_term_start(
    state: tauri::State<'_, LocalTermState>,
    app_handle: tauri::AppHandle,
    rows: u32,
    cols: u32,
) -> Result<(), String> {
    local_term::start(&state, app_handle, rows, cols)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn local_term_write(
    state: tauri::State<'_, LocalTermState>,
    data: Vec<u8>,
) -> Result<(), String> {
    local_term::write(&state, &data).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn local_term_resize(
    state: tauri::State<'_, LocalTermState>,
    rows: u32,
    cols: u32,
) -> Result<(), String> {
    local_term::resize(&state, rows, cols).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn local_term_close(
    state: tauri::State<'_, LocalTermState>,
) -> Result<(), String> {
    local_term::close(&state).await.map_err(|e| e.to_string())
}

// ---- SFTP 命令 ----

#[tauri::command]
async fn sftp_list_dir(
    state: tauri::State<'_, SshState>,
    remote_path: String,
) -> Result<Vec<FileEntry>, String> {
    let credentials = state
        .credentials
        .lock()
        .await
        .clone()
        .ok_or_else(|| "请先建立 SSH 连接".to_string())?;
    sftp::list_dir(&credentials, &remote_path)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn sftp_delete(
    state: tauri::State<'_, SshState>,
    remote_path: String,
) -> Result<(), String> {
    let credentials = state
        .credentials
        .lock()
        .await
        .clone()
        .ok_or_else(|| "请先建立 SSH 连接".to_string())?;
    sftp::delete_path(&credentials, &remote_path)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn sftp_rename(
    state: tauri::State<'_, SshState>,
    old_path: String,
    new_path: String,
) -> Result<(), String> {
    let credentials = state
        .credentials
        .lock()
        .await
        .clone()
        .ok_or_else(|| "请先建立 SSH 连接".to_string())?;
    sftp::rename_path(&credentials, &old_path, &new_path)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn sftp_mkdir(
    state: tauri::State<'_, SshState>,
    remote_path: String,
) -> Result<(), String> {
    let credentials = state
        .credentials
        .lock()
        .await
        .clone()
        .ok_or_else(|| "请先建立 SSH 连接".to_string())?;
    sftp::create_dir(&credentials, &remote_path)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn sftp_download(
    state: tauri::State<'_, SshState>,
    app_handle: tauri::AppHandle,
    remote_path: String,
    local_path: String,
) -> Result<(), String> {
    let credentials = state
        .credentials
        .lock()
        .await
        .clone()
        .ok_or_else(|| "请先建立 SSH 连接".to_string())?;
    sftp::download_file(&credentials, &remote_path, &local_path, &app_handle)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn sftp_upload(
    state: tauri::State<'_, SshState>,
    app_handle: tauri::AppHandle,
    local_paths: Vec<String>,
    remote_dir: String,
) -> Result<UploadResult, String> {
    // 获取已保存的凭据
    let credentials = state
        .credentials
        .lock()
        .await
        .clone()
        .ok_or_else(|| "请先建立 SSH 连接".to_string())?;

    sftp::upload_files(&credentials, local_paths, remote_dir, &app_handle)
        .await
        .map_err(|e| e.to_string())
}

// ---- 系统信息命令 ----

#[tauri::command]
async fn sys_info(
    state: tauri::State<'_, SshState>,
) -> Result<SystemInfo, String> {
    let credentials = state
        .credentials
        .lock()
        .await
        .clone()
        .ok_or_else(|| "请先建立 SSH 连接".to_string())?;

    sysinfo::get_system_info(&credentials)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn local_sys_info() -> Result<SystemInfo, String> {
    // 本地系统信息采集可能耗时，放到阻塞线程中执行
    tokio::task::spawn_blocking(|| sysinfo::get_local_system_info())
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

// ---- 本地文件系统命令 ----

#[tauri::command]
fn read_local_dir(path: String) -> Result<Vec<FileEntry>, String> {
    let entries = std::fs::read_dir(&path).map_err(|e| e.to_string())?;
    let mut files: Vec<FileEntry> = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let meta = entry.metadata().map_err(|e| e.to_string())?;
        let modified = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0)
            .to_string();
        files.push(FileEntry {
            name: entry.file_name().to_string_lossy().to_string(),
            is_dir: meta.is_dir(),
            size: meta.len(),
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

// ---- 数据库命令 ----

#[tauri::command]
fn list_connections(db: tauri::State<'_, DbState>) -> Result<Vec<ConnectionConfig>, String> {
    db.list_all().map_err(|e| e.to_string())
}

#[tauri::command]
fn save_connection(
    db: tauri::State<'_, DbState>,
    config: ConnectionConfig,
) -> Result<ConnectionConfig, String> {
    let new_id = db.save(&config).map_err(|e| e.to_string())?;
    Ok(ConnectionConfig { id: new_id, ..config })
}

#[tauri::command]
fn delete_connection(db: tauri::State<'_, DbState>, id: i64) -> Result<(), String> {
    db.delete(id).map_err(|e| e.to_string())
}

// ---- 分组命令 ----

#[tauri::command]
fn list_groups(db: tauri::State<'_, DbState>) -> Result<Vec<HostGroup>, String> {
    db.list_groups().map_err(|e| e.to_string())
}

#[tauri::command]
fn save_group(db: tauri::State<'_, DbState>, group: HostGroup) -> Result<HostGroup, String> {
    let new_id = db.save_group(&group).map_err(|e| e.to_string())?;
    Ok(HostGroup { id: new_id, ..group })
}

#[tauri::command]
fn delete_group(db: tauri::State<'_, DbState>, id: i64) -> Result<(), String> {
    // 检查是否有子分组
    if db.has_child_groups(id).map_err(|e| e.to_string())? {
        return Err("该分组下存在子分组，请先删除子分组".to_string());
    }
    // 检查是否有 host
    let count = db.count_hosts_in_group(id).map_err(|e| e.to_string())?;
    if count > 0 {
        return Err(format!("该分组及子分组下存在 {} 个 host，请先移除这些 host", count));
    }
    db.delete_group(id).map_err(|e| e.to_string())
}

// ---- 标签命令 ----

#[tauri::command]
fn list_tags(db: tauri::State<'_, DbState>) -> Result<Vec<Tag>, String> {
    db.list_tags().map_err(|e| e.to_string())
}

#[tauri::command]
fn save_tag(
    db: tauri::State<'_, DbState>,
    name: String,
    color: String,
) -> Result<Tag, String> {
    db.save_tag(&name, &color).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_tag(db: tauri::State<'_, DbState>, id: i64) -> Result<(), String> {
    db.delete_tag(id).map_err(|e| e.to_string())
}

#[tauri::command]
fn update_tag(
    db: tauri::State<'_, DbState>,
    id: i64,
    name: String,
    color: String,
) -> Result<(), String> {
    db.update_tag(id, &name, &color).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_host_tags(db: tauri::State<'_, DbState>, host_id: i64) -> Result<Vec<Tag>, String> {
    db.get_host_tags(host_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn set_host_tags(
    db: tauri::State<'_, DbState>,
    host_id: i64,
    tag_ids: Vec<i64>,
) -> Result<(), String> {
    db.set_host_tags(host_id, &tag_ids).map_err(|e| e.to_string())
}

// ---- 启动入口 ----

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_handle = app.handle();
            let db_path = app_handle
                .path()
                .app_data_dir()
                .map_err(|e| e.to_string())?
                .join("terminalz.db");
            let db = DbState::new(&db_path).map_err(|e| e.to_string())?;
            app.manage(db);
            Ok(())
        })
        .manage(SshState::new())
        .manage(LocalTermState::new())
        .invoke_handler(tauri::generate_handler![
            ssh_connect,
            ssh_write,
            ssh_resize,
            ssh_disconnect,
            local_term_start,
            local_term_write,
            local_term_resize,
            local_term_close,
            sftp_list_dir,
            sftp_delete,
            sftp_rename,
            sftp_mkdir,
            sftp_download,
            sftp_upload,
            sys_info,
            local_sys_info,
            read_local_dir,
            list_connections,
            save_connection,
            delete_connection,
            list_groups,
            save_group,
            delete_group,
            list_tags,
            save_tag,
            delete_tag,
            update_tag,
            get_host_tags,
            set_host_tags,
        ])
        .run(tauri::generate_context!())
        .expect("启动 Tauri 应用失败");
}
