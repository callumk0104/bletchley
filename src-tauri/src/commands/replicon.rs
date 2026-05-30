//! Replicon connection: store the password in the OS keychain (the rest of the
//! connection lives in the settings table), a Test-connection probe, and a
//! read-only timecode sync that walks the Gen3 time-allocation tree
//! (Clients -> Projects -> Tasks) and upserts it into the local timecode table.
use super::{err, CmdResult};
use crate::AppState;
use rusqlite::OptionalExtension;
use serde::Serialize;
use serde_json::{json, Value};
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

// ---------------------------------------------------------------------------
// Timecode sync (read-only)
// ---------------------------------------------------------------------------

/// The credentials needed for one Gen3 request, gathered once per sync.
struct RepliconCfg {
    base_url: String,
    company: String,
    username: String,
    password: String,
}

/// POST a Gen3 method and unwrap the `{ "d": ... }` / `{ "error": ... }`
/// envelope. Returns the inner `d` payload, or a readable error.
async fn gen3(cfg: &RepliconCfg, svc: &str, method: &str, body: Value) -> CmdResult<Value> {
    let url = format!("{}/services/{}/{}", cfg.base_url, svc, method);
    let resp = reqwest::Client::new()
        .post(&url)
        .basic_auth(
            format!("{}\\{}", cfg.company, cfg.username),
            Some(&cfg.password),
        )
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Network error: {e}"))?;
    let status = resp.status();
    let v: Value = resp
        .json()
        .await
        .map_err(|e| format!("{method}: unreadable response ({e})"))?;
    if let Some(error) = v.get("error") {
        let msg = error
            .get("details")
            .and_then(|d| d.get("displayText"))
            .and_then(Value::as_str)
            .or_else(|| error.get("reason").and_then(Value::as_str))
            .unwrap_or("Replicon error");
        return Err(format!("{method}: {msg} (HTTP {})", status.as_u16()));
    }
    Ok(v.get("d").cloned().unwrap_or(Value::Null))
}

/// Page through a `GetPageOf…` method until a short page signals the end.
async fn page_all(
    cfg: &RepliconCfg,
    svc: &str,
    method: &str,
    base_body: Value,
) -> CmdResult<Vec<Value>> {
    const PAGE_SIZE: i64 = 200;
    let mut out = Vec::new();
    let mut page = 1i64;
    loop {
        let mut body = base_body.clone();
        if let Some(obj) = body.as_object_mut() {
            obj.insert("page".into(), json!(page));
            obj.insert("pageSize".into(), json!(PAGE_SIZE));
        }
        let arr = gen3(cfg, svc, method, body)
            .await?
            .as_array()
            .cloned()
            .unwrap_or_default();
        let n = arr.len() as i64;
        out.extend(arr);
        if n < PAGE_SIZE {
            break;
        }
        page += 1;
        if page > 500 {
            break; // safety bound against a misbehaving endpoint
        }
    }
    Ok(out)
}

/// Best human label for a Replicon ref object: name, then displayText, then uri.
fn best_name(v: &Value) -> String {
    for k in ["name", "displayText"] {
        if let Some(s) = v.get(k).and_then(Value::as_str) {
            if !s.is_empty() {
                return s.to_string();
            }
        }
    }
    // Time-allocation task refs wrap the real ref under `.task`.
    if let Some(inner) = v.get("task") {
        for k in ["name", "displayText"] {
            if let Some(s) = inner.get(k).and_then(Value::as_str) {
                if !s.is_empty() {
                    return s.to_string();
                }
            }
        }
    }
    v.get("uri").and_then(Value::as_str).unwrap_or("?").to_string()
}

/// One bookable leaf: a (client, project, task) triple with its Replicon URNs.
struct Leaf {
    client: String,
    project: String,
    task: String,
    client_uri: String,
    project_uri: String,
    task_uri: Option<String>,
}

#[derive(Serialize)]
pub struct SyncResult {
    pub ok: bool,
    pub added: i64,
    pub updated: i64,
    pub total: i64,
    pub message: String,
}

