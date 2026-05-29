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
</style>
