use super::{err, CmdResult};
use crate::AppState;
use rusqlite::params;
use std::collections::HashMap;
use tauri::State;

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> CmdResult<HashMap<String, String>> {
    let conn = state.conn.lock().map_err(err)?;
    let mut stmt = conn.prepare("SELECT key, value FROM setting").map_err(err)?;
    let rows = stmt
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
        .map_err(err)?;
    let mut map = HashMap::new();
    for row in rows {
        let (k, v) = row.map_err(err)?;
        map.insert(k, v);
    }
    Ok(map)
}

#[tauri::command]
pub fn set_setting(state: State<AppState>, key: String, value: String) -> CmdResult<()> {
    let conn = state.conn.lock().map_err(err)?;
    conn.execute(
        "INSERT INTO setting (key, value) VALUES (?1, ?2) \
         ON CONFLICT(key) DO UPDATE SET value = ?2",
        params![key, value],
    )
    .map_err(err)?;
    Ok(())
}
