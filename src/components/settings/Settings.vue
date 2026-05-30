<script setup>
import { ref, computed, onMounted } from "vue";
import { store } from "../../store/index.js";
import * as api from "../../api/index.js";
import { ensureNotificationPermission, sendTestNotification } from "../../composables/useEodReminder.js";

const emit = defineEmits(["close"]);

// Daily target edited in hours, stored as minutes.
const targetHours = ref((store.dailyTargetMinutes / 60).toString());
function saveTarget() {
  const h = parseFloat(targetHours.value);
  if (Number.isFinite(h) && h > 0) {
    store.saveSetting("daily_target_minutes", Math.round(h * 60));
  }
}

const theme = computed({
  get: () => store.settings.theme || "auto",
  set: (v) => store.saveSetting("theme", v),
});

const eodEnabled = computed({
  get: () => store.settings.eod_reminder_enabled === "1",
  set: async (v) => {
    await store.saveSetting("eod_reminder_enabled", v ? "1" : "0");
    if (v) {
      await store.saveSetting("eod_last_fired", ""); // allow it to fire again today
      ensureNotificationPermission();
    }
  },
});
const eodTime = computed({
  get: () => store.settings.eod_reminder_time || "17:00",
  set: async (v) => {
    await store.saveSetting("eod_reminder_time", v);
    await store.saveSetting("eod_last_fired", ""); // re-arm for the new time today
  },
});

const testMsg = ref("");
async function testReminder() {
  testMsg.value = "Sending…";
  testMsg.value = await sendTestNotification();
}

const hotkey = ref(store.settings.hotkey || "CmdOrCtrl+Shift+Space");
function saveHotkey() {
  store.saveSetting("hotkey", hotkey.value.trim());
}

// Backups
const backupsDir = ref("");
const backupMsg = ref("");
onMounted(async () => {
  try {
    backupsDir.value = await api.backupsPath();
  } catch (e) {
    /* ignore */
  }
});
async function backupNow() {
  backupMsg.value = "";
  try {
    const path = await api.backupNow();
    backupMsg.value = "Saved: " + path;
  } catch (e) {
    backupMsg.value = "Backup failed: " + e;
  }
}

// --- Replicon connection (Phase 1: read-only) ---
const rBaseUrl = ref(store.settings.replicon_base_url || "https://eu1.replicon.com");
const rCompany = ref(store.settings.replicon_company || "");
const rUsername = ref(store.settings.replicon_username || "");
const rPassword = ref("");
const rHasPw = ref(false);
const rTesting = ref(false);
const rMsg = ref("");
const rOk = ref(false);
const rBody = ref("");
function saveR(key, val) {
  store.saveSetting(key, (val || "").trim());
}
async function savePassword() {
  await api.repliconSetPassword(rPassword.value);
  rHasPw.value = rPassword.value.length > 0;
  rPassword.value = "";
  rMsg.value = "Password saved to your OS keychain.";
  rOk.value = true;
  rBody.value = "";
}
async function testRepl() {
  rTesting.value = true;
  rMsg.value = "";
  rBody.value = "";
  // Persist the current field values first — they may not have been blurred,
  // and the backend reads them straight from the settings table.
  await store.saveSetting("replicon_base_url", rBaseUrl.value.trim());
  await store.saveSetting("replicon_company", rCompany.value.trim());
  await store.saveSetting("replicon_username", rUsername.value.trim());
  try {
    const res = await api.repliconTestConnection();
    rOk.value = res.ok;
    rMsg.value = res.message + (res.status ? `  (HTTP ${res.status})` : "");
    rBody.value = res.body || "";
  } catch (e) {
    rOk.value = false;
    rMsg.value = String(e);
  } finally {
    rTesting.value = false;
  }
}
onMounted(async () => {
  try {
    rHasPw.value = await api.repliconHasPassword();
  } catch (e) {
    /* ignore */
  }
});
</script>

