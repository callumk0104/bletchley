import { invoke } from "@tauri-apps/api/core";

export const backupNow = () => invoke("backup_now");

export const backupsPath = () => invoke("backups_path");

export const updateTray = (text) => invoke("update_tray", { text });
