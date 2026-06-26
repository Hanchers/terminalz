mod crypto;
mod db;
mod local_term;
mod models;
mod ssh;
mod sftp;
mod sysinfo;
mod terminal;
mod vault;

use db::DbState;
use local_term::LocalTermState;
use ssh::SshState;
use sftp::FileEntry;
use tauri::Manager;
use vault::Vault;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Init debug logging (no-op in release builds)
            #[cfg(debug_assertions)]
            {
                env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
                    .format_timestamp_millis()
                    .init();
            }

            let app_handle = app.handle();
            let app_data_dir = app_handle
                .path()
                .app_data_dir()
                .map_err(|e| e.to_string())?;
            let db_path = app_data_dir.join("terminalz.db");
            let db = DbState::new(&db_path).map_err(|e| e.to_string())?;

            // Init credential vault (keychain + AES fallback).
            let vault = Vault::new(&app_data_dir.to_string_lossy());

            app.manage(db);
            app.manage(vault);
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
            // DB — keychain
            db::list_ssh_keys,
            db::save_ssh_key,
            db::delete_ssh_key,
            // DB — port forward
            db::list_port_forwards,
            db::save_port_forward,
            db::delete_port_forward,
            // DB — snippets
            db::list_snippets,
            db::save_snippet,
            db::delete_snippet,
            // DB — helpers
            db::list_connections_compact,
        ])
        .run(tauri::generate_context!())
        .expect("启动 Tauri 应用失败");
}

// ---- 本地文件系统浏览 ----

#[tauri::command]
fn read_local_dir(path: String, app_handle: tauri::AppHandle) -> Result<Vec<FileEntry>, String> {
    let read = |p: &str| -> Result<std::fs::ReadDir, i32> {
        std::fs::read_dir(p).map_err(|e| e.raw_os_error().unwrap_or(0))
    };

    let entries = match read(&path) {
        Ok(entries) => entries,
        Err(os_err) => {
            // For macOS EPERM (os error 1), try well-known directories as fallback
            if cfg!(target_os = "macos") && os_err == 1 {
                let mut entries = None;
                // Try common system-resolved paths: home, desktop, documents
                for dir in ["home", "desktop", "document", "download"] {
                    let name: &str = dir;
                    let fallback = match name {
                        "home" => app_handle.path().home_dir(),
                        "desktop" => app_handle.path().desktop_dir(),
                        "document" => app_handle.path().document_dir(),
                        "download" => app_handle.path().download_dir(),
                        _ => continue,
                    };
                    if let Ok(p) = fallback {
                        if let Ok(ents) = std::fs::read_dir(&p) {
                            entries = Some(ents);
                            break;
                        }
                    }
                }
                match entries {
                    Some(ents) => ents,
                    None => {
                        let home = app_handle.path().home_dir().map_err(|e| e.to_string())?;
                        std::fs::read_dir(&home).map_err(|e| {
                            format!(
                                "Cannot access this directory (macOS permission denied).\n\
                                 Tip: grant Full Disk Access to your terminal in System Settings > Privacy & Security > Full Disk Access.\n\
                                 System error: {}",
                                e
                            )
                        })?
                    }
                }
            } else {
                // Non-macOS or non-EPERM: fall back to home dir
                let home = app_handle.path().home_dir().map_err(|e| e.to_string())?;
                std::fs::read_dir(&home)
                    .map_err(|e| format!("Cannot read directory ({}). Try navigating from your home folder.", e))?
            }
        }
    };

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
