import { invoke } from "@tauri-apps/api/core";

export const getSettings = () => invoke("get_settings");

export const setSetting = (key, value) => invoke("set_setting", { key, value });
