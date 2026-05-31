<script setup>
import { ref, onMounted } from "vue";
import { store } from "./store/index.js";
import * as api from "./api/index.js";
import { startEodWatcher } from "./composables/useEodReminder.js";
import { checkForUpdate, updateState } from "./composables/useUpdater.js";
import QuickCapture from "./components/capture/QuickCapture.vue";
import WeeklyGrid from "./components/grid/WeeklyGrid.vue";
import UnresolvedTray from "./components/timecodes/UnresolvedTray.vue";
import TimecodeManager from "./components/timecodes/TimecodeManager.vue";
import Settings from "./components/settings/Settings.vue";
import SearchEntries from "./components/search/SearchEntries.vue";
import WhatsNew from "./components/layout/WhatsNew.vue";
import UpdatePrompt from "./components/layout/UpdatePrompt.vue";
import TotalsBar from "./components/layout/TotalsBar.vue";
import WindowControls from "./components/layout/WindowControls.vue";
import QuickAddMini from "./components/capture/QuickAddMini.vue";
import StatusBar from "./components/layout/StatusBar.vue";
import About from "./components/layout/About.vue";

// The same bundle serves two windows: the full app, and a compact quick-add
// popup loaded at index.html#quick.
const isQuick = typeof window !== "undefined" && window.location.hash === "#quick";

const tab = ref("capture"); // 'capture' | 'grid'
const showTray = ref(false);
const showManager = ref(false);
const showSettings = ref(false);
const showSearch = ref(false);
const showAbout = ref(false);
const showWhatsNew = ref(false);
const showUpdate = ref(false);
const checkMsg = ref("");
const menu = ref({ open: false, x: 0, y: 0 });
const ready = ref(false);

function openMenu(e) {
  menu.value = { open: true, x: e.clientX, y: e.clientY };
}
function closeMenu() {
  menu.value.open = false;
}

function refreshOnFocus() {
  store.refreshAll();
}

async function maybeAutoSync() {
  const s = store.settings;
  if (s.sync_on_startup === "0") return;
  if (!(s.replicon_base_url && s.replicon_company && s.replicon_username)) return;
  const last = s.replicon_last_sync ? new Date(s.replicon_last_sync) : null;
  if (last && last.toDateString() === new Date().toDateString()) return; // already synced today
  try {
    if (!(await api.repliconHasPassword())) return;
    const res = await api.repliconSyncTimecodes();
    if (res.ok) {
      await store.saveSetting("replicon_last_sync", new Date().toISOString());
      await store.loadTimecodes();
    }
  } catch (e) {
    /* silent — the manual Sync button in Timecodes is still there */
  }
}

async function manualCheck() {
  checkMsg.value = "Checking…";
  await checkForUpdate();
  if (updateState.available) {
    checkMsg.value = "";
    showUpdate.value = true;
  } else {
    checkMsg.value = updateState.error
      ? "Couldn't check for updates."
      : "You're on the latest version.";
    setTimeout(() => (checkMsg.value = ""), 2600);
  }
}

onMounted(async () => {
  if (isQuick) return; // the mini window manages itself
  await store.refreshAll();
  ready.value = true;
  startEodWatcher();
  checkForUpdate(); // silent; surfaces in the status bar if an update exists
  maybeAutoSync(); // once-a-day background timecode refresh if connected
  // Pick up entries added from the quick window when returning to the app.
  window.addEventListener("focus", refreshOnFocus);
});
</script>

