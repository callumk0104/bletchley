<script setup>
import { ref, onMounted } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";

const win = getCurrentWindow();
const maximized = ref(false);

async function refresh() {
  try {
    maximized.value = await win.isMaximized();
  } catch (e) {
    /* not in a Tauri window */
  }
}
onMounted(refresh);

const minimize = () => win.minimize();
async function toggleMax() {
  await win.toggleMaximize();
  refresh();
}
const close = () => win.close();
</script>

<template>
  <div class="win-controls">
    <button class="wc" @click="minimize" title="Minimize" aria-label="Minimize">
      <svg width="10" height="10" viewBox="0 0 10 10">
        <line x1="1" y1="5" x2="9" y2="5" stroke="currentColor" stroke-width="1" />
      </svg>
    </button>
    <button class="wc" @click="toggleMax" :title="maximized ? 'Restore' : 'Maximize'">
      <svg v-if="!maximized" width="10" height="10" viewBox="0 0 10 10">
        <rect x="1.5" y="1.5" width="7" height="7" fill="none" stroke="currentColor" stroke-width="1" />
      </svg>
      <svg v-else width="10" height="10" viewBox="0 0 10 10">
        <rect x="1.5" y="2.5" width="6" height="6" fill="none" stroke="currentColor" stroke-width="1" />
        <path d="M3.5 2.5 V1.5 H8.5 V6.5 H7.5" fill="none" stroke="currentColor" stroke-width="1" />
      </svg>
    </button>
    <button class="wc close" @click="close" title="Close" aria-label="Close">
      <svg width="10" height="10" viewBox="0 0 10 10">
        <line x1="1" y1="1" x2="9" y2="9" stroke="currentColor" stroke-width="1.3" />
        <line x1="9" y1="1" x2="1" y2="9" stroke="currentColor" stroke-width="1.3" />
      </svg>
    </button>
  </div>
</template>

<style scoped>
.win-controls {
  display: flex;
  gap: 2px;
  margin-left: 6px;
}
.wc {
  width: 36px;
  height: 30px;
  display: grid;
  place-items: center;
  border: none;
  background: transparent;
  border-radius: 6px;
  color: var(--text-dim);
  padding: 0;
}
.wc:hover {
  background: var(--surface-2);
  color: var(--text);
}
.wc.close:hover {
  background: #e81123;
  color: #fff;
}
</style>
