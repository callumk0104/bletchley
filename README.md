# Bletchley

A tiny desktop app for capturing work as **structured data the moment it happens**, so transferring into Replicon at week's end is transcription, not reconstruction.

Built with **Tauri 2** (Rust backend) + **Vue 3** (Vite) + **SQLite** (single local file).

## The idea

Replicon wants `Client → Project → Task` with hours per day, but that's painful to enter live, so work ends up as rough free-text notes that have to be reconstructed later. This app closes that gap:

- **Quick Capture** makes structured entry as fast as a note: fuzzy-pick a timecode, type what you did, type a duration, hit enter.
- **Weekly Grid** mirrors Replicon's shape (timecodes × Mon–Fri, hours per cell) so transfer is near-mechanical — with a *Copy for Replicon* button that puts the table on your clipboard as TSV.

## Decisions baked in

| Question | Decision |
|---|---|
| Duration entry | **Manual** (`2h`, `90m`, `1h30m`, `1:30`, `1.5h`) with an **optional start/stop timer** that fills the field. |
| Week | **Mon–Fri**, week starts Monday. |
| Unresolved entries | Captured with a blank timecode go to a **"needs timecode" tray** *and* show a **count badge** in the header and grid toolbar. |
| Replicon autofill | Not built (v1 is copy-to-clipboard). The data layer is isolated behind Rust commands, so browser automation can be bolted on later without touching the UI. |

## Key design point

A grid cell is a **rollup of entries, not a single number.** Each entry keeps its own description ("fixed checkout bug"); the cell shows the summed hours and **expands on click** to reveal — and edit — the underlying entries. Days that don't reach 7.5h are flagged (under / empty), so gaps are obvious.

## Running it (Windows)

Prerequisites — install once:

1. **Rust** — https://www.rust-lang.org/tools/install (the `x86_64-pc-windows-msvc` toolchain).
2. **Microsoft C++ Build Tools** — "Desktop development with C++" workload (https://visualstudio.microsoft.com/visual-studio-build-tools/). Needed by Rust to link.
3. **WebView2 runtime** — preinstalled on Windows 11; on Windows 10 grab the Evergreen runtime from Microsoft.
4. **Node.js 18+** — https://nodejs.org.

Then, from the project folder:

```bash
npm install          # frontend deps
npm run tauri dev    # launches the app with hot reload
```

To produce a distributable `.exe` / installer:

```bash
npm run tauri build  # output in src-tauri/target/release/
```

> First `tauri dev`/`build` is slow — it compiles the Rust + Tauri toolchain and SQLite from source. Subsequent runs are fast.

## Where the data lives

A single SQLite file, `timesheet.db`, in your OS app-data dir (on Windows: `%APPDATA%\com.callum.timesheet`). Back it up by copying that file. It's seeded with a starter set of timecodes on first run — edit them in **Manage codes**.

## Architecture

```
src/                     Vue 3 frontend
  api.js                 invoke() wrappers around the Rust commands
  store.js               small reactive shared store
  lib/                   fuzzy search, duration parsing, week/date helpers
  components/
    QuickCapture.vue     the <5s capture flow (typeahead, recents, timer)
    WeeklyGrid.vue       rollup matrix + totals + copy-for-Replicon
    TimecodePicker.vue   reusable fuzzy timecode selector
    UnresolvedTray.vue   "needs timecode" resolution
    TimecodeManager.vue  add codes, retire/reactivate (active flag)
src-tauri/               Rust backend
  src/db.rs              schema, migrations, seed data
  src/models.rs          Timecode, TimeEntry
  src/commands.rs        all DB operations exposed to the frontend
  src/lib.rs             Tauri builder, manages the SQLite connection
```

### Data model

```
timecode(id, client, project, task, active)         -- active flag, never deleted
time_entry(id, timecode_id?, date, duration_minutes, description, created_at)
```

`timecode_id` is nullable to support capture-now-resolve-later. Retiring a code (`active = 0`) drops it from the typeahead but leaves old entries intact (`ON DELETE SET NULL`). The grid is the pivot of:

```sql
SELECT timecode_id, date, SUM(duration_minutes)
FROM time_entry WHERE date BETWEEN ? AND ? GROUP BY timecode_id, date
```

## Notes

- The included app icon is a generated placeholder. To swap it, drop a 1024×1024 PNG in and run `npm run tauri icon path/to/icon.png` (regenerates every size, including macOS `.icns`).
- Verified so far: the Vue frontend builds clean via Vite; the full SQLite schema, seed, and every command's SQL were exercised against SQLite and pass. The Rust compile itself needs the toolchain above on your machine (it couldn't be run in the build sandbox).
```
