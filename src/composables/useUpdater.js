// In-app auto-update via tauri-plugin-updater (source: GitHub Releases).
// On startup we silently check; if a newer signed release exists, the status
// bar shows an "update" pill. Clicking it downloads, installs, and relaunches.
import { reactive } from "vue";
import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

export const updateState = reactive({
  available: false,
  version: null,
  notes: "",
  checking: false,
  downloading: false,
  progress: 0, // 0..100, best-effort
  error: null,
  _update: null,
});

export async function checkForUpdate() {
  updateState.checking = true;
  updateState.error = null;
  try {
    const update = await check();
    if (update) {
      updateState.available = true;
      updateState.version = update.version;
      updateState.notes = update.body || "";
      updateState._update = update;
    }
  } catch (e) {
    // No network / no release / not yet configured — stay quiet in the UI.
    updateState.error = String(e);
  } finally {
    updateState.checking = false;
  }
}

export async function installUpdate() {
  const update = updateState._update;
  if (!update || updateState.downloading) return;
  updateState.downloading = true;
  updateState.error = null;
  try {
    let total = 0;
    let got = 0;
    await update.downloadAndInstall((event) => {
      if (event.event === "Started") total = event.data.contentLength || 0;
      else if (event.event === "Progress") {
        got += event.data.chunkLength || 0;
        if (total) updateState.progress = Math.round((got / total) * 100);
      } else if (event.event === "Finished") {
        updateState.progress = 100;
      }
    });
    await relaunch(); // restart into the new version
  } catch (e) {
    updateState.error = String(e);
    updateState.downloading = false;
  }
}
