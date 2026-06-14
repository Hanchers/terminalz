mod commands;
mod db;
mod local_term;
mod ssh;
mod sftp;
mod sysinfo;

use crate::commands::*;
use db::DbState;
use local_term::LocalTermState;
use ssh::SshState;
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
