# Bletchley

A small Windows desktop app for capturing work as **structured data the moment it happens**, so filling in **Replicon** at week's end is transcription, not reconstruction.

Bletchley isn't a Replicon replacement — Replicon stays the system of record. It's the friendlier front you actually log into, then copy across in a couple of minutes.

Built with **Tauri 2** (Rust) + **Vue 3** (Vite) + **SQLite**, with signed in-app auto-updates from GitHub Releases.

## The idea

Replicon wants `Client → Project → Task` with hours per day, which is painful to enter live — so work ends up as rough free-text notes that get reconstructed from memory later. Bletchley closes that gap:

- **Quick Capture** makes structured entry as fast as a note: fuzzy-pick a timecode, type what you did, type a duration, hit enter (under 5 seconds).
- **Weekly Grid** mirrors Replicon's shape (timecodes × Mon–Fri, hours per cell) so transfer is near-mechanical, with a *Copy for Replicon* button that puts the table on your clipboard as TSV.

## Features

- **Fuzzy capture** over a flat list of `Client / Project / Task` timecodes (`chevqa` → `William Smith / Chevron Kits / QA`), with recents, sticky last-used, and a duration parser (`2` = 2h, `1.5`, `90m`, `1:30`, `1h30m`).
- **Optional start/stop timer** that fills the duration field.
- **Weekly grid** with rollup cells (sum per code/day, click to expand and edit the individual entries), row/day/grand totals, and under-target days flagged.
- **Capture-now, resolve-later**: log with a blank timecode; it lands in a "needs timecode" tray (also badged in the header) to assign later.
- **Timecode manager** with an `active` flag (retire codes without losing old entries).
- **Live totals** strip (today + week vs target) and a **status bar** (version, theme toggle, update pill).
- **Global hotkey + tray icon + mini quick-add window** — pop a compact capture box from any app.
- **Settings**: configurable daily target, theme (auto / light / dark), end-of-day reminder, hotkey.
- **Auto-backup** of the database on every launch (plus a manual "Back up now").
- **Frameless custom title bar** with window controls; right-click the logo for **About**.
- **Auto-update**: checks GitHub Releases on launch and self-installs signed updates.

## Install (users)

Download the latest `Bletchley_x.y.z_x64-setup.exe` from the [Releases](https://github.com/callumk0104/bletchley/releases) page and run it. It installs to your user profile (no admin), adds a Start-Menu shortcut, and updates itself in-app from then on.

## Develop

Prerequisites (install once):

1. **Rust** — https://www.rust-lang.org/tools/install (the `x86_64-pc-windows-msvc` toolchain).
2. **Microsoft C++ Build Tools** — the "Desktop development with C++" workload. Rust needs it to link.
3. **WebView2 runtime** — preinstalled on Windows 11; on Windows 10 grab the Evergreen runtime.
4. **Node.js 18+**.

```bash
npm install
npm run tauri dev    # launches with hot reload
```

Producing a signed installer locally needs the updater signing key in your environment — see **[RELEASING.md](RELEASING.md)** for keys, secrets, and the tag-driven release workflow.

## Where the data lives

A single SQLite file, `timesheet.db`, in your OS app-data dir (Windows: `%APPDATA%\com.callum.timesheet`). Daily backups sit alongside in `backups/`. The DB is seeded with starter timecodes on first run — edit them in **Manage codes**.

The bundle identifier (`com.callum.timesheet`) and the GitHub repo name are deliberately neutral and kept stable: the display name can change without breaking updates or moving your data.

## Architecture

```
src/                         Vue 3 frontend
  main.js                    entry; mounts App, loads global styles
  App.vue                    shell: header, tabs, status bar, context menu, modals
  api/                       invoke() wrappers around Rust commands, by domain
    timecodes.js  entries.js  settings.js  system.js  index.js (barrel)
  store/index.js             reactive shared store (codes, recents, settings, totals)
  composables/
    useEodReminder.js        end-of-day reminder scheduler + notifications
    useUpdater.js            GitHub-Releases update check / download / install
  lib/                       pure helpers: fuzzy.js, duration.js, dates.js
  assets/style.css           theme variables + global styles
  components/
    capture/    QuickCapture.vue, QuickAddMini.vue
    grid/       WeeklyGrid.vue, EntryEditor.vue
    timecodes/  TimecodeManager.vue, UnresolvedTray.vue
    settings/   Settings.vue
    layout/     TotalsBar.vue, StatusBar.vue, WindowControls.vue, About.vue
    common/     TimecodePicker.vue
src-tauri/                   Rust backend (Tauri 2)
  src/
    main.rs                  thin shim -> lib::run()
    lib.rs                   Tauri builder: plugins, system tray, global hotkey, windows
    db.rs                    schema, migrations, seed data
    models.rs                Timecode, TimeEntry
    backup.rs                dependency-free startup + manual DB backups
    commands/                commands grouped by domain, re-exported from mod.rs
      mod.rs (shared helpers) timecodes.rs  entries.rs  settings.rs  system.rs
  tauri.conf.json            app, NSIS bundle, and updater config
  capabilities/default.json  window + plugin permissions
.github/workflows/release.yml  tag-triggered build → sign → draft release
```

The frontend talks to the backend only through `invoke()` (the `api/` wrappers), so the data layer stays isolated — which keeps the door open for the parked stretch goal of Replicon browser-autofill without touching the UI.

### Data model

```
timecode(id, client, project, task, active)
time_entry(id, timecode_id?, date, duration_minutes, description, created_at)
```

`timecode_id` is nullable to support capture-now-resolve-later. Retiring a code (`active = 0`) drops it from the typeahead but leaves old entries intact (`ON DELETE SET NULL`). The grid is a pivot of:

```sql
SELECT timecode_id, date, SUM(duration_minutes)
FROM time_entry WHERE date BETWEEN ? AND ? GROUP BY timecode_id, date
```

## Notes

- The app icon is a generated placeholder. To swap it, drop a 1024×1024 PNG in and run `npm run tauri icon path/to/icon.png`.
- Releasing, code signing, and the auto-update setup are documented in **[RELEASING.md](RELEASING.md)**.
