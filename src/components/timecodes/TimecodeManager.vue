<script setup>
import { ref, computed } from "vue";
import { store } from "../../store/index.js";
import * as api from "../../api/index.js";
import { fuzzySearch } from "../../lib/fuzzy.js";

const emit = defineEmits(["close"]);

const client = ref("");
const project = ref("");
const task = ref("");
const error = ref("");
const filter = ref("");
const showRetired = ref(false);

const visible = computed(() => {
  let list = store.timecodes;
  if (!showRetired.value) list = list.filter((t) => t.active);
  return fuzzySearch(filter.value, list, (t) => t.label);
});

async function addCode() {
  error.value = "";
  if (!client.value.trim() || !project.value.trim() || !task.value.trim()) {
    error.value = "Client, project and task are all required.";
    return;
  }
  try {
    await api.addTimecode(client.value, project.value, task.value);
    // Keep client/project so adding several tasks to one project is quick.
    task.value = "";
    await store.loadTimecodes();
  } catch (e) {
    error.value = String(e);
  }
}

async function toggleActive(tc) {
  await api.setTimecodeActive(tc.id, !tc.active);
  await store.loadTimecodes();
  await store.loadRecents();
}

// Datalists for quick reuse of existing clients/projects.
const clients = computed(() => [...new Set(store.timecodes.map((t) => t.client))].sort());
const projects = computed(() =>
  [...new Set(store.timecodes.filter((t) => !client.value || t.client === client.value).map((t) => t.project))].sort()
);
</script>

<template>
  <div class="modal-backdrop" @click.self="emit('close')">
    <div class="modal">
      <header>
        <h2>Timecodes</h2>
        <button class="ghost" @click="emit('close')">Done</button>
      </header>
      <div class="body">
        <div class="add-box">
          <div class="add-fields">
            <input v-model="client" list="clients" placeholder="Client" />
            <input v-model="project" list="projects" placeholder="Project" />
            <input v-model="task" placeholder="Task" @keydown.enter="addCode" />
            <button class="primary" @click="addCode">Add code</button>
          </div>
          <datalist id="clients">
            <option v-for="c in clients" :key="c" :value="c" />
          </datalist>
          <datalist id="projects">
            <option v-for="p in projects" :key="p" :value="p" />
          </datalist>
          <p v-if="error" class="err">{{ error }}</p>
        </div>

        <div class="list-head">
          <input v-model="filter" class="filter" placeholder="Filter…" />
          <label class="chk">
            <input type="checkbox" v-model="showRetired" /> show retired
          </label>
        </div>

        <div class="list">
          <div v-for="tc in visible" :key="tc.id" class="trow" :class="{ off: !tc.active }">
            <span class="label">{{ tc.label }}</span>
            <button class="ghost small" @click="toggleActive(tc)">
              {{ tc.active ? "Retire" : "Reactivate" }}
            </button>
          </div>
          <p v-if="!visible.length" class="empty">No timecodes match.</p>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.add-box {
  background: var(--surface-2);
  border-radius: var(--radius-sm);
  padding: 12px;
  margin-bottom: 14px;
}
.add-fields {
  display: flex;
  gap: 8px;
}
.add-fields input {
  flex: 1;
}
.add-fields button {
  white-space: nowrap;
}
.err {
  color: var(--danger);
  font-size: 13px;
  margin: 8px 0 0;
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
