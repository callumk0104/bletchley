use super::{err, CmdResult};
use tauri::Manager;
use tauri_plugin_autostart::ManagerExt;

/// Write a timestamped copy of the database file and return its full path.
#[tauri::command]
pub fn backup_now(app: tauri::AppHandle) -> CmdResult<String> {
    let dir = app.path().app_data_dir().map_err(err)?;
    let db_path = dir.join("timesheet.db");
    let dest = crate::backup::backup_now(&db_path).map_err(err)?;
    Ok(dest.to_string_lossy().to_string())
}

/// The folder where backups are written (shown in Settings).
#[tauri::command]
pub fn backups_path(app: tauri::AppHandle) -> CmdResult<String> {
    let dir = app.path().app_data_dir().map_err(err)?;
    Ok(dir.join("backups").to_string_lossy().to_string())
}

/// Update the tray tooltip — called from the UI with the running day total.
#[tauri::command]
pub fn update_tray(app: tauri::AppHandle, text: String) -> CmdResult<()> {
    if let Some(tray) = app.tray_by_id("main-tray") {
        tray.set_tooltip(Some(text)).map_err(err)?;
    }
    Ok(())
}

/// Whether the app is registered to launch at login.
#[tauri::command]
pub fn autostart_is_enabled(app: tauri::AppHandle) -> CmdResult<bool> {
    app.autolaunch().is_enabled().map_err(err)
}

/// Enable or disable launch-at-login. When enabled the app starts hidden in
/// the tray (the registration passes `--minimized`).
#[tauri::command]
pub fn autostart_set(app: tauri::AppHandle, enabled: bool) -> CmdResult<()> {
    let mgr = app.autolaunch();
    if enabled {
        mgr.enable().map_err(err)
    } else {
        mgr.disable().map_err(err)
    }
}
