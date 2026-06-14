use crate::db::{ConnectionConfig, DbState, HostGroup, Tag};
use crate::local_term::LocalTermState;
use crate::ssh::SshState;
use crate::sftp::{FileEntry, UploadResult};
use crate::sysinfo::SystemInfo;

// ---- SSH 命令 ----

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
    crate::ssh::connect(&state, app_handle, &host, port, &username, &password, rows, cols)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub(crate) async fn ssh_write(
    state: tauri::State<'_, SshState>,
    data: Vec<u8>,
) -> Result<(), String> {
    crate::ssh::write(&state, &data).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub(crate) async fn ssh_resize(
    state: tauri::State<'_, SshState>,
    rows: u32,
    cols: u32,
) -> Result<(), String> {
    crate::ssh::resize(&state, rows, cols).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub(crate) async fn ssh_disconnect(
    state: tauri::State<'_, SshState>,
) -> Result<(), String> {
    crate::ssh::disconnect(&state).await.map_err(|e| e.to_string())
}

// ---- 本地终端命令 ----

#[tauri::command]
pub(crate) async fn local_term_start(
    state: tauri::State<'_, LocalTermState>,
    app_handle: tauri::AppHandle,
    rows: u32,
    cols: u32,
) -> Result<(), String> {
    crate::local_term::start(&state, app_handle, rows, cols)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub(crate) async fn local_term_write(
    state: tauri::State<'_, LocalTermState>,
    data: Vec<u8>,
) -> Result<(), String> {
    crate::local_term::write(&state, &data).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub(crate) async fn local_term_resize(
    state: tauri::State<'_, LocalTermState>,
    rows: u32,
    cols: u32,
) -> Result<(), String> {
    crate::local_term::resize(&state, rows, cols).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub(crate) async fn local_term_close(
    state: tauri::State<'_, LocalTermState>,
) -> Result<(), String> {
    crate::local_term::close(&state).await.map_err(|e| e.to_string())
}

// ---- SFTP 命令 ----

async fn get_ssh_creds(state: &tauri::State<'_, SshState>) -> Result<(String, u16, String, String), String> {
    state.credentials.lock().await.clone()
        .ok_or_else(|| "请先建立 SSH 连接".to_string())
}

// ---- SFTP 命令 ----

#[tauri::command]
pub(crate) async fn sftp_list_dir(
    state: tauri::State<'_, SshState>,
    remote_path: String,
) -> Result<Vec<FileEntry>, String> {
    let creds = get_ssh_creds(&state).await?;
    crate::sftp::list_dir(&creds, &remote_path).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub(crate) async fn sftp_delete(
    state: tauri::State<'_, SshState>,
    remote_path: String,
) -> Result<(), String> {
    let creds = get_ssh_creds(&state).await?;
    crate::sftp::delete_path(&creds, &remote_path).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub(crate) async fn sftp_rename(
    state: tauri::State<'_, SshState>,
    old_path: String,
    new_path: String,
) -> Result<(), String> {
    let creds = get_ssh_creds(&state).await?;
    crate::sftp::rename_path(&creds, &old_path, &new_path).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub(crate) async fn sftp_mkdir(
    state: tauri::State<'_, SshState>,
    remote_path: String,
) -> Result<(), String> {
    let creds = get_ssh_creds(&state).await?;
    crate::sftp::create_dir(&creds, &remote_path).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub(crate) async fn sftp_download(
    state: tauri::State<'_, SshState>,
    app_handle: tauri::AppHandle,
    remote_path: String,
    local_path: String,
) -> Result<(), String> {
    let creds = get_ssh_creds(&state).await?;
    crate::sftp::download_file(&creds, &remote_path, &local_path, &app_handle)
        .await.map_err(|e| e.to_string())
}

#[tauri::command]
pub(crate) async fn sftp_upload(
    state: tauri::State<'_, SshState>,
    app_handle: tauri::AppHandle,
    local_paths: Vec<String>,
    remote_dir: String,
) -> Result<UploadResult, String> {
    let creds = get_ssh_creds(&state).await?;
    crate::sftp::upload_files(&creds, local_paths, remote_dir, &app_handle)
        .await.map_err(|e| e.to_string())
}

// ---- 系统信息命令 ----

#[tauri::command]
pub(crate) async fn sys_info(
    state: tauri::State<'_, SshState>,
) -> Result<SystemInfo, String> {
    let creds = get_ssh_creds(&state).await?;
    crate::sysinfo::get_system_info(&creds).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub(crate) async fn local_sys_info() -> Result<SystemInfo, String> {
    tokio::task::spawn_blocking(|| crate::sysinfo::get_local_system_info())
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

// ---- 本地文件系统命令 ----

#[tauri::command]
pub(crate) fn read_local_dir(path: String) -> Result<Vec<FileEntry>, String> {
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
pub(crate) fn list_connections(db: tauri::State<'_, DbState>) -> Result<Vec<ConnectionConfig>, String> {
    db.list_all().map_err(|e| e.to_string())
}

#[tauri::command]
pub(crate) fn save_connection(
    db: tauri::State<'_, DbState>,
    config: ConnectionConfig,
) -> Result<ConnectionConfig, String> {
    let new_id = db.save(&config).map_err(|e| e.to_string())?;
    Ok(ConnectionConfig { id: new_id, ..config })
}

#[tauri::command]
pub(crate) fn delete_connection(db: tauri::State<'_, DbState>, id: i64) -> Result<(), String> {
    db.delete(id).map_err(|e| e.to_string())
}

// ---- 分组命令 ----

#[tauri::command]
pub(crate) fn list_groups(db: tauri::State<'_, DbState>) -> Result<Vec<HostGroup>, String> {
    db.list_groups().map_err(|e| e.to_string())
}

#[tauri::command]
pub(crate) fn save_group(db: tauri::State<'_, DbState>, group: HostGroup) -> Result<HostGroup, String> {
    let new_id = db.save_group(&group).map_err(|e| e.to_string())?;
    Ok(HostGroup { id: new_id, ..group })
}

#[tauri::command]
pub(crate) fn delete_group(db: tauri::State<'_, DbState>, id: i64) -> Result<(), String> {
    if db.has_child_groups(id).map_err(|e| e.to_string())? {
        return Err("该分组下存在子分组，请先删除子分组".to_string());
    }
    let count = db.count_hosts_in_group(id).map_err(|e| e.to_string())?;
    if count > 0 {
        return Err(format!("该分组及子分组下存在 {} 个 host，请先移除这些 host", count));
    }
    db.delete_group(id).map_err(|e| e.to_string())
}

// ---- 标签命令 ----

#[tauri::command]
pub(crate) fn list_tags(db: tauri::State<'_, DbState>) -> Result<Vec<Tag>, String> {
    db.list_tags().map_err(|e| e.to_string())
}

#[tauri::command]
pub(crate) fn save_tag(
    db: tauri::State<'_, DbState>,
    name: String,
    color: String,
) -> Result<Tag, String> {
    db.save_tag(&name, &color).map_err(|e| e.to_string())
}

#[tauri::command]
pub(crate) fn delete_tag(db: tauri::State<'_, DbState>, id: i64) -> Result<(), String> {
    db.delete_tag(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub(crate) fn update_tag(
    db: tauri::State<'_, DbState>,
    id: i64,
    name: String,
    color: String,
) -> Result<(), String> {
    db.update_tag(id, &name, &color).map_err(|e| e.to_string())
}

#[tauri::command]
pub(crate) fn get_host_tags(db: tauri::State<'_, DbState>, host_id: i64) -> Result<Vec<Tag>, String> {
    db.get_host_tags(host_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub(crate) fn set_host_tags(
    db: tauri::State<'_, DbState>,
    host_id: i64,
    tag_ids: Vec<i64>,
) -> Result<(), String> {
    db.set_host_tags(host_id, &tag_ids).map_err(|e| e.to_string())
}
