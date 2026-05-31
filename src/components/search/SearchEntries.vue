<script setup>
import { ref, computed, onMounted, watch } from "vue";
import { store } from "../../store/index.js";
import * as api from "../../api/index.js";
import { formatDuration } from "../../lib/duration.js";
import EntryEditor from "../grid/EntryEditor.vue";

const emit = defineEmits(["close"]);

const query = ref("");
const results = ref([]);
const expandedId = ref(null);
const loading = ref(false);
let timer = null;

async function run() {
  loading.value = true;
  try {
    results.value = await api.searchEntries(query.value, 200);
  } finally {
    loading.value = false;
  }
}
watch(query, () => {
  clearTimeout(timer);
  timer = setTimeout(run, 150);
});
onMounted(run); // empty query → most recent entries

const totalMinutes = computed(() =>
  results.value.reduce((a, e) => a + e.duration_minutes, 0)
);

function labelFor(e) {
  return store.timecodeLabel(e.timecode_id) || "⚠ needs timecode";
}
function toggle(id) {
  expandedId.value = expandedId.value === id ? null : id;
}
async function onChanged() {
  await run();
  await store.loadUnresolved();
  store.bump();
}
</script>

<template>
  <div class="modal-backdrop" @click.self="emit('close')">
    <div class="modal">
      <header>
        <h2>Search entries</h2>
        <button class="ghost" @click="emit('close')">Done</button>
      </header>
      <div class="body">
        <input
          v-model="query"
          class="search"
          placeholder="Search description or client / project / task…"
          spellcheck="false"
          autocomplete="off"
          autofocus
        />
        <div class="summary">
          {{ results.length }} {{ results.length === 1 ? "entry" : "entries" }}
          · {{ formatDuration(totalMinutes) }}
          <span v-if="!query.trim()" class="hint">· most recent</span>
        </div>

        <div class="results">
          <template v-for="e in results" :key="e.id">
            <button class="res" :class="{ open: expandedId === e.id }" @click="toggle(e.id)">
              <span class="date mono">{{ e.date }}</span>
              <span class="lbl" :class="{ none: e.timecode_id == null }">{{ labelFor(e) }}</span>
              <span class="desc">{{ e.description || "—" }}</span>
              <span class="dur mono">{{ formatDuration(e.duration_minutes) }}</span>
            </button>
            <div v-if="expandedId === e.id" class="edit-wrap">
              <EntryEditor :entry="e" @changed="onChanged" />
            </div>
          </template>
          <p v-if="!results.length && !loading" class="empty">
            {{ query.trim() ? "No entries match." : "No entries yet." }}
          </p>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.search {
  width: 100%;
  font-size: 14px;
  padding: 10px 12px;
}
.summary {
  font-size: 12px;
  color: var(--text-dim);
  margin: 10px 2px;
}
.hint {
  color: var(--text-faint);
}
.results {
  display: flex;
  flex-direction: column;
}
.res {
  display: grid;
  grid-template-columns: 96px minmax(140px, 1fr) minmax(120px, 2fr) 64px;
  gap: 10px;
  align-items: center;
  width: 100%;
  text-align: left;
  border: none;
  background: transparent;
  border-bottom: 1px solid var(--border);
  padding: 9px 6px;
  font-size: 13px;
}
.res:hover {
  background: var(--surface-2);
}
.res.open {
  background: var(--accent-soft);
}
.date {
  color: var(--text-faint);
  font-variant-numeric: tabular-nums;
}
.lbl {
  font-weight: 600;
  color: var(--accent);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.lbl.none {
  color: var(--warn);
}
.desc {
  color: var(--text-dim);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.dur {
  text-align: right;
  font-weight: 600;
  color: var(--text-dim);
}
.edit-wrap {
  padding: 4px 6px 10px;
  border-bottom: 1px solid var(--border);
  background: var(--surface);
}
.empty {
  color: var(--text-faint);
  text-align: center;
  padding: 24px;
}
</style>