<template>
  <QuickAddMini v-if="isQuick" />

  <div v-else class="app">
    <header class="topbar" data-tauri-drag-region>
      <div class="brand" data-tauri-drag-region @contextmenu.prevent="openMenu">
        <span class="logo">⏱</span>
        <span class="name">Bletchley</span>
      </div>
      <nav class="tabs">
        <button :class="{ on: tab === 'capture' }" @click="tab = 'capture'">Capture</button>
        <button :class="{ on: tab === 'grid' }" @click="tab = 'grid'">Weekly grid</button>
      </nav>
      <div class="actions">
        <TotalsBar v-if="ready" />
        <div class="vsep"></div>
        <button
          class="ghost tray-btn"
          :class="{ alert: store.unresolved.length }"
          :title="store.unresolved.length ? store.unresolved.length + ' need a timecode' : 'Nothing needs a timecode'"
          @click="showTray = true"
        >
          <span v-if="store.unresolved.length">⚠ {{ store.unresolved.length }}</span>
          <span v-else class="quiet">✓</span>
        </button>
        <button class="ghost" title="Search entries" @click="showSearch = true">🔍</button>
        <button class="ghost" @click="showManager = true">Timecodes</button>
        <button class="ghost" title="Settings" @click="showSettings = true">⚙</button>
      </div>
      <WindowControls />
    </header>

    <main v-if="ready">
      <QuickCapture v-show="tab === 'capture'" @open-codes="showManager = true" />
      <WeeklyGrid v-if="tab === 'grid'" @open-tray="showTray = true" />
    </main>
    <main v-else class="loading">Loading…</main>

    <StatusBar v-if="ready" @open-update="showUpdate = true" />

    <UnresolvedTray v-if="showTray" @close="showTray = false" />
    <TimecodeManager v-if="showManager" @close="showManager = false" />
    <Settings v-if="showSettings" @close="showSettings = false" />
    <SearchEntries v-if="showSearch" @close="showSearch = false" />
    <About v-if="showAbout" @close="showAbout = false" />
    <WhatsNew v-if="showWhatsNew" @close="showWhatsNew = false" />
    <UpdatePrompt v-if="showUpdate" @close="showUpdate = false" />
    <div v-if="checkMsg" class="toast">{{ checkMsg }}</div>

    <div
      v-if="menu.open"
      class="ctx-backdrop"
      @click="closeMenu"
      @contextmenu.prevent="closeMenu"
    >
      <div class="ctx-menu" :style="{ top: menu.y + 'px', left: menu.x + 'px' }" @click.stop>
        <button @click="showWhatsNew = true; closeMenu()">What's new</button>
        <button @click="showAbout = true; closeMenu()">About Bletchley</button>
        <button @click="manualCheck(); closeMenu()">Check for updates</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.app {
  height: 100%;
  display: flex;
  flex-direction: column;
}
.topbar {
  display: flex;
  align-items: center;
  gap: 20px;
  padding: 8px 10px 8px 18px;
  border-bottom: 1px solid var(--border);
  background: var(--surface);
  -webkit-user-select: none;
  user-select: none;
}
.brand {
  display: flex;
  align-items: center;
  gap: 7px;
  font-weight: 700;
}
.logo {
  font-size: 18px;
}
.name {
  letter-spacing: -0.01em;
}
.tabs {
  display: flex;
  gap: 4px;
}
.tabs button {
  border: none;
  background: transparent;
  color: var(--text-dim);
  font-weight: 600;
  padding: 6px 14px;
  border-radius: 999px;
}
.tabs button.on {
  background: var(--accent-soft);
  color: var(--accent);
}
.actions {
  margin-left: auto;
  display: flex;
  align-items: center;
  gap: 8px;
}
.vsep {
  width: 1px;
  height: 24px;
  background: var(--border);
  margin: 0 2px;
}
.tray-btn.alert {
  background: var(--warn-soft);
  color: var(--warn);
  border-color: transparent;
  font-weight: 700;
}
.tray-btn .quiet {
  color: var(--ok);
}
main {
  flex: 1;
  overflow-y: auto;
  padding: 22px 18px 40px;
}
.loading {
  display: grid;
  place-items: center;
  color: var(--text-faint);
}
.ctx-backdrop {
  position: fixed;
  inset: 0;
  z-index: 60;
}
.ctx-menu {
  position: fixed;
  min-width: 170px;
  background: var(--surface);
  border: 1px solid var(--border-strong);
  border-radius: var(--radius-sm);
  box-shadow: var(--shadow);
  padding: 4px;
  display: flex;
  flex-direction: column;
}
.ctx-menu button {
  text-align: left;
  border: none;
  background: transparent;
  border-radius: 5px;
  padding: 7px 10px;
  font-size: 13px;
}
.ctx-menu button:hover {
  background: var(--accent-soft);
  color: var(--accent);
}
.toast {
  position: fixed;
  bottom: 40px;
  left: 50%;
  transform: translateX(-50%);
  background: var(--surface);
  border: 1px solid var(--border-strong);
  border-radius: 999px;
  box-shadow: var(--shadow);
  padding: 8px 16px;
  font-size: 13px;
  z-index: 80;
}
</style>
