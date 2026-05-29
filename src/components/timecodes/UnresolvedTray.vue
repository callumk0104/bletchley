<script setup>
import { store } from "../../store/index.js";
import * as api from "../../api/index.js";
import { formatDuration } from "../../lib/duration.js";
import TimecodePicker from "../common/TimecodePicker.vue";

const emit = defineEmits(["close"]);

async function assign(entry, tc) {
  await api.updateEntry(entry.id, tc.id, entry.date, entry.duration_minutes, entry.description);
  store.lastUsedId = tc.id;
  await refresh();
}

async function remove(entry) {
  await api.deleteEntry(entry.id);
  await refresh();
}

async function refresh() {
  await Promise.all([store.loadUnresolved(), store.loadRecents()]);
  store.bump();
}
</script>

<template>
  <div class="modal-backdrop" @click.self="emit('close')">
    <div class="modal">
      <header>
        <h2>Needs a timecode
          <span class="badge warn" v-if="store.unresolved.length">
            {{ store.unresolved.length }}
          </span>
        </h2>
        <button class="ghost" @click="emit('close')">Done</button>
      </header>
      <div class="body">
        <p v-if="!store.unresolved.length" class="empty">
          Nothing waiting — every entry has a timecode. 🎉
        </p>
        <div v-for="e in store.unresolved" :key="e.id" class="row">
          <div class="meta">
            <span class="mono dur">{{ formatDuration(e.duration_minutes) }}</span>
            <span class="date">{{ e.date }}</span>
            <span class="desc">{{ e.description || "—" }}</span>
            <button class="danger small" @click="remove(e)">delete</button>
          </div>
          <TimecodePicker
            :recents="store.recents"
            placeholder="Assign a timecode…"
            @select="(tc) => assign(e, tc)"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.empty {
  color: var(--text-dim);
  text-align: center;
  padding: 20px;
}
.row {
  padding: 12px 0;
  border-bottom: 1px solid var(--border);
}
.row:last-child {
  border-bottom: none;
}
.meta {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 8px;
}
.dur {
  font-weight: 600;
  color: var(--warn);
}
.date {
  color: var(--text-faint);
  font-variant-numeric: tabular-nums;
}
.desc {
  flex: 1;
  color: var(--text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.small {
  padding: 4px 9px;
  font-size: 12px;
}
</style>
