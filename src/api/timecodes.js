import { invoke } from "@tauri-apps/api/core";

export const listTimecodes = (activeOnly = true) =>
  invoke("list_timecodes", { activeOnly });

export const addTimecode = (client, project, task) =>
  invoke("add_timecode", { client, project, task });

export const setTimecodeActive = (id, active) =>
  invoke("set_timecode_active", { id, active });

export const setTimecodeHidden = (id, hidden) =>
  invoke("set_timecode_hidden", { id, hidden });

export const recentTimecodes = (limit = 5) =>
  invoke("recent_timecodes", { limit });
