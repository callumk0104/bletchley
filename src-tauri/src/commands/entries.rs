use super::{entry_from_row, err, fetch_entry, CmdResult};
use crate::models::TimeEntry;
use crate::AppState;
use rusqlite::params;
use serde::Serialize;
use tauri::State;

#[tauri::command]
pub fn add_entry(
    state: State<AppState>,
    timecode_id: Option<i64>,
    date: String,
    duration_minutes: i64,
    description: String,
) -> CmdResult<TimeEntry> {
    if duration_minutes <= 0 {
        return Err("Duration must be greater than zero.".into());
    }
    let conn = state.conn.lock().map_err(err)?;
    conn.execute(
        "INSERT INTO time_entry (timecode_id, date, duration_minutes, description) \
         VALUES (?1, ?2, ?3, ?4)",
        params![timecode_id, date, duration_minutes, description],
    )
    .map_err(err)?;
    let id = conn.last_insert_rowid();
    fetch_entry(&conn, id)
}

#[tauri::command]
pub fn update_entry(
    state: State<AppState>,
    id: i64,
    timecode_id: Option<i64>,
    date: String,
    duration_minutes: i64,
    description: String,
) -> CmdResult<TimeEntry> {
    if duration_minutes <= 0 {
        return Err("Duration must be greater than zero.".into());
    }
    let conn = state.conn.lock().map_err(err)?;
    conn.execute(
        "UPDATE time_entry \
         SET timecode_id = ?2, date = ?3, duration_minutes = ?4, description = ?5 \
         WHERE id = ?1",
        params![id, timecode_id, date, duration_minutes, description],
    )
    .map_err(err)?;
    fetch_entry(&conn, id)
}

#[tauri::command]
pub fn delete_entry(state: State<AppState>, id: i64) -> CmdResult<()> {
    let conn = state.conn.lock().map_err(err)?;
    conn.execute("DELETE FROM time_entry WHERE id = ?1", [id])
        .map_err(err)?;
    Ok(())
}

/// All entries in [week_start, week_end] inclusive (ISO dates). The frontend
/// pivots these into the rows x days grid and groups them per cell.
#[tauri::command]
pub fn entries_for_week(
    state: State<AppState>,
    week_start: String,
    week_end: String,
) -> CmdResult<Vec<TimeEntry>> {
    let conn = state.conn.lock().map_err(err)?;
    let mut stmt = conn
        .prepare(
            "SELECT id, timecode_id, date, duration_minutes, description, created_at \
             FROM time_entry \
             WHERE date BETWEEN ?1 AND ?2 \
             ORDER BY date, created_at",
        )
        .map_err(err)?;
    let rows = stmt
        .query_map(params![week_start, week_end], entry_from_row)
        .map_err(err)?;
    rows.collect::<rusqlite::Result<Vec<_>>>().map_err(err)
}

/// Entries with no timecode yet — the "needs timecode" tray.
#[tauri::command]
pub fn unresolved_entries(state: State<AppState>) -> CmdResult<Vec<TimeEntry>> {
    let conn = state.conn.lock().map_err(err)?;
    let mut stmt = conn
        .prepare(
            "SELECT id, timecode_id, date, duration_minutes, description, created_at \
             FROM time_entry \
             WHERE timecode_id IS NULL \
             ORDER BY date DESC, created_at DESC",
        )
        .map_err(err)?;
    let rows = stmt.query_map([], entry_from_row).map_err(err)?;
    rows.collect::<rusqlite::Result<Vec<_>>>().map_err(err)
}

/// Logged minutes for a single day and for the whole week — powers the
/// always-visible totals strip.
#[derive(Serialize)]
pub struct Totals {
    pub today_minutes: i64,
    pub week_minutes: i64,
}

#[tauri::command]
pub fn totals(
    state: State<AppState>,
    today: String,
    week_start: String,
    week_end: String,
) -> CmdResult<Totals> {
    let conn = state.conn.lock().map_err(err)?;
    let today_minutes: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(duration_minutes), 0) FROM time_entry WHERE date = ?1",
            [&today],
            |r| r.get(0),
        )
        .map_err(err)?;
    let week_minutes: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(duration_minutes), 0) FROM time_entry WHERE date BETWEEN ?1 AND ?2",
            params![week_start, week_end],
            |r| r.get(0),
        )
        .map_err(err)?;
    Ok(Totals {
        today_minutes,
        week_minutes,
    })
}
