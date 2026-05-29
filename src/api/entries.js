import { invoke } from "@tauri-apps/api/core";

export const addEntry = (timecodeId, date, durationMinutes, description) =>
  invoke("add_entry", { timecodeId, date, durationMinutes, description });

export const updateEntry = (id, timecodeId, date, durationMinutes, description) =>
  invoke("update_entry", { id, timecodeId, date, durationMinutes, description });

export const deleteEntry = (id) => invoke("delete_entry", { id });

export const entriesForWeek = (weekStart, weekEnd) =>
  invoke("entries_for_week", { weekStart, weekEnd });

export const unresolvedEntries = () => invoke("unresolved_entries");

export const totals = (today, weekStart, weekEnd) =>
  invoke("totals", { today, weekStart, weekEnd });
