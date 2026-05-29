<script setup>
import { ref } from "vue";
import { updateState, checkForUpdate, installUpdate } from "../../composables/useUpdater.js";

const emit = defineEmits(["close"]);
const version = typeof __APP_VERSION__ !== "undefined" ? __APP_VERSION__ : "dev";

const checking = ref(false);
const checked = ref(false);
async function check() {
  checking.value = true;
  await checkForUpdate();
  checking.value = false;
  checked.value = true;
}
</script>

<template>
  <div class="modal-backdrop" @click.self="emit('close')">
    <div class="modal about">
      <header>
        <h2>About</h2>
        <button class="ghost" @click="emit('close')">Done</button>
      </header>
      <div class="body">
        <div class="hero">
          <span class="logo">⏱</span>
          <div>
            <div class="title">Bletchley</div>
            <div class="ver mono">v{{ version }}</div>
          </div>
        </div>

        <p class="tagline">Capture your week as it happens — then transcribe it into Replicon in minutes, not memory.</p>

        <p class="desc">
          A fast local timesheet companion: jot work against a fuzzy-searchable
          timecode, review it in a weekly grid that mirrors Replicon's shape, and
          copy it straight across.
        </p>

        <p class="flavour">
          Named after Bletchley Park, where messy signals were turned into something
          readable — same idea, smaller stakes.
        </p>

        <div class="meta">
          <span>Built with Tauri, Vue &amp; SQLite</span>
          <span class="dim">© Callum · Enigma Interactive</span>
        </div>

        <div class="update">
          <button
            v-if="updateState.available"
            class="primary"
            :disabled="updateState.downloading"
            @click="installUpdate"
          >
            {{ updateState.downloading ? `Updating ${updateState.progress}%` : `Update to ${updateState.version}` }}
          </button>
          <template v-else>
            <button @click="check" :disabled="checking">
              {{ checking ? "Checking…" : "Check for updates" }}
            </button>
            <span v-if="checked && !checking" class="dim">You're on the latest version.</span>
          </template>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.about .body {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.hero {
  display: flex;
  align-items: center;
  gap: 12px;
}
.hero .logo {
  font-size: 34px;
}
.title {
  font-size: 20px;
  font-weight: 700;
  letter-spacing: -0.01em;
}
.ver {
  color: var(--text-faint);
  font-size: 12px;
}
.tagline {
  font-weight: 600;
  margin: 4px 0 0;
}
.desc,
.flavour {
  margin: 0;
  color: var(--text-dim);
  font-size: 13px;
  line-height: 1.5;
}
.flavour {
  color: var(--text-faint);
  font-style: italic;
}
.meta {
  display: flex;
  flex-direction: column;
  gap: 2px;
  font-size: 12px;
  color: var(--text-dim);
  padding-top: 8px;
  border-top: 1px solid var(--border);
}
.dim {
  color: var(--text-faint);
}
.update {
  display: flex;
  align-items: center;
  gap: 10px;
  padding-top: 4px;
}
</style>
