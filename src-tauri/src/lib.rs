mod db;
mod ssh;

use db::{ConnectionConfig, DbState};
use ssh::SshState;
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

// ---- 启动入口 ----

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
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
        .invoke_handler(tauri::generate_handler![
            ssh_connect,
            ssh_write,
            ssh_resize,
            ssh_disconnect,
            list_connections,
            save_connection,
            delete_connection,
        ])
        .run(tauri::generate_context!())
        .expect("启动 Tauri 应用失败");
}