<template>
  <div class="modal-backdrop" @click.self="emit('close')">
    <div class="modal">
      <header>
        <h2>Settings</h2>
        <button class="ghost" @click="emit('close')">Done</button>
      </header>
      <div class="body">
        <div class="setting">
          <label>Replicon connection</label>
          <div class="repl-grid">
            <input v-model="rBaseUrl" placeholder="Base URL" @change="saveR('replicon_base_url', rBaseUrl)" />
            <input v-model="rCompany" placeholder="Company key (e.g. Enigma)" @change="saveR('replicon_company', rCompany)" />
            <input v-model="rUsername" placeholder="Username" @change="saveR('replicon_username', rUsername)" />
            <input
              v-model="rPassword"
              type="password"
              :placeholder="rHasPw ? 'Password stored — leave blank to keep' : 'Password'"
            />
          </div>
          <div class="repl-actions">
            <button @click="savePassword">Save password</button>
            <button @click="testRepl" :disabled="rTesting">
              {{ rTesting ? "Testing…" : "Test connection" }}
            </button>
          </div>
          <p v-if="rMsg" class="note" :class="{ 'ok-note': rOk }">{{ rMsg }}</p>
          <pre v-if="rBody" class="repl-body mono">{{ rBody }}</pre>
          <p class="note">
            Password lives in your OS keychain, never the database. Read-only for now
            (used to sync timecodes) — writing hours stays disabled until you enable it.
          </p>
        </div>

        <div class="setting">
          <label>Daily target (hours)</label>
          <input
            class="narrow"
            v-model="targetHours"
            type="number"
            min="0.5"
            step="0.5"
            @change="saveTarget"
          />
          <p class="note">Days under this are flagged in the grid.</p>
        </div>

        <div class="setting">
          <label>Theme</label>
          <select v-model="theme" class="narrow">
            <option value="auto">Auto (match system)</option>
            <option value="light">Light</option>
            <option value="dark">Dark</option>
          </select>
        </div>

        <div class="setting">
          <label>
            <input type="checkbox" v-model="eodEnabled" />
            End-of-day reminder
          </label>
          <input type="time" v-model="eodTime" class="narrow" :disabled="!eodEnabled" />
          <p class="note">
            Nudges you at this time if today's logged hours are under target.
            Only fires while the app is running.
          </p>
          <button class="test-btn" @click="testReminder">Send test notification</button>
          <p v-if="testMsg" class="note">{{ testMsg }}</p>
        </div>

        <div class="setting">
          <label>Global quick-add hotkey</label>
          <div class="hotkey-row">
            <input v-model="hotkey" class="mono" @change="saveHotkey" />
          </div>
          <p class="note">
            e.g. <span class="mono">CmdOrCtrl+Shift+Space</span>. Pops the quick-add
            window from any app. Takes effect after restarting the app.
          </p>
        </div>

        <div class="setting">
          <label>Data &amp; backups</label>
          <button @click="backupNow">Back up now</button>
          <p class="note">
            A dated backup is also taken automatically each time the app starts
            (last 14 kept).<br />
            Folder: <span class="mono break">{{ backupsDir || "…" }}</span>
          </p>
          <p v-if="backupMsg" class="note ok-note mono break">{{ backupMsg }}</p>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.setting {
  padding: 14px 0;
  border-bottom: 1px solid var(--border);
}
.setting:last-child {
  border-bottom: none;
}
.setting > label {
  display: block;
  font-weight: 600;
  margin-bottom: 8px;
}
.setting > label input[type="checkbox"] {
  width: auto;
  margin-right: 6px;
}
.narrow {
  width: 200px;
}
select.narrow {
  width: 210px;
}
.note {
  color: var(--text-dim);
  font-size: 12px;
  margin: 8px 0 0;
}
.ok-note {
  color: var(--ok);
}
.break {
  word-break: break-all;
}
.hotkey-row {
  display: flex;
  gap: 8px;
}
.test-btn {
  margin-top: 10px;
}
.repl-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
}
.repl-actions {
  display: flex;
  gap: 8px;
  margin-top: 10px;
}
.repl-body {
  margin: 8px 0 0;
  padding: 8px;
  background: var(--surface-2);
  border-radius: var(--radius-sm);
  font-size: 11px;
  max-height: 120px;
  overflow: auto;
  white-space: pre-wrap;
  word-break: break-all;
}
</style>
