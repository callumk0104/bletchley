<script setup>
import { ref, computed, nextTick, onMounted, onUnmounted } from "vue";
import { store } from "../../store/index.js";
import { fuzzySearch } from "../../lib/fuzzy.js";

const props = defineProps({
  recents: { type: Array, default: () => [] },
  placeholder: { type: String, default: "Search timecode…  (try \"chevqa\")" },
  autofocus: { type: Boolean, default: false },
});
const emit = defineEmits(["select"]);

const query = ref("");
const open = ref(false);
const highlight = ref(0);
const inputEl = ref(null);
// The dropdown is position:fixed so it can't be clipped by a scrolling modal.
const dropStyle = ref({});

const matches = computed(() => {
  const list = fuzzySearch(query.value, store.activeTimecodes, (t) => t.label);
  return list.slice(0, 8);
});

const showRecents = computed(
  () => open.value && !query.value.trim() && props.recents.length > 0
);

const navList = computed(() =>
  showRecents.value ? props.recents : matches.value
);

function focus() {
  inputEl.value?.focus();
}
defineExpose({ focus });

function positionDropdown() {
  const el = inputEl.value;
  if (!el) return;
  const r = el.getBoundingClientRect();
  const spaceBelow = window.innerHeight - r.bottom;
  const MAX = 280;
  const style = {
    position: "fixed",
    left: Math.round(r.left) + "px",
    width: Math.round(r.width) + "px",
    "z-index": 1000,
  };
  // Flip upward if there isn't room below and there's more room above.
  if (spaceBelow < 220 && r.top > spaceBelow) {
    style.bottom = Math.round(window.innerHeight - r.top + 4) + "px";
    style["max-height"] = Math.min(MAX, Math.round(r.top - 12)) + "px";
  } else {
    style.top = Math.round(r.bottom + 4) + "px";
    style["max-height"] = Math.min(MAX, Math.round(spaceBelow - 12)) + "px";
  }
  dropStyle.value = style;
}

function reposition() {
  if (open.value) positionDropdown();
}
onMounted(() => {
  window.addEventListener("scroll", reposition, true);
  window.addEventListener("resize", reposition);
});
onUnmounted(() => {
  window.removeEventListener("scroll", reposition, true);
  window.removeEventListener("resize", reposition);
});

function onFocus() {
  open.value = true;
  highlight.value = 0;
  nextTick(positionDropdown);
}

function choose(tc) {
  emit("select", tc);
  query.value = "";
  open.value = false;
}

function move(delta) {
  if (!open.value) {
    open.value = true;
    nextTick(positionDropdown);
  }
  const n = navList.value.length;
  if (!n) return;
  highlight.value = (highlight.value + delta + n) % n;
}

function onEnter() {
  const item = navList.value[highlight.value];
  if (item) choose(item);
}

function onInput() {
  nextTick(positionDropdown);
}

function onKeydown(e) {
  if (showRecents.value && /^[1-9]$/.test(e.key)) {
    const idx = Number(e.key) - 1;
    if (idx < props.recents.length) {
      e.preventDefault();
      choose(props.recents[idx]);
    }
  }
}

if (props.autofocus) nextTick(focus);
</script>

<template>
  <div class="picker" @keydown.down.prevent="move(1)" @keydown.up.prevent="move(-1)"
       @keydown.enter.prevent="onEnter" @keydown.esc="open = false">
    <input
      ref="inputEl"
      v-model="query"
      :placeholder="placeholder"
      @focus="onFocus"
      @input="onInput"
      @keydown="onKeydown"
      @blur="open = false"
      spellcheck="false"
      autocomplete="off"
    />
    <div v-if="open" class="dropdown" :style="dropStyle" @mousedown.prevent>
      <template v-if="showRecents">
        <div class="section-label">Recent</div>
        <button
          v-for="(tc, i) in recents"
          :key="'r' + tc.id"
          class="opt"
          :class="{ hot: highlight === i }"
          @click="choose(tc)"
          @mousemove="highlight = i"
        >
          <span class="num">{{ i + 1 }}</span>
          <span class="label">{{ tc.label }}</span>
        </button>
      </template>
      <template v-else>
        <button
          v-for="(tc, i) in matches"
          :key="tc.id"
          class="opt"
          :class="{ hot: highlight === i }"
          @click="choose(tc)"
          @mousemove="highlight = i"
        >
          <span class="label">{{ tc.label }}</span>
        </button>
        <div v-if="!matches.length" class="empty">No matching timecode</div>
      </template>
    </div>
  </div>
</template>

<style scoped>
.picker {
  position: relative;
}
.dropdown {
  background: var(--surface);
  border: 1px solid var(--border-strong);
  border-radius: var(--radius-sm);
  box-shadow: var(--shadow);
  padding: 4px;
  overflow-y: auto;
}
.section-label {
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--text-faint);
  padding: 6px 8px 4px;
}
.opt {
  display: flex;
  align-items: center;
  gap: 9px;
  width: 100%;
  text-align: left;
  border: none;
  background: transparent;
  border-radius: 6px;
  padding: 8px 9px;
}
.opt.hot {
  background: var(--accent-soft);
}
.opt .num {
  font-family: var(--mono);
  font-size: 11px;
  color: var(--text-faint);
  border: 1px solid var(--border);
  border-radius: 4px;
  padding: 0 5px;
}
.opt .label {
  font-size: 13px;
}
.empty {
  padding: 10px;
  color: var(--text-faint);
  font-size: 13px;
}
</style>
