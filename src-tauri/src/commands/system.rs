use super::{err, CmdResult};
use tauri::Manager;

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
