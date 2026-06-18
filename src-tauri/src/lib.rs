mod db;
mod local_term;
mod models;
mod ssh;
mod sftp;
mod sysinfo;
mod terminal;

use db::DbState;
use local_term::LocalTermState;
use ssh::SshState;
use sftp::FileEntry;
use tauri::Manager;

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
            // SSH
            ssh::ssh_connect,
            ssh::ssh_write,
            ssh::ssh_resize,
            ssh::ssh_disconnect,
            // Local terminal
            local_term::local_term_start,
            local_term::local_term_write,
            local_term::local_term_resize,
            local_term::local_term_close,
            // SFTP
            sftp::sftp_list_dir,
            sftp::sftp_delete,
            sftp::sftp_rename,
            sftp::sftp_mkdir,
            sftp::sftp_download,
            sftp::sftp_upload,
            // System info
            sysinfo::sys_info,
            sysinfo::local_sys_info,
            // Local filesystem
            read_local_dir,
            // DB — connections
            db::list_connections,
            db::save_connection,
            db::delete_connection,
            // DB — groups
            db::list_groups,
            db::save_group,
            db::delete_group,
            // DB — tags
            db::list_tags,
            db::save_tag,
            db::delete_tag,
            db::update_tag,
            db::get_host_tags,
            db::list_all_host_tags,
            db::set_host_tags,
        ])
        .run(tauri::generate_context!())
        .expect("启动 Tauri 应用失败");
}

// ---- 本地文件系统浏览 ----

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