/// Read-only sync: enumerate everything the signed-in user can book time to and
/// upsert it as `source = 'replicon'` timecodes. Replicon stays the source of
/// truth; local edits to synced rows are overwritten on the next sync.
#[tauri::command]
pub async fn replicon_sync_timecodes(state: State<'_, AppState>) -> CmdResult<SyncResult> {
    let (base_url, company, username) = {
        let conn = state.conn.lock().map_err(err)?;
        (
            read_setting(&conn, "replicon_base_url").unwrap_or_default(),
            read_setting(&conn, "replicon_company").unwrap_or_default(),
            read_setting(&conn, "replicon_username").unwrap_or_default(),
        )
    };
    if base_url.is_empty() || company.is_empty() || username.is_empty() {
        return Ok(SyncResult {
            ok: false,
            added: 0,
            updated: 0,
            total: 0,
            message: "Fill in base URL, company and username first.".into(),
        });
    }
    let password = keyring::Entry::new(KEY_SERVICE, KEY_USER)
        .map_err(err)?
        .get_password()
        .map_err(|_| "No password stored — set it first.".to_string())?;
    let cfg = RepliconCfg {
        base_url: base_url.trim_end_matches('/').to_string(),
        company,
        username,
        password,
    };

    // 1. Resolve the signed-in user's URN. Basic auth logs in with the login
    //    name, so GetUser2 by loginName returns the authenticated user. The
    //    UserTargetParameter is wrapped under the `user` parameter name.
    let user = gen3(
        &cfg,
        "UserService1.svc",
        "GetUser2",
        json!({ "user": { "loginName": cfg.username } }),
    )
    .await?;
    let user_uri = user
        .get("uri")
        .and_then(Value::as_str)
        .ok_or_else(|| "Couldn't resolve your Replicon user — check the username.".to_string())?
        .to_string();

    // 2. The current timesheet scopes which projects/tasks are bookable.
    let (y, m, d, _, _, _) = crate::backup::now_parts();
    let ts = gen3(
        &cfg,
        "TimesheetService1.svc",
        "GetTimesheetForDate",
        json!({ "userUri": user_uri, "date": { "day": d, "month": m, "year": y } }),
    )
    .await?;
    let ts_uri = ts
        .get("uri")
        .and_then(Value::as_str)
        .ok_or_else(|| "No timesheet found for today.".to_string())?
        .to_string();

    // 3. Walk Clients -> Projects -> Tasks.
    let mut leaves: Vec<Leaf> = Vec::new();
    let clients = page_all(
        &cfg,
        "TimesheetService1.svc",
        "GetPageOfClientsAvailableForTimeAllocationFilteredByTextSearch",
        json!({ "timesheetUri": ts_uri, "textSearch": "" }),
    )
    .await?;
    for c in &clients {
        let c_uri = match c.get("uri").and_then(Value::as_str) {
            Some(u) => u.to_string(),
            None => continue,
        };
        let c_name = best_name(c);
        let projects = page_all(
            &cfg,
            "TimesheetService1.svc",
            "GetPageOfProjectsAvailableForTimeAllocationFilteredByClientAndTextSearch",
            json!({ "timesheetUri": ts_uri, "clientUri": c_uri, "textSearch": "" }),
        )
        .await?;
        for pr in &projects {
            let proj = match pr.get("project") {
                Some(p) => p,
                None => continue,
            };
            let p_uri = match proj.get("uri").and_then(Value::as_str) {
                Some(u) => u.to_string(),
                None => continue,
            };
            let p_name = best_name(proj);
            let has_tasks = pr
                .get("hasTasksAvailableForTimeAllocation")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            if has_tasks {
                let tasks = page_all(
                    &cfg,
                    "TimesheetService1.svc",
                    "GetPageOfTasksAvailableForTimeAllocationFilteredByProjectAndTextSearch",
                    json!({ "timesheetUri": ts_uri, "projectUri": p_uri, "textSearch": "" }),
                )
                .await?;
                for tk in &tasks {
                    // Tasks come back double-nested: row.task = { parentTask, task },
                    // where the bookable leaf ref ({ code, displayText, name, uri }) is
                    // the inner `task`. parentTask is null for top-level tasks.
                    let outer = match tk.get("task") {
                        Some(o) => o,
                        None => continue,
                    };
                    let task = match outer.get("task") {
                        Some(t) => t,
                        None => continue,
                    };
                    let t_uri = task.get("uri").and_then(Value::as_str).map(str::to_string);
                    let leaf_name = best_name(task);
                    // Qualify with the parent task (when present) so sibling tasks that
                    // share a leaf name stay distinct under UNIQUE(client, project, task).
                    let task_label = match outer.get("parentTask") {
                        Some(p) if p.is_object() => {
                            let pn = best_name(p);
                            if pn == "?" {
                                leaf_name
                            } else {
                                format!("{pn} / {leaf_name}")
                            }
                        }
                        _ => leaf_name,
                    };
                    leaves.push(Leaf {
                        client: c_name.clone(),
                        project: p_name.clone(),
                        task: task_label,
                        client_uri: c_uri.clone(),
                        project_uri: p_uri.clone(),
                        task_uri: t_uri,
                    });
                }
            } else {
                // Project bookable directly, with no task breakdown.
                leaves.push(Leaf {
                    client: c_name.clone(),
                    project: p_name.clone(),
                    task: "(project)".into(),
                    client_uri: c_uri.clone(),
                    project_uri: p_uri.clone(),
                    task_uri: None,
                });
            }
        }
    }

    // 4. Upsert. Match first on the Replicon (project, task) pair, then fall
    //    back to the (client, project, task) name triple so an existing local
    //    code is upgraded in place rather than duplicated.
    let mut added = 0i64;
    let mut updated = 0i64;
    // Assigned exactly once inside the transaction block below.
    let retired: i64;
    {
        let mut conn = state.conn.lock().map_err(err)?;
        let tx = conn.transaction().map_err(err)?;
        for lf in &leaves {
            let by_pair: Option<i64> = tx
                .query_row(
                    "SELECT id FROM timecode WHERE replicon_project_uri = ?1 \
                     AND ((replicon_task_uri IS NULL AND ?2 IS NULL) OR replicon_task_uri = ?2)",
                    rusqlite::params![lf.project_uri, lf.task_uri],
                    |r| r.get(0),
                )
                .optional()
                .map_err(err)?;
            let target = match by_pair {
                Some(id) => Some(id),
                None => tx
                    .query_row(
                        "SELECT id FROM timecode WHERE client = ?1 AND project = ?2 AND task = ?3",
                        rusqlite::params![lf.client, lf.project, lf.task],
                        |r| r.get(0),
                    )
                    .optional()
                    .map_err(err)?,
            };
            match target {
                Some(id) => {
                    tx.execute(
                        "UPDATE timecode SET client = ?1, project = ?2, task = ?3, \
                         source = 'replicon', active = 1, replicon_client_uri = ?4, \
                         replicon_project_uri = ?5, replicon_task_uri = ?6 WHERE id = ?7",
                        rusqlite::params![
                            lf.client,
                            lf.project,
                            lf.task,
                            lf.client_uri,
                            lf.project_uri,
                            lf.task_uri,
                            id
                        ],
                    )
                    .map_err(err)?;
                    updated += 1;
                }
                None => {
                    tx.execute(
                        "INSERT INTO timecode \
                         (client, project, task, active, source, replicon_client_uri, \
                          replicon_project_uri, replicon_task_uri) \
                         VALUES (?1, ?2, ?3, 1, 'replicon', ?4, ?5, ?6)",
                        rusqlite::params![
                            lf.client,
                            lf.project,
                            lf.task,
                            lf.client_uri,
                            lf.project_uri,
                            lf.task_uri
                        ],
                    )
                    .map_err(err)?;
                    added += 1;
                }
            }
        }

        // Reconcile: retire any previously-synced Replicon codes no longer in
        // this pull (renamed/removed tasks, and artifacts of earlier runs).
        // Retiring keeps history while dropping them from the picker.
        let seen: std::collections::HashSet<(String, Option<String>)> = leaves
            .iter()
            .map(|lf| (lf.project_uri.clone(), lf.task_uri.clone()))
            .collect();
        let stale: Vec<i64> = {
            let mut stmt = tx
                .prepare(
                    "SELECT id, replicon_project_uri, replicon_task_uri \
                     FROM timecode WHERE source = 'replicon' AND active = 1",
                )
                .map_err(err)?;
            let rows = stmt
                .query_map([], |r| {
                    Ok((
                        r.get::<_, i64>(0)?,
                        r.get::<_, Option<String>>(1)?,
                        r.get::<_, Option<String>>(2)?,
                    ))
                })
                .map_err(err)?;
            let mut ids = Vec::new();
            for row in rows {
                let (id, puri, turi) = row.map_err(err)?;
                if !seen.contains(&(puri.unwrap_or_default(), turi)) {
                    ids.push(id);
                }
            }
            ids
        };
        for id in &stale {
            tx.execute("UPDATE timecode SET active = 0 WHERE id = ?1", [id])
                .map_err(err)?;
        }
        retired = stale.len() as i64;
        tx.commit().map_err(err)?;
    }

    let total = leaves.len() as i64;
    Ok(SyncResult {
        ok: true,
        added,
        updated,
        total,
        message: format!(
            "Synced {total} timecodes ({added} new, {updated} updated, {retired} retired)."
        ),
    })
}
