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

    seed_if_empty(&conn)?;
    seed_default_settings(&conn)?;
    Ok(conn)
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

/// Hand-maintained seed data. Replicon has no export, so the timecode tree
/// lives locally; this gives a realistic starting set to type against.
/// ~10 clients, a couple of projects each, tasks fanning out per project.
fn seed_if_empty(conn: &Connection) -> rusqlite::Result<()> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM timecode", [], |r| r.get(0))?;
    if count > 0 {
        return Ok(());
    }

    // (client, project, task)
    let seed: &[(&str, &str, &str)] = &[
        ("William Smith", "Chevron Kits", "QA"),
        ("William Smith", "Chevron Kits", "Frontend"),
        ("William Smith", "Chevron Kits", "Backend"),
        ("William Smith", "Chevron Kits", "Standups"),
        ("William Smith", "Atlas Migration", "Data modelling"),
        ("William Smith", "Atlas Migration", "ETL"),
        ("Northwind Trading", "Ledger Revamp", "Discovery"),
        ("Northwind Trading", "Ledger Revamp", "Implementation"),
        ("Northwind Trading", "Ledger Revamp", "Code review"),
        ("Northwind Trading", "Mobile App", "iOS"),
        ("Northwind Trading", "Mobile App", "Android"),
        ("Acme Logistics", "Route Optimiser", "Algorithm"),
        ("Acme Logistics", "Route Optimiser", "Testing"),
        ("Acme Logistics", "Warehouse Portal", "Frontend"),
        ("Globex", "Billing Platform", "API design"),
        ("Globex", "Billing Platform", "Integration"),
        ("Globex", "Billing Platform", "Support"),
        ("Initech", "TPS Reporting", "Reports"),
        ("Initech", "TPS Reporting", "Maintenance"),
        ("Umbrella Health", "Patient Records", "Compliance"),
        ("Umbrella Health", "Patient Records", "Backend"),
        ("Hooli", "Search Revamp", "Indexing"),
        ("Hooli", "Search Revamp", "Relevance tuning"),
        ("Wayne Enterprises", "Security Audit", "Pen testing"),
        ("Wayne Enterprises", "Security Audit", "Remediation"),
        ("Stark Industries", "Telemetry", "Pipeline"),
        ("Stark Industries", "Telemetry", "Dashboards"),
        ("Soylent Corp", "Internal", "Admin"),
        ("Soylent Corp", "Internal", "Training"),
        ("Soylent Corp", "Internal", "Leave"),
    ];

    let mut stmt =
        conn.prepare("INSERT INTO timecode (client, project, task, active) VALUES (?1, ?2, ?3, 1)")?;
    for (client, project, task) in seed {
        stmt.execute([client, project, task])?;
    }
    Ok(())
}
