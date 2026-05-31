<script setup>
import { ref, computed, onMounted, onUnmounted } from "vue";
import { store } from "../../store/index.js";
import * as api from "../../api/index.js";
import { parseDuration, formatDuration } from "../../lib/duration.js";
import { isoDate, startOfWeek, weekDays, addDays, weekRangeLabel } from "../../lib/dates.js";
import TimecodePicker from "../common/TimecodePicker.vue";

const emit = defineEmits(["open-codes"]);

const selected = ref(null); // timecode object or null (blank = unresolved)

// Week navigation: the shown week (Monday) + which weekday is selected (0..4),
// so you can log to past/other weeks, not just today.
const weekStart = ref(startOfWeek(new Date()));
const _todayIso = isoDate(new Date());
const _initIdx = weekDays(weekStart.value).findIndex((d) => d.iso === _todayIso);
const dayIdx = ref(_initIdx >= 0 ? _initIdx : 4); // weekends default to Friday
const date = computed(() => days.value[dayIdx.value]?.iso ?? _todayIso);
const isThisWeek = computed(
  () => isoDate(weekStart.value) === isoDate(startOfWeek(new Date()))
);
function prevWeek() {
  weekStart.value = addDays(weekStart.value, -7);
}
function nextWeek() {
  weekStart.value = addDays(weekStart.value, 7);
}
function thisWeek() {
  const wk = startOfWeek(new Date());
  weekStart.value = wk;
  const i = weekDays(wk).findIndex((d) => d.iso === isoDate(new Date()));
  dayIdx.value = i >= 0 ? i : 4;
}
const description = ref("");
const durationText = ref("");
const error = ref("");
const justAdded = ref([]); // session log for reassurance

const picker = ref(null);
const descEl = ref(null);
const durEl = ref(null);

// Day chips for the selected week so you can log to any weekday quickly.
const days = computed(() => weekDays(weekStart.value));

const parsedMinutes = computed(() => parseDuration(durationText.value));
const durationPreview = computed(() =>
  parsedMinutes.value ? formatDuration(parsedMinutes.value) : ""
);
const canAdd = computed(() => !!parsedMinutes.value);

onMounted(() => {
  // Sticky default to the last-used timecode.
  if (store.lastUsedId != null && store.byId[store.lastUsedId]) {
    selected.value = store.byId[store.lastUsedId];
    nextFocus(descEl); // a chip is shown, not the picker — go straight to typing
  } else if (store.activeTimecodes.length) {
    picker.value?.focus?.();
  } else {
    nextFocus(descEl); // no codes yet; let them capture to the tray
  }
});

function onSelectTimecode(tc) {
  selected.value = tc;
  // Move straight to the description for an uninterrupted flow.
  nextFocus(descEl);
}

function clearTimecode() {
  selected.value = null;
  picker.value?.focus?.();
}

function nextFocus(elRef) {
  requestAnimationFrame(() => elRef.value?.focus?.());
}

// ---- optional timer ----
const timerStart = ref(null);
const now = ref(Date.now());
let ticker = null;
const timerRunning = computed(() => timerStart.value != null);
const elapsedMin = computed(() =>
  timerRunning.value ? Math.max(0, Math.round((now.value - timerStart.value) / 60000)) : 0
);
function toggleTimer() {
  if (timerRunning.value) {
    const mins = Math.max(1, elapsedMin.value);
    durationText.value = `${mins}m`;
    timerStart.value = null;
  } else {
    timerStart.value = Date.now();
  }
}
onMounted(() => {
  ticker = setInterval(() => (now.value = Date.now()), 1000);
});
onUnmounted(() => clearInterval(ticker));

async function add() {
  error.value = "";
  const mins = parsedMinutes.value;
  if (!mins) {
    error.value = "Couldn't read that duration. Try 2h, 90m, or 1h30m.";
    nextFocus(durEl);
    return;
  }
  try {
    const created = await api.addEntry(
      selected.value ? selected.value.id : null,
      date.value,
      mins,
      description.value.trim()
    );
    justAdded.value.unshift({
      id: created.id,
      label: selected.value ? selected.value.label : "⚠ needs timecode",
      mins,
      description: description.value.trim(),
      date: date.value,
      unresolved: !selected.value,
    });
    justAdded.value = justAdded.value.slice(0, 6);

    // Sticky timecode; clear the rest for the next rapid entry.
    if (selected.value) store.lastUsedId = selected.value.id;
    description.value = "";
    durationText.value = "";
    timerStart.value = null;

    await store.loadRecents();
    await store.loadUnresolved();
    store.bump();
    nextFocus(descEl);
  } catch (e) {
    error.value = String(e);
  }
}

