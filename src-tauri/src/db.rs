use rusqlite::Connection;
use std::path::Path;

/// Open (or create) the database, apply the schema, and seed a starter set
/// of timecodes on first run.
pub fn init(path: &Path) -> rusqlite::Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch(
        r#"
        PRAGMA journal_mode = WAL;
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS timecode (
            id      INTEGER PRIMARY KEY AUTOINCREMENT,
            client  TEXT NOT NULL,
            project TEXT NOT NULL,
            task    TEXT NOT NULL,
            active  INTEGER NOT NULL DEFAULT 1,
            -- user curation, never touched by sync: hide from the picker
            hidden  INTEGER NOT NULL DEFAULT 0,
            -- 'local' = hand-added, 'replicon' = synced from Replicon
            source  TEXT NOT NULL DEFAULT 'local',
            billing TEXT,
            replicon_client_uri  TEXT,
            replicon_project_uri TEXT,
            replicon_task_uri    TEXT,
            UNIQUE(client, project, task)
        );

        CREATE TABLE IF NOT EXISTS time_entry (
            id               INTEGER PRIMARY KEY AUTOINCREMENT,
            timecode_id      INTEGER REFERENCES timecode(id) ON DELETE SET NULL,
            date             TEXT NOT NULL,
            duration_minutes INTEGER NOT NULL,
            description      TEXT NOT NULL DEFAULT '',
            created_at       TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE INDEX IF NOT EXISTS idx_entry_date     ON time_entry(date);
        CREATE INDEX IF NOT EXISTS idx_entry_timecode ON time_entry(timecode_id);

        CREATE TABLE IF NOT EXISTS setting (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
        "#,
    )?;

    migrate(&conn)?;
    purge_seed_timecodes(&conn)?;
    seed_default_settings(&conn)?;
    Ok(conn)
}

/// Idempotent column additions for databases created before the Replicon
/// fields existed. SQLite's ALTER TABLE ADD COLUMN is a no-op-safe forward
/// migration as long as we only add columns that aren't already present.
fn migrate(conn: &Connection) -> rusqlite::Result<()> {
    let additions: &[(&str, &str)] = &[
        ("hidden", "hidden INTEGER NOT NULL DEFAULT 0"),
        ("source", "source TEXT NOT NULL DEFAULT 'local'"),
        ("billing", "billing TEXT"),
        ("replicon_client_uri", "replicon_client_uri TEXT"),
        ("replicon_project_uri", "replicon_project_uri TEXT"),
        ("replicon_task_uri", "replicon_task_uri TEXT"),
    ];
    for (name, decl) in additions {
        if !column_exists(conn, "timecode", name)? {
            conn.execute(&format!("ALTER TABLE timecode ADD COLUMN {decl}"), [])?;
        }
    }
    // Natural key for syncing: a Replicon timecode is a (project, task) pair.
    conn.execute_batch(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_timecode_replicon          ON timecode(replicon_project_uri, replicon_task_uri)          WHERE replicon_project_uri IS NOT NULL;",
    )?;
    Ok(())
}

fn column_exists(conn: &Connection, table: &str, column: &str) -> rusqlite::Result<bool> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
    let names = stmt.query_map([], |r| r.get::<_, String>(1))?;
    for n in names {
        if n? == column {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Insert default settings without clobbering anything the user has changed.
fn seed_default_settings(conn: &Connection) -> rusqlite::Result<()> {
    let defaults: &[(&str, &str)] = &[
        ("daily_target_minutes", "450"), // 7.5h
        ("theme", "auto"),               // auto | light | dark
        ("eod_reminder_enabled", "0"),
        ("eod_reminder_time", "17:00"),
        ("hotkey", "CmdOrCtrl+Shift+Space"),
    ];
    let mut stmt =
        conn.prepare("INSERT OR IGNORE INTO setting (key, value) VALUES (?1, ?2)")?;
    for (k, v) in defaults {
        stmt.execute([k, v])?;
    }
    Ok(())
}

/// One-time removal of the original hand-seeded placeholder timecodes, now that
/// Replicon sync provides the real ones. Guarded by a setting so it runs once;
/// any time entries attached to a removed code fall back to the unresolved tray
/// (the timecode FK is ON DELETE SET NULL).
fn purge_seed_timecodes(conn: &Connection) -> rusqlite::Result<()> {
    let already = conn
        .query_row("SELECT 1 FROM setting WHERE key = 'seed_purged'", [], |_| Ok(()))
        .is_ok();
    if already {
        return Ok(());
    }
    conn.execute("DELETE FROM timecode WHERE source = 'local'", [])?;
    conn.execute(
        "INSERT OR REPLACE INTO setting (key, value) VALUES ('seed_purged', '1')",
        [],
    )?;
    Ok(())
}
