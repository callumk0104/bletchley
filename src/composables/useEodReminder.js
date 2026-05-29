// End-of-day reminder. Runs while the app is open: once per day, at the
// configured time, if today's logged hours are under target, it fires a
// desktop notification. State is kept in the settings table so it won't
// double-fire across restarts.
import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from "@tauri-apps/plugin-notification";
import { store } from "../store/index.js";
import { isoDate } from "../lib/dates.js";

export async function ensureNotificationPermission() {
  try {
    let granted = await isPermissionGranted();
    if (!granted) granted = (await requestPermission()) === "granted";
    return granted;
  } catch (e) {
    return false;
  }
}

// Fire a notification right now, bypassing all timing/target guards.
// Returns a human-readable status so the UI can report what happened.
export async function sendTestNotification() {
  if (!(await ensureNotificationPermission())) {
    return "Permission denied — enable notifications for Bletchley in Windows settings.";
  }
  try {
    await sendNotification({
      title: "Bletchley",
      body: "Test reminder — notifications are working.",
    });
    return "Sent. If no toast appeared, check Windows → Notifications (and turn off Focus Assist / Do Not Disturb).";
  } catch (e) {
    return "Failed to send: " + e;
  }
}

function parseHM(s) {
  const m = /^(\d{1,2}):(\d{2})$/.exec(s || "");
  return m ? { h: +m[1], min: +m[2] } : null;
}

let timer = null;

async function tick() {
  try {
    if (store.settings.eod_reminder_enabled !== "1") return;
    const hm = parseHM(store.settings.eod_reminder_time);
    if (!hm) return;

    const now = new Date();
    const today = isoDate(now);
    if (store.settings.eod_last_fired === today) return; // already evaluated today

    const past =
      now.getHours() > hm.h || (now.getHours() === hm.h && now.getMinutes() >= hm.min);
    if (!past) return;

    await store.loadTotals();
    // Mark handled for today regardless, so we evaluate exactly once at the time.
    await store.saveSetting("eod_last_fired", today);

    if (store.todayMinutes >= store.dailyTargetMinutes) return; // hit target, no nudge
    if (!(await ensureNotificationPermission())) return;

    const logged = (store.todayMinutes / 60).toFixed(1);
    const target = (store.dailyTargetMinutes / 60).toFixed(1);
    sendNotification({
      title: "Bletchley",
      body: `You've logged ${logged}h today (target ${target}h). Anything missing?`,
    });
  } catch (e) {
    /* never let the watcher throw */
  }
}

export function startEodWatcher() {
  if (timer) return;
  timer = setInterval(tick, 60000);
  tick(); // also check right away (e.g. app opened in the evening)
}
