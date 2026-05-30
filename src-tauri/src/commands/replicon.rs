//! Replicon connection: store the password in the OS keychain (the rest of the
//! connection lives in the settings table), and a Test-connection probe that
//! confirms Basic auth works against the Gen3 Web Services API.
use super::{err, CmdResult};
use crate::AppState;
use serde::Serialize;
use tauri::State;

const KEY_SERVICE: &str = "Bletchley";
const KEY_USER: &str = "replicon-password";

fn read_setting(conn: &rusqlite::Connection, key: &str) -> Option<String> {
    conn.query_row("SELECT value FROM setting WHERE key = ?1", [key], |r| r.get(0))
        .ok()
}

/// Store (or clear) the Replicon password in the OS keychain — never the DB.
#[tauri::command]
pub fn replicon_set_password(password: String) -> CmdResult<()> {
    let entry = keyring::Entry::new(KEY_SERVICE, KEY_USER).map_err(err)?;
    if password.is_empty() {
        let _ = entry.delete_credential(); // ignore "not found"
        Ok(())
    } else {
        entry.set_password(&password).map_err(err)
    }
}

/// Whether a password is currently stored (so the UI can show its state).
#[tauri::command]
pub fn replicon_has_password() -> CmdResult<bool> {
    let entry = keyring::Entry::new(KEY_SERVICE, KEY_USER).map_err(err)?;
    Ok(entry.get_password().is_ok())
}

#[derive(Serialize)]
pub struct TestResult {
    pub ok: bool,
    pub status: u16,
    pub message: String,
    pub body: String,
}

/// Probe the Gen3 API with Basic auth. Treats 401/403 as bad credentials and
/// any other reachable response as "auth OK" — the body snippet lets us refine
/// the exact endpoints from a real response.
#[tauri::command]
pub async fn replicon_test_connection(state: State<'_, AppState>) -> CmdResult<TestResult> {
    // Pull config synchronously, then drop the lock before any await.
    let (base_url, company, username) = {
        let conn = state.conn.lock().map_err(err)?;
        (
            read_setting(&conn, "replicon_base_url").unwrap_or_default(),
            read_setting(&conn, "replicon_company").unwrap_or_default(),
            read_setting(&conn, "replicon_username").unwrap_or_default(),
        )
    };
    if base_url.is_empty() || company.is_empty() || username.is_empty() {
        return Ok(TestResult {
            ok: false,
            status: 0,
            message: "Fill in base URL, company and username first.".into(),
            body: String::new(),
        });
    }
    let password = keyring::Entry::new(KEY_SERVICE, KEY_USER)
        .map_err(err)?
        .get_password()
        .map_err(|_| "No password stored — set it first.".to_string())?;

    let url = format!(
        "{}/services/TimesheetService1.svc/GetAllowableTimesheetDateRangeForUser",
        base_url.trim_end_matches('/')
    );
    let resp = reqwest::Client::new()
        .post(&url)
        .basic_auth(format!("{company}\\{username}"), Some(password))
        .header("Content-Type", "application/json")
        .body("{}")
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;

    let status = resp.status().as_u16();
    let text = resp.text().await.unwrap_or_default();
    let body: String = text.chars().take(300).collect();
    let (ok, message) = match status {
        401 | 403 => (
            false,
            "Authentication failed — check company, username and password.".to_string(),
        ),
        200..=299 => (true, "Connected to Replicon.".to_string()),
        s => (
            true,
            format!("Reached Replicon (HTTP {s}); auth appears OK but the response was unexpected."),
        ),
    };
    Ok(TestResult { ok, status, message, body })
}
