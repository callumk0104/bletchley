use super::{err, timecode_from_row, CmdResult};
use crate::models::Timecode;
use crate::AppState;
use rusqlite::params;
use tauri::State;

#[tauri::command]
pub fn list_timecodes(state: State<AppState>, active_only: bool) -> CmdResult<Vec<Timecode>> {
    let conn = state.conn.lock().map_err(err)?;
    let sql = if active_only {
        "SELECT id, client, project, task, active, hidden FROM timecode \
         WHERE active = 1 ORDER BY client, project, task"
    } else {
        "SELECT id, client, project, task, active, hidden FROM timecode \
         ORDER BY active DESC, client, project, task"
    };
    let mut stmt = conn.prepare(sql).map_err(err)?;
    let rows = stmt.query_map([], timecode_from_row).map_err(err)?;
    rows.collect::<rusqlite::Result<Vec<_>>>().map_err(err)
}

#[tauri::command]
pub fn add_timecode(
    state: State<AppState>,
    client: String,
    project: String,
    task: String,
) -> CmdResult<Timecode> {
    let client = client.trim().to_string();
    let project = project.trim().to_string();
    let task = task.trim().to_string();
    if client.is_empty() || project.is_empty() || task.is_empty() {
        return Err("Client, project and task are all required.".into());
    }
    let conn = state.conn.lock().map_err(err)?;
    conn.execute(
        "INSERT INTO timecode (client, project, task, active) VALUES (?1, ?2, ?3, 1) \
         ON CONFLICT(client, project, task) DO UPDATE SET active = 1",
        params![client, project, task],
    )
    .map_err(err)?;
    let id: i64 = conn
        .query_row(
            "SELECT id FROM timecode WHERE client = ?1 AND project = ?2 AND task = ?3",
            params![client, project, task],
            |r| r.get(0),
        )
        .map_err(err)?;
    let label = Timecode::label(&client, &project, &task);
    Ok(Timecode {
        id,
        client,
        project,
        task,
        active: true,
        hidden: false,
        label,
    })
}

#[tauri::command]
pub fn set_timecode_active(state: State<AppState>, id: i64, active: bool) -> CmdResult<()> {
    let conn = state.conn.lock().map_err(err)?;
    conn.execute(
        "UPDATE timecode SET active = ?1 WHERE id = ?2",
        params![active as i64, id],
    )
    .map_err(err)?;
    Ok(())
}

/// User curation, independent of `active` (which sync owns): hide a code from
/// the Quick Capture picker without affecting whether Replicon still lists it.
#[tauri::command]
pub fn set_timecode_hidden(state: State<AppState>, id: i64, hidden: bool) -> CmdResult<()> {
    let conn = state.conn.lock().map_err(err)?;
    conn.execute(
        "UPDATE timecode SET hidden = ?1 WHERE id = ?2",
        params![hidden as i64, id],
    )
    .map_err(err)?;
    Ok(())
}

/// Distinct active timecodes ordered by most recently used.
#[tauri::command]
pub fn recent_timecodes(state: State<AppState>, limit: i64) -> CmdResult<Vec<Timecode>> {
    let conn = state.conn.lock().map_err(err)?;
    let mut stmt = conn
        .prepare(
            "SELECT t.id, t.client, t.project, t.task, t.active, t.hidden \
             FROM timecode t \
             JOIN ( \
                SELECT timecode_id, MAX(created_at) AS last_used \
                FROM time_entry \
                WHERE timecode_id IS NOT NULL \
                GROUP BY timecode_id \
             ) e ON e.timecode_id = t.id \
             WHERE t.active = 1 AND t.hidden = 0 \
             ORDER BY e.last_used DESC \
             LIMIT ?1",
        )
        .map_err(err)?;
    let rows = stmt.query_map([limit], timecode_from_row).map_err(err)?;
    rows.collect::<rusqlite::Result<Vec<_>>>().map_err(err)
}
