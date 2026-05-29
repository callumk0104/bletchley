<script setup>
import { ref, onMounted, onUnmounted, computed } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { store } from "../../store/index.js";
import * as api from "../../api/index.js";
import { parseDuration } from "../../lib/duration.js";
import { isoDate } from "../../lib/dates.js";
import TimecodePicker from "../common/TimecodePicker.vue";

// Compact capture shown in the always-on-top "quick" window, toggled by the
// global hotkey. Adds an entry for *today* and hides itself.
const win = getCurrentWindow();
const selected = ref(null);
const description = ref("");
const durationText = ref("");
const error = ref("");
const picker = ref(null);
const descEl = ref(null);

const canAdd = computed(() => !!parseDuration(durationText.value));

async function load() {
  await store.loadTimecodes();
  await store.loadRecents();
  if (store.lastUsedId != null && store.byId[store.lastUsedId]) {
    selected.value = store.byId[store.lastUsedId];
  }
}

function focusPicker() {
  requestAnimationFrame(() => picker.value?.focus?.());
}
function focusDesc() {
  requestAnimationFrame(() => descEl.value?.focus?.());
}

async function onWinFocus() {
  await store.loadRecents();
  if (!selected.value) focusPicker();
  else focusDesc();
}
function onKey(e) {
  if (e.key === "Escape") hide();
}

onMounted(async () => {
  await load();
  selected.value ? focusDesc() : focusPicker();
  window.addEventListener("focus", onWinFocus);
  window.addEventListener("keydown", onKey);
});
onUnmounted(() => {
  window.removeEventListener("focus", onWinFocus);
  window.removeEventListener("keydown", onKey);
});

function pick(tc) {
  selected.value = tc;
  focusDesc();
}
function clearCode() {
  selected.value = null;
  focusPicker();
}

function reset() {
  description.value = "";
  durationText.value = "";
  error.value = "";
}
async function hide() {
  reset();
  try {
    await win.hide();
  } catch (e) {
    /* not in a window */
  }
}

async function add() {
  const mins = parseDuration(durationText.value);
  if (!mins) {
    error.value = "Enter a duration, e.g. 1h or 30m.";
    return;
  }
  try {
    await api.addEntry(
      selected.value ? selected.value.id : null,
      isoDate(new Date()),
      mins,
      description.value.trim()
    );
    if (selected.value) store.lastUsedId = selected.value.id;
    await hide();
  } catch (e) {
    error.value = String(e);
  }
}
</script>

<template>
  <div class="mini">
    <div class="row">
      <div v-if="selected" class="chip">
        <span class="lbl">{{ selected.label }}</span>
        <button class="x" @click="clearCode" title="Change timecode">×</button>
      </div>
      <TimecodePicker
        v-else
        ref="picker"
        :recents="store.recents"
        placeholder="Timecode…  (Esc to close)"
        @select="pick"
      />
    </div>
    <div class="row two">
      <input
        ref="descEl"
        v-model="description"
        placeholder="What did you do?"
        @keydown.enter="add"
      />
      <input
        v-model="durationText"
        class="mono dur"
        placeholder="1 = 1h"
        @keydown.enter="add"
      />
      <button class="primary" :disabled="!canAdd" @click="add">Add</button>
    </div>
    <div v-if="error" class="err">{{ error }}</div>
  </div>
</template>

<style scoped>
.mini {
  padding: 12px 14px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.row.two {
  display: flex;
  gap: 8px;
}
.row.two input:first-child {
  flex: 1;
}
.dur {
  width: 72px;
  text-align: center;
}
.chip {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  background: var(--accent-soft);
  color: var(--accent);
  font-weight: 600;
  padding: 9px 12px;
  border-radius: var(--radius-sm);
}
.x {
  border: none;
  background: transparent;
  color: var(--accent);
  font-size: 16px;
  line-height: 1;
  padding: 0 4px;
}
.err {
  color: var(--danger);
  font-size: 12px;
}
</style>
