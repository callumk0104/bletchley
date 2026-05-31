<script setup>
import { computed } from "vue";
import { store } from "../../store/index.js";
import { updateState } from "../../composables/useUpdater.js";

const emit = defineEmits(["open-update"]);

// Injected at build time from package.json (see vite.config.js).
const version = typeof __APP_VERSION__ !== "undefined" ? __APP_VERSION__ : "dev";

const themes = [
  { v: "auto", label: "Auto (match system)" },
  { v: "light", label: "Light" },
  { v: "dark", label: "Dark" },
];
const current = computed(() => store.settings.theme || "auto");
function setTheme(v) {
  store.saveSetting("theme", v);
}

const updateTitle = computed(() => {
  if (updateState.error) return `Update failed: ${updateState.error} — click to retry`;
  return updateState.downloading
    ? `Downloading ${updateState.progress}%…`
    : `Version ${updateState.version} is available — click to update and restart`;
});
</script>

<template>
  <footer class="statusbar">
    <div class="left">
      <span class="ver mono">v{{ version }}</span>
      <button
        v-if="updateState.available"
        class="update-pill"
        :class="{ failed: updateState.error && !updateState.downloading }"
        :disabled="updateState.downloading"
        :title="updateTitle"
        @click="emit('open-update')"
      >
        <span class="dot"></span>
        <span v-if="updateState.downloading">Updating {{ updateState.progress }}%</span>
        <span v-else-if="updateState.error">Update failed — retry</span>
        <span v-else>Update to {{ updateState.version }}</span>
      </button>
    </div>
    <div class="spacer"></div>
    <div class="right">
      <span class="lbl">Theme</span>
      <div class="seg">
        <button
          v-for="t in themes"
          :key="t.v"
          :class="{ on: current === t.v }"
          :title="t.label"
          :aria-label="t.label"
          @click="setTheme(t.v)"
        >
          <svg v-if="t.v === 'auto'" viewBox="0 0 16 16" width="13" height="13">
            <circle cx="8" cy="8" r="6" fill="none" stroke="currentColor" stroke-width="1.4" />
            <path d="M8 2 A6 6 0 0 0 8 14 Z" fill="currentColor" />
          </svg>
          <svg
            v-else-if="t.v === 'light'"
            viewBox="0 0 16 16"
            width="13"
            height="13"
            fill="none"
            stroke="currentColor"
            stroke-width="1.4"
            stroke-linecap="round"
          >
            <circle cx="8" cy="8" r="3" />
            <path
              d="M8 1v1.6M8 13.4V15M1 8h1.6M13.4 8H15M3.2 3.2l1.1 1.1M11.7 11.7l1.1 1.1M12.8 3.2l-1.1 1.1M4.3 11.7l-1.1 1.1"
            />
          </svg>
          <svg v-else viewBox="0 0 16 16" width="13" height="13">
            <path d="M13 9.7 A5.5 5.5 0 1 1 6.3 3 A4.3 4.3 0 0 0 13 9.7 Z" fill="currentColor" />
          </svg>
        </button>
      </div>
    </div>
  </footer>
</template>

<style scoped>
.statusbar {
  display: flex;
  align-items: center;
  gap: 10px;
  height: 26px;
  padding: 0 12px;
  border-top: 1px solid var(--border);
  background: var(--surface);
  font-size: 11px;
  color: var(--text-dim);
  -webkit-user-select: none;
  user-select: none;
}
.left {
  display: flex;
  align-items: center;
  gap: 8px;
}
.spacer {
  flex: 1;
}
.ver {
  color: var(--text-faint);
}
.update-pill {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  border: none;
  background: var(--accent-soft);
  color: var(--accent);
  font-weight: 600;
  font-size: 11px;
  padding: 2px 8px;
  border-radius: 999px;
}
.update-pill:hover {
  filter: brightness(1.05);
}
.update-pill .dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--accent);
}
.update-pill.failed {
  background: color-mix(in srgb, var(--danger, #d9534f) 18%, transparent);
  color: var(--danger, #d9534f);
}
.update-pill.failed .dot {
  background: var(--danger, #d9534f);
}
.right {
  display: flex;
  align-items: center;
  gap: 6px;
}
.lbl {
  color: var(--text-faint);
}
.seg {
  display: flex;
  border: 1px solid var(--border);
  border-radius: 6px;
  overflow: hidden;
}
.seg button {
  display: grid;
  place-items: center;
  border: none;
  background: transparent;
  color: var(--text-dim);
  padding: 2px 8px;
  border-radius: 0;
}
.seg button:hover {
  background: var(--surface-2);
  color: var(--text);
}
.seg button.on {
  background: var(--accent-soft);
  color: var(--accent);
}
</style>
