import { invoke } from "@tauri-apps/api/core";

export const repliconSetPassword = (password) =>
  invoke("replicon_set_password", { password });

export const repliconHasPassword = () => invoke("replicon_has_password");

export const repliconTestConnection = () => invoke("replicon_test_connection");

export const repliconSyncTimecodes = () => invoke("replicon_sync_timecodes");
