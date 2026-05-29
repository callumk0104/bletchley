// Dependency-free local backups of the SQLite file. No chrono: we compute the
// civil date from the system clock with Howard Hinnant's algorithm.
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// (year, month, day, hour, minute, second) in UTC.
pub fn now_parts() -> (i64, i64, i64, i64, i64, i64) {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let days = secs.div_euclid(86400);
    let rem = secs.rem_euclid(86400);
    let (h, mi, s) = (rem / 3600, (rem % 3600) / 60, rem % 60);

    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d, h, mi, s)
}

fn backups_dir(db_path: &Path) -> PathBuf {
    db_path
        .parent()
        .map(|p| p.join("backups"))
        .unwrap_or_else(|| PathBuf::from("backups"))
}

/// One backup per calendar day, taken at startup before the DB is opened.
/// Keeps the most recent `keep` files.
pub fn auto_backup(db_path: &Path, keep: usize) {
    if !db_path.exists() {
        return;
    }
    let dir = backups_dir(db_path);
    if std::fs::create_dir_all(&dir).is_err() {
        return;
    }
    let (y, m, d, _, _, _) = now_parts();
    let dest = dir.join(format!("timesheet-{:04}-{:02}-{:02}.db", y, m, d));
    if !dest.exists() {
        let _ = std::fs::copy(db_path, &dest);
    }
    prune(&dir, keep);
}

/// A manual, timestamped backup (allows several per day). Returns its path.
pub fn backup_now(db_path: &Path) -> std::io::Result<PathBuf> {
    let dir = backups_dir(db_path);
    std::fs::create_dir_all(&dir)?;
    let (y, m, d, h, mi, s) = now_parts();
    let dest = dir.join(format!(
        "timesheet-{:04}-{:02}-{:02}_{:02}-{:02}-{:02}.db",
        y, m, d, h, mi, s
    ));
    std::fs::copy(db_path, &dest)?;
    Ok(dest)
}

fn prune(dir: &Path, keep: usize) {
    let mut files: Vec<PathBuf> = match std::fs::read_dir(dir) {
        Ok(rd) => rd
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().map_or(false, |e| e == "db"))
            .collect(),
        Err(_) => return,
    };
    files.sort(); // names sort chronologically (YYYY-MM-DD...)
    if files.len() > keep {
        for p in &files[..files.len() - keep] {
            let _ = std::fs::remove_file(p);
        }
    }
}