function onDescEnter() {
  nextFocus(durEl);
}

// Undo a just-added entry (deletes it). Handy for the immediate "oops".
async function undo(entry, i) {
  try {
    await api.deleteEntry(entry.id);
  } catch (e) {
    /* already gone */
  }
  justAdded.value.splice(i, 1);
  await store.loadRecents();
  await store.loadUnresolved();
  store.bump();
}
</script>

<template>
  <div class="capture">
    <div class="card">
      <!-- Week navigation + day selector -->
      <div class="week-nav">
        <button class="ghost wk" title="Previous week" @click="prevWeek">‹</button>
        <button
          class="ghost wk-label"
          :class="{ past: !isThisWeek }"
          :title="isThisWeek ? 'Current week' : 'Back to this week'"
          @click="thisWeek"
        >
          {{ isThisWeek ? "This week" : weekRangeLabel(weekStart) }}
        </button>
        <button class="ghost wk" title="Next week" @click="nextWeek">›</button>
      </div>
      <div class="days">
        <button
          v-for="(d, i) in days"
          :key="d.iso"
          class="day"
          :class="{ on: dayIdx === i }"
          @click="dayIdx = i"
        >
          <span class="dow">{{ d.label }}</span>
          <span class="dnum">{{ d.dayNum }}</span>
        </button>
      </div>
      <p v-if="!isThisWeek" class="offweek">
        ⚠ Logging to {{ weekRangeLabel(weekStart) }} — not the current week.
      </p>

      <!-- Timecode -->
      <label class="field-label">Timecode</label>
      <div v-if="selected" class="chip-row">
        <span class="chip">{{ selected.label }}</span>
        <button class="ghost small" @click="clearTimecode">change</button>
      </div>
      <div v-else-if="!store.activeTimecodes.length" class="tc-empty">
        <span>No timecodes yet — connect Replicon and sync to pull your bookable codes.</span>
        <button class="ghost small" @click="emit('open-codes')">Set up timecodes</button>
      </div>
      <TimecodePicker
        v-else
        ref="picker"
        :recents="store.recents"
        @select="onSelectTimecode"
      />

      <!-- Description + duration -->
      <div class="entry-row">
        <div class="grow">
          <label class="field-label">What did you do?</label>
          <input
            ref="descEl"
            v-model="description"
            placeholder="e.g. fixed checkout bug"
            spellcheck="true"
            @keydown.enter.prevent="onDescEnter"
          />
        </div>
        <div class="dur">
          <label class="field-label">Duration</label>
          <div class="dur-input">
            <input
              ref="durEl"
              v-model="durationText"
              class="mono"
              :placeholder="timerRunning ? formatDuration(elapsedMin) + ' (timer)' : '2 = 2h'"
              @keydown.enter.prevent="add"
            />
            <button
              class="ghost timer"
              :class="{ running: timerRunning }"
              :title="timerRunning ? 'Stop timer & fill duration' : 'Start timer'"
              @click="toggleTimer"
            >
              {{ timerRunning ? "■ " + formatDuration(elapsedMin) : "▶ timer" }}
            </button>
          </div>
        </div>
        <button class="primary add" :disabled="!canAdd" @click="add">Add</button>
      </div>

      <div class="hint-row">
        <span v-if="durationPreview" class="ok-pill mono">= {{ durationPreview }}</span>
        <span v-if="!selected" class="warn-pill">
          No timecode — will go to the “needs timecode” tray
        </span>
        <span v-if="error" class="err">{{ error }}</span>
      </div>
    </div>

    <!-- Session log -->
    <div v-if="justAdded.length" class="log">
      <div class="log-title">Added this session</div>
      <div v-for="(e, i) in justAdded" :key="i" class="log-row">
        <span class="mono dur-tag" :class="{ warn: e.unresolved }">{{ formatDuration(e.mins) }}</span>
        <span class="log-label" :class="{ warn: e.unresolved }">{{ e.label }}</span>
        <span class="log-desc">{{ e.description || "—" }}</span>
        <span class="log-date">{{ e.date }}</span>
        <button class="ghost undo" title="Delete this entry" @click="undo(e, i)">undo</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.capture {
  max-width: 760px;
  margin: 0 auto;
}
.card {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 18px;
}
.week-nav {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 10px;
}
.wk {
  padding: 4px 10px;
  font-size: 14px;
  line-height: 1;
}
.wk-label {
  font-weight: 600;
  font-size: 12px;
  min-width: 110px;
}
.wk-label.past {
  background: var(--warn-soft);
  color: var(--warn);
  border-color: transparent;
}
.days {
  display: flex;
  gap: 6px;
  margin-bottom: 16px;
}
.offweek {
  color: var(--warn);
  font-size: 12px;
  margin: -8px 0 14px 2px;
}
.day {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1px;
  padding: 6px 14px;
  border-radius: var(--radius-sm);
}
.day .dow {
  font-size: 11px;
  color: var(--text-dim);
}
.day .dnum {
  font-size: 15px;
  font-weight: 600;
}
.day.on {
  background: var(--accent);
  border-color: var(--accent);
}
.day.on .dow,
.day.on .dnum {
  color: var(--accent-text);
}
.field-label {
  display: block;
  font-size: 12px;
  color: var(--text-dim);
  margin: 0 0 5px 2px;
}
.chip-row {
  display: flex;
  align-items: center;
  gap: 10px;
}
.chip {
  background: var(--accent-soft);
  color: var(--accent);
  font-weight: 600;
  padding: 9px 12px;
  border-radius: var(--radius-sm);
  flex: 1;
}
.tc-empty {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 11px 12px;
  border: 1px dashed var(--border-strong);
  border-radius: var(--radius-sm);
  color: var(--text-dim);
  font-size: 13px;
}
.tc-empty button {
  white-space: nowrap;
}
.small {
  padding: 4px 9px;
  font-size: 12px;
}
.entry-row {
  display: flex;
  gap: 10px;
  align-items: flex-end;
  margin-top: 14px;
}
.grow {
  flex: 1;
}
.dur {
  width: 220px;
}
.dur-input {
  display: flex;
  gap: 6px;
}
.dur-input input {
  width: 70px;
  text-align: center;
}
.timer {
  white-space: nowrap;
  font-size: 12px;
  flex: 1;
}
.timer.running {
  background: var(--warn-soft);
  color: var(--warn);
}
.add {
  height: 38px;
  padding: 0 20px;
}
.hint-row {
  display: flex;
  gap: 10px;
  align-items: center;
  margin-top: 12px;
  min-height: 22px;
  flex-wrap: wrap;
}
.ok-pill {
  background: var(--ok-soft);
  color: var(--ok);
  font-weight: 600;
  padding: 3px 9px;
  border-radius: 999px;
  font-size: 13px;
}
.warn-pill {
  color: var(--warn);
  font-size: 12px;
}
.err {
  color: var(--danger);
  font-size: 13px;
}
.log {
  margin-top: 22px;
}
.log-title {
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--text-faint);
  margin-bottom: 8px;
}
.log-row {
  display: grid;
  grid-template-columns: 64px 1fr 1fr 90px auto;
  gap: 10px;
  align-items: center;
  padding: 7px 8px;
  border-bottom: 1px solid var(--border);
  font-size: 13px;
}
.dur-tag {
  color: var(--text-dim);
  font-weight: 600;
}
.dur-tag.warn,
.log-label.warn {
  color: var(--warn);
}
.log-label {
  font-weight: 500;
}
.log-desc {
  color: var(--text-dim);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.log-date {
  color: var(--text-faint);
  text-align: right;
  font-variant-numeric: tabular-nums;
}
.undo {
  padding: 3px 9px;
  font-size: 12px;
  color: var(--text-dim);
}
</style>
