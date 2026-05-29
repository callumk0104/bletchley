<script setup>
import { ref, computed } from "vue";
import { store } from "../../store/index.js";
import * as api from "../../api/index.js";
import { parseDuration } from "../../lib/duration.js";
import TimecodePicker from "../common/TimecodePicker.vue";

// Edits a single time_entry in place: its timecode, day, duration, and
// description — or deletes it. update_entry on the backend takes every field,
// so "wrong code" and "wrong day" are both fixable without re-capturing.
const props = defineProps({ entry: Object });
const emit = defineEmits(["changed"]);

const durText = ref(String((props.entry.duration_minutes / 60).toFixed(2)).replace(/\.00$/, "") + "h");
const desc = ref(props.entry.description);
const date = ref(props.entry.date);
const timecodeId = ref(props.entry.timecode_id);
const changingCode = ref(false);
const error = ref("");
const saving = ref(false);

const label = computed(() => store.timecodeLabel(timecodeId.value) || "⚠ no timecode");
const parsed = computed(() => parseDuration(durText.value));
const dirty = computed(
  () =>
    parsed.value !== props.entry.duration_minutes ||
    desc.value !== props.entry.description ||
    date.value !== props.entry.date ||
    timecodeId.value !== props.entry.timecode_id
);

function pickCode(tc) {
  timecodeId.value = tc.id;
  changingCode.value = false;
}

async function save() {
  error.value = "";
  const mins = parsed.value;
  if (!mins) {
    error.value = "Couldn't read that duration.";
    return;
  }
  saving.value = true;
  try {
    await api.updateEntry(props.entry.id, timecodeId.value, date.value, mins, desc.value.trim());
    await store.loadUnresolved();
    emit("changed");
  } catch (e) {
    error.value = String(e);
  } finally {
    saving.value = false;
  }
}

async function remove() {
  saving.value = true;
  try {
    await api.deleteEntry(props.entry.id);
    await store.loadUnresolved();
    emit("changed");
  } finally {
    saving.value = false;
  }
}
</script>

<template>
  <div class="editor">
    <div class="line">
      <!-- timecode -->
      <div class="code">
        <template v-if="!changingCode">
          <span class="code-label" :class="{ none: timecodeId == null }">{{ label }}</span>
          <button class="ghost xs" @click="changingCode = true">move</button>
        </template>
        <TimecodePicker
          v-else
          :recents="store.recents"
          placeholder="Move to timecode…"
          @select="pickCode"
        />
      </div>
    </div>

    <div class="line fields">
      <input type="date" v-model="date" class="date" />
      <input v-model="durText" class="mono dur" placeholder="2h" @keydown.enter="save" />
      <input v-model="desc" class="desc" placeholder="description" @keydown.enter="save" />
      <button class="primary xs" :disabled="!dirty || saving" @click="save">Save</button>
      <button class="danger xs" :disabled="saving" @click="remove">Delete</button>
    </div>
    <p v-if="error" class="err">{{ error }}</p>
  </div>
</template>

<style scoped>
.editor {
  padding: 8px 0;
  border-bottom: 1px solid var(--border);
}
.editor:last-child {
  border-bottom: none;
}
.line {
  display: flex;
  align-items: center;
  gap: 8px;
}
.code {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 8px;
}
.code-label {
  font-weight: 600;
  color: var(--accent);
}
.code-label.none {
  color: var(--warn);
}
.fields {
  margin-top: 7px;
}
.date {
  width: 150px;
}
.dur {
  width: 80px;
  text-align: center;
}
.desc {
  flex: 1;
}
.xs {
  padding: 5px 10px;
  font-size: 12px;
}
.err {
  color: var(--danger);
  font-size: 12px;
  margin: 6px 0 0;
}
</style>
