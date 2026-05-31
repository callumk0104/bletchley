mod backup;
mod commands;
mod db;
mod models;

use std::sync::Mutex;

use rusqlite::Connection;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

/// Shared application state: a single SQLite connection behind a mutex.
pub struct AppState {
    pub conn: Mutex<Connection>,
}

fn show_window(app: &tauri::AppHandle, label: &str) {
    if let Some(w) = app.get_webview_window(label) {
        let _ = w.show();
        let _ = w.set_focus();
    }
}

/// Show the quick-add popup if hidden, hide it if already visible.
fn toggle_quick(app: &tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("quick") {
        if w.is_visible().unwrap_or(false) {
            let _ = w.hide();
        } else {
            let _ = w.show();
            let _ = w.set_focus();
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Must be the first plugin: if a second copy of Bletchley is launched,
        // focus the running window instead of starting a clashing process
        // (which is what caused the global-hotkey "already registered" error).
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.show();
                let _ = w.unminimize();
                let _ = w.set_focus();
            }
        }))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        toggle_quick(app);
                    }
                })
                .build(),
        )
        .setup(|app| {
            let dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");
            std::fs::create_dir_all(&dir).ok();
            let db_path = dir.join("timesheet.db");

            backup::auto_backup(&db_path, 14);
            let conn = db::init(&db_path).expect("failed to initialise database");

            // Read the configured hotkey before the connection moves into state.
            let hotkey: String = conn
                .query_row("SELECT value FROM setting WHERE key = 'hotkey'", [], |r| {
                    r.get(0)
                })
                .unwrap_or_else(|_| "CmdOrCtrl+Shift+Space".to_string());

            app.manage(AppState {
                conn: Mutex::new(conn),
            });

            // --- System tray ---
            let open_i = MenuItem::with_id(app, "open", "Open Bletchley", true, None::<&str>)?;
            let quick_i = MenuItem::with_id(app, "quick", "Quick add", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open_i, &quick_i, &quit_i])?;

            TrayIconBuilder::with_id("main-tray")
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("Bletchley")
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "open" => show_window(app, "main"),
                    "quick" => toggle_quick(app),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        show_window(tray.app_handle(), "main");
                    }
                })
                .build(app)?;

            // --- Global quick-add hotkey (non-fatal if it can't be set) ---
            match hotkey.parse::<Shortcut>() {
                Ok(sc) => {
                    if let Err(e) = app.global_shortcut().register(sc) {
                        eprintln!("Failed to register hotkey '{hotkey}': {e}");
                    }
                }
                Err(e) => eprintln!("Invalid hotkey '{hotkey}': {e}"),
            }

            // --- Close-to-tray: hide the window instead of quitting (unless the
            // user turned it off), so the hotkey and reminders keep working. ---
            let handle = app.handle().clone();
            if let Some(main) = app.get_webview_window("main") {
                main.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        let keep = (|| {
                            let st = handle.try_state::<AppState>()?;
                            let conn = st.conn.lock().ok()?;
                            let v: String = conn
                                .query_row(
                                    "SELECT value FROM setting WHERE key = 'close_to_tray'",
                                    [],
                                    |r| r.get(0),
                                )
                                .ok()?;
                            Some(v != "0")
                        })()
                        .unwrap_or(true);
                        if keep {
                            api.prevent_close();
                            if let Some(w) = handle.get_webview_window("main") {
                                let _ = w.hide();
                            }
                        } else {
                            handle.exit(0);
                        }
                    }
                });
            }

            // Launched at login with `--minimized`: start hidden in the tray.
            if std::env::args().any(|a| a == "--minimized") {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.hide();
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_timecodes,
            commands::add_timecode,
            commands::set_timecode_active,
            commands::set_timecode_hidden,
            commands::recent_timecodes,
            commands::add_entry,
            commands::update_entry,
            commands::delete_entry,
            commands::entries_for_week,
            commands::unresolved_entries,
            commands::search_entries,
            commands::totals,
            commands::get_settings,
            commands::set_setting,
            commands::backup_now,
            commands::backups_path,
            commands::update_tray,
            commands::autostart_is_enabled,
            commands::autostart_set,
            commands::replicon_set_password,
            commands::replicon_has_password,
            commands::replicon_test_connection,
            commands::replicon_sync_timecodes,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Bletchley");
}
