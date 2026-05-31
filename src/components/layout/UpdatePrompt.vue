<script setup>
import { computed } from "vue";
import { updateState, installUpdate } from "../../composables/useUpdater.js";
import { tokenize } from "../../lib/changelog.js";

const emit = defineEmits(["close"]);
const tokens = computed(() => tokenize(updateState.notes));
</script>

<template>
  <div class="modal-backdrop" @click.self="updateState.downloading || emit('close')">
    <div class="modal narrow">
      <header>
        <h2>Update available</h2>
        <button class="ghost" :disabled="updateState.downloading" @click="emit('close')">Later</button>
      </header>
      <div class="body">
        <p class="lead">
          Version <strong>{{ updateState.version }}</strong> is ready.
        </p>
        <ul v-if="tokens.length" class="notes">
          <template v-for="(t, i) in tokens" :key="i">
            <h4 v-if="t.type === 'h'">{{ t.text }}</h4>
            <li v-else-if="t.type === 'li'">{{ t.text }}</li>
            <hr v-else-if="t.type === 'hr'" />
            <p v-else-if="t.type === 'p'">{{ t.text }}</p>
          </template>
        </ul>
        <p v-if="updateState.error" class="err">{{ updateState.error }}</p>
        <div class="actions">
          <button class="ghost" :disabled="updateState.downloading" @click="emit('close')">Later</button>
          <button class="primary" :disabled="updateState.downloading" @click="installUpdate">
            {{ updateState.downloading ? `Updating ${updateState.progress}%…` : "Update & restart" }}
          </button>
        </div>
        <p class="hint">The app will close briefly to install, then reopen.</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.lead {
  font-size: 14px;
  margin: 0 0 10px;
}
.notes {
  list-style: none;
  padding: 0;
  margin: 0 0 12px;
  max-height: 240px;
  overflow-y: auto;
}
.notes h4 {
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--text-dim);
  margin: 8px 0 4px;
}
.notes li {
  position: relative;
  padding-left: 16px;
  margin: 4px 0;
  font-size: 13px;
  line-height: 1.45;
}
.notes li::before {
  content: "•";
  position: absolute;
  left: 3px;
  color: var(--accent);
}
.notes p {
  font-size: 13px;
  color: var(--text-dim);
  margin: 4px 0;
}
.notes hr {
  border: none;
  border-top: 1px solid var(--border);
  margin: 10px 0;
}
.actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 8px;
}
.err {
  color: var(--danger);
  font-size: 13px;
  margin: 6px 0;
}
.hint {
  color: var(--text-faint);
  font-size: 12px;
  margin: 10px 0 0;
}
</style>
