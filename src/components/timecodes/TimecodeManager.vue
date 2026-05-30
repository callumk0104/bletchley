<script setup>
import { ref, computed } from "vue";
import { store } from "../../store/index.js";
import * as api from "../../api/index.js";
import { fuzzySearch } from "../../lib/fuzzy.js";

const emit = defineEmits(["close"]);

const filter = ref("");
const showHidden = ref(false);

// --- Sync (Replicon is the source of truth for the code list) ---
const syncing = ref(false);
const syncMsg = ref("");
const syncOk = ref(false);

const hasConn = computed(
  () =>
    !!(
      store.settings.replicon_base_url &&
      store.settings.replicon_company &&
      store.settings.replicon_username
    )
);

const lastSync = computed(() => {
  const v = store.settings.replicon_last_sync;
  if (!v) return "";
  const d = new Date(v);
  return Number.isNaN(d.getTime()) ? "" : d.toLocaleString();
});

async function syncNow() {
  syncing.value = true;
  syncMsg.value = "";
  try {
    const res = await api.repliconSyncTimecodes();
    syncOk.value = res.ok;
    syncMsg.value = res.message;
    if (res.ok) {
      await store.saveSetting("replicon_last_sync", new Date().toISOString());
      await Promise.all([store.loadTimecodes(), store.loadRecents()]);
    }
  } catch (e) {
    syncOk.value = false;
    syncMsg.value = String(e);
  } finally {
    syncing.value = false;
  }
}

// --- Curation: hide/show codes in the picker (never touched by sync) ---
const visible = computed(() => {
  let list = store.timecodes.filter((t) => t.active);
  if (!showHidden.value) list = list.filter((t) => !t.hidden);
  return fuzzySearch(filter.value, list, (t) => t.label);
});

const hiddenCount = computed(
  () => store.timecodes.filter((t) => t.active && t.hidden).length
);

async function toggleHidden(tc) {
  await api.setTimecodeHidden(tc.id, !tc.hidden);
  await Promise.all([store.loadTimecodes(), store.loadRecents()]);
}
</script>

<template>
  <div class="modal-backdrop" @click.self="emit('close')">
    <div class="modal">
      <header>
        <h2>Timecodes</h2>
        <button class="ghost" @click="emit('close')">Done</button>
      </header>
      <div class="body">
        <div class="sync-box">
          <div class="sync-row">
            <button class="primary" @click="syncNow" :disabled="syncing || !hasConn">
              {{ syncing ? "Syncing…" : "Sync from Replicon" }}
            </button>
            <span v-if="lastSync" class="last">Last synced {{ lastSync }}</span>
          </div>
          <p v-if="!hasConn" class="note">
            Connect Replicon in <strong>Settings</strong> first, then sync to pull your
            bookable Client / Project / Task list.
          </p>
          <p v-if="syncMsg" class="note" :class="{ 'ok-note': syncOk }">{{ syncMsg }}</p>
        </div>

        <div class="list-head">
          <input v-model="filter" class="filter" placeholder="Filter…" />
          <label class="chk">
            <input type="checkbox" v-model="showHidden" /> show hidden ({{ hiddenCount }})
          </label>
        </div>

        <div class="list">
          <div v-for="tc in visible" :key="tc.id" class="trow" :class="{ off: tc.hidden }">
            <span class="label">{{ tc.label }}</span>
            <button class="ghost small" @click="toggleHidden(tc)">
              {{ tc.hidden ? "Show" : "Hide" }}
            </button>
          </div>
          <p v-if="!visible.length" class="empty">
            {{
              store.timecodes.length
                ? "No timecodes match."
                : "No timecodes yet — sync from Replicon above."
            }}
          </p>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.sync-box {
  background: var(--surface-2);
  border-radius: var(--radius-sm);
  padding: 12px;
  margin-bottom: 14px;
}
.sync-row {
  display: flex;
  align-items: center;
  gap: 12px;
}
.sync-row button {
  white-space: nowrap;
}
.last {
  font-size: 12px;
  color: var(--text-faint);
}
.note {
  color: var(--text-dim);
  font-size: 12px;
  margin: 8px 0 0;
}
.ok-note {
  color: var(--ok);
}
.list-head {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 8px;
}
.filter {
  flex: 1;
}
.chk {
  display: flex;
  align-items: center;
  gap: 5px;
  font-size: 12px;
  color: var(--text-dim);
  white-space: nowrap;
}
.chk input {
  width: auto;
}
.trow {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 4px;
  border-bottom: 1px solid var(--border);
}
.trow.off .label {
  color: var(--text-faint);
  text-decoration: line-through;
}
.small {
  padding: 4px 9px;
  font-size: 12px;
}
.empty {
  color: var(--text-faint);
  text-align: center;
  padding: 16px;
}
</style>
