//! Tauri commands, grouped by domain. Shared helpers live here; each submodule
//! holds the commands for one area and is re-exported so `commands::foo`
//! continues to resolve in `generate_handler!`.
use crate::models::{TimeEntry, Timecode};
use rusqlite::{Connection, Row};

/// Commands return Result<T, String> so any rusqlite error surfaces as a
/// readable message on the JS side instead of panicking the backend.
pub type CmdResult<T> = Result<T, String>;

pub fn err<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}

pub fn timecode_from_row(r: &Row) -> rusqlite::Result<Timecode> {
    let client: String = r.get(1)?;
    let project: String = r.get(2)?;
    let task: String = r.get(3)?;
    let active: i64 = r.get(4)?;
    let label = Timecode::label(&client, &project, &task);
    Ok(Timecode {
        id: r.get(0)?,
        client,
        project,
        task,
        active: active != 0,
        label,
    })
}

pub fn entry_from_row(r: &Row) -> rusqlite::Result<TimeEntry> {
    Ok(TimeEntry {
        id: r.get(0)?,
        timecode_id: r.get(1)?,
        date: r.get(2)?,
        duration_minutes: r.get(3)?,
        description: r.get(4)?,
        created_at: r.get(5)?,
    })
}

pub fn fetch_entry(conn: &Connection, id: i64) -> CmdResult<TimeEntry> {
    conn.query_row(
        "SELECT id, timecode_id, date, duration_minutes, description, created_at \
         FROM time_entry WHERE id = ?1",
        [id],
        entry_from_row,
    )
    .map_err(err)
}

mod entries;
mod replicon;
mod settings;
mod system;
mod timecodes;

pub use entries::*;
pub use replicon::*;
pub use settings::*;
pub use system::*;
pub use timecodes::*;
