<script setup>
import { ref, computed, watch, onMounted } from "vue";
import { store } from "../../store/index.js";
import * as api from "../../api/index.js";
import { formatDuration, formatHours, parseDuration } from "../../lib/duration.js";
import {
  startOfWeek,
  weekDays,
  weekRangeLabel,
  addDays,
  isoDate,
} from "../../lib/dates.js";
import EntryEditor from "./EntryEditor.vue";

const emit = defineEmits(["open-tray"]);

const weekStart = ref(startOfWeek(new Date()));
const entries = ref([]);
const expanded = ref(null); // { timecodeId, iso }

const days = computed(() => weekDays(weekStart.value));
const dayIsos = computed(() => days.value.map((d) => d.iso));

async function load() {
  const start = isoDate(weekStart.value);
  const end = isoDate(addDays(weekStart.value, 4));
  entries.value = await api.entriesForWeek(start, end);
}
onMounted(load);
watch([weekStart, () => store.dataVersion], load);

// Build rows: one per timecode that has at least one (resolved) entry this week.
const rows = computed(() => {
  const map = new Map();
  for (const e of entries.value) {
    if (e.timecode_id == null) continue; // unresolved -> tray, not the grid
    if (!map.has(e.timecode_id)) {
      map.set(e.timecode_id, { cells: {}, total: 0 });
    }
    const row = map.get(e.timecode_id);
    if (!row.cells[e.date]) row.cells[e.date] = { minutes: 0, entries: [] };
    row.cells[e.date].minutes += e.duration_minutes;
    row.cells[e.date].entries.push(e);
    row.total += e.duration_minutes;
  }
  return [...map.entries()]
    .map(([id, data]) => ({
      timecode: store.byId[id] || { id, label: `#${id}`, active: true },
      ...data,
    }))
    .sort((a, b) => a.timecode.label.localeCompare(b.timecode.label));
});

const columnTotals = computed(() => {
  const totals = {};
  for (const iso of dayIsos.value) totals[iso] = 0;
  for (const r of rows.value) {
    for (const iso of dayIsos.value) {
      totals[iso] += r.cells[iso]?.minutes || 0;
    }
  }
  return totals;
});
const grandTotal = computed(() =>
  Object.values(columnTotals.value).reduce((a, b) => a + b, 0)
);

const unresolvedCount = computed(() => store.unresolved.length);

function cellEntries() {
  if (!expanded.value) return [];
  const row = rows.value.find((r) => r.timecode.id === expanded.value.timecodeId);
  return row?.cells[expanded.value.iso]?.entries || [];
}
const detailEntries = computed(cellEntries);
const detailLabel = computed(() => {
  if (!expanded.value) return "";
  return store.timecodeLabel(expanded.value.timecodeId);
});

function toggleCell(timecodeId, iso, minutes) {
  if (!minutes) return;
  if (
    expanded.value &&
    expanded.value.timecodeId === timecodeId &&
    expanded.value.iso === iso
  ) {
    expanded.value = null;
  } else {
    expanded.value = { timecodeId, iso };
  }
}

function dayOk(iso) {
  const m = columnTotals.value[iso];
  return m >= store.dailyTargetMinutes;
}
function dayEmpty(iso) {
  return (columnTotals.value[iso] || 0) === 0;
}

function prevWeek() {
  weekStart.value = addDays(weekStart.value, -7);
  expanded.value = null;
}
function nextWeek() {
  weekStart.value = addDays(weekStart.value, 7);
  expanded.value = null;
}
function thisWeek() {
  weekStart.value = startOfWeek(new Date());
  expanded.value = null;
}

// ---- entry editing within the expanded cell ----
// The per-entry edit/delete UI lives in EntryEditor; we just reload after a
// change and close the panel if the cell emptied out (e.g. entry moved away).
async function onEntryChanged() {
  await load();
  if (!detailEntries.value.length) expanded.value = null;
}

// ---- copy for Replicon ----
const copied = ref(false);
async function copyTable() {
  const header = ["Timecode", ...days.value.map((d) => d.label), "Total"].join("\t");
  const lines = rows.value.map((r) => {
    const cols = dayIsos.value.map((iso) =>
      r.cells[iso]?.minutes ? formatHours(r.cells[iso].minutes) : ""
    );
    return [r.timecode.label, ...cols, formatHours(r.total)].join("\t");
  });
  const totalsRow = [
    "Total",
    ...dayIsos.value.map((iso) => formatHours(columnTotals.value[iso])),
    formatHours(grandTotal.value),
  ].join("\t");
  const tsv = [header, ...lines, totalsRow].join("\n");
  try {
    await navigator.clipboard.writeText(tsv);
    copied.value = true;
    setTimeout(() => (copied.value = false), 1600);
  } catch {
    /* clipboard unavailable */
  }
}
</script>

<template>
  <div class="grid-wrap">
    <div class="toolbar">
      <div class="nav">
        <button class="ghost" @click="prevWeek">‹</button>
        <button class="ghost week-label" @click="thisWeek">
          {{ weekRangeLabel(weekStart) }}
        </button>
        <button class="ghost" @click="nextWeek">›</button>
      </div>
      <div class="tools">
        <button
          v-if="unresolvedCount"
          class="needs"
          @click="emit('open-tray')"
        >
          ⚠ {{ unresolvedCount }} need a timecode
        </button>
        <button class="ghost" @click="copyTable">
          {{ copied ? "✓ Copied" : "Copy for Replicon" }}
        </button>
      </div>
    </div>

    <div class="table-scroll">
      <table>
        <thead>
          <tr>
            <th class="code-col">Timecode</th>
            <th
              v-for="d in days"
              :key="d.iso"
              class="day-col"
              :class="{ empty: dayEmpty(d.iso) }"
            >
              <div class="dow">{{ d.label }}</div>
              <div class="dnum">{{ d.monthLabel }} {{ d.dayNum }}</div>
            </th>
            <th class="total-col">Total</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="r in rows" :key="r.timecode.id">
            <td class="code-col">
              <span :class="{ retired: !r.timecode.active }">{{ r.timecode.label }}</span>
              <span v-if="!r.timecode.active" class="retired-tag">retired</span>
            </td>
            <td
              v-for="iso in dayIsos"
              :key="iso"
              class="cell"
              :class="{
                has: r.cells[iso]?.minutes,
                sel: expanded && expanded.timecodeId === r.timecode.id && expanded.iso === iso,
              }"
              @click="toggleCell(r.timecode.id, iso, r.cells[iso]?.minutes)"
            >
              <span v-if="r.cells[iso]?.minutes" class="mono">
                {{ formatHours(r.cells[iso].minutes) }}
              </span>
            </td>
            <td class="total-col mono">{{ formatHours(r.total) }}</td>
          </tr>
          <tr v-if="!rows.length" class="empty-row">
            <td :colspan="days.length + 2">
              No entries this week. Capture some work and it’ll show up here.
            </td>
          </tr>
        </tbody>
        <tfoot>
          <tr>
            <td class="code-col">Daily total</td>
            <td
              v-for="iso in dayIsos"
              :key="iso"
              class="mono day-total"
              :class="{ ok: dayOk(iso), under: !dayOk(iso) && !dayEmpty(iso), empty: dayEmpty(iso) }"
            >
              {{ formatHours(columnTotals[iso]) }}
            </td>
            <td class="total-col grand mono">{{ formatHours(grandTotal) }}</td>
          </tr>
        </tfoot>
      </table>
    </div>

    <p class="legend">
      <span class="dot ok"></span> hit 7.5h
      <span class="dot under"></span> under
      <span class="dot empty"></span> empty &nbsp;·&nbsp; click a cell to see the
      underlying entries
    </p>

    <!-- Expanded cell: the rollup's underlying entries -->
    <div v-if="expanded && detailEntries.length" class="detail">
      <div class="detail-head">
        <strong>{{ detailLabel }}</strong>
        <span class="detail-date">{{ expanded.iso }}</span>
        <button class="ghost small" @click="expanded = null">close</button>
      </div>
      <EntryEditor
        v-for="e in detailEntries"
        :key="e.id"
        :entry="e"
        @changed="onEntryChanged"
      />
    </div>
  </div>
</template>

<style scoped>
.grid-wrap {
  max-width: 920px;
  margin: 0 auto;
}
.toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 14px;
}
.nav {
  display: flex;
  align-items: center;
  gap: 4px;
}
.week-label {
  font-weight: 600;
  min-width: 150px;
}
.tools {
  display: flex;
  gap: 8px;
}
.needs {
  background: var(--warn-soft);
  color: var(--warn);
  border-color: transparent;
  font-weight: 600;
}
.table-scroll {
  overflow-x: auto;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface);
}
table {
  border-collapse: collapse;
  width: 100%;
  font-size: 13px;
}
th,
td {
  padding: 9px 10px;
  text-align: center;
  border-bottom: 1px solid var(--border);
}
thead th {
  position: sticky;
  top: 0;
  background: var(--surface-2);
  font-weight: 600;
  z-index: 1;
}
.code-col {
  text-align: left;
  min-width: 230px;
  white-space: nowrap;
}
.dow {
  font-size: 12px;
}
.dnum {
  font-size: 11px;
  color: var(--text-dim);
  font-weight: 400;
}
.day-col.empty .dow {
  color: var(--text-faint);
}
.cell {
  cursor: default;
  min-width: 64px;
}
.cell.has {
  cursor: pointer;
}
.cell.has:hover {
  background: var(--accent-soft);
}
.cell.sel {
  background: var(--accent-soft);
  box-shadow: inset 0 0 0 2px var(--accent);
}
.total-col {
  font-weight: 600;
  background: var(--surface-2);
}
.retired {
  color: var(--text-dim);
  text-decoration: line-through;
}
.retired-tag {
  font-size: 10px;
  color: var(--text-faint);
  margin-left: 6px;
  text-transform: uppercase;
}
tfoot td {
  font-weight: 600;
  border-top: 2px solid var(--border-strong);
  border-bottom: none;
  background: var(--surface-2);
}
.day-total.ok {
  color: var(--ok);
}
.day-total.under {
  color: var(--warn);
}
.day-total.empty {
  color: var(--text-faint);
}
.grand {
  color: var(--accent);
}
.empty-row td {
  color: var(--text-faint);
  padding: 26px;
  font-style: italic;
}
.legend {
  display: flex;
  align-items: center;
  gap: 7px;
  color: var(--text-dim);
  font-size: 12px;
  margin: 10px 2px;
}
.dot {
  width: 9px;
  height: 9px;
  border-radius: 50%;
  display: inline-block;
}
.dot.ok {
  background: var(--ok);
}
.dot.under {
  background: var(--warn);
}
.dot.empty {
  background: var(--border-strong);
}
.detail {
  margin-top: 14px;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: 14px;
}
.detail-head {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 10px;
}
.detail-date {
  color: var(--text-dim);
  font-variant-numeric: tabular-nums;
}
.detail-head button {
  margin-left: auto;
}
.small {
  padding: 4px 9px;
  font-size: 12px;
}
:deep(.erow) {
  display: flex;
  gap: 8px;
  align-items: center;
  padding: 6px 0;
}
:deep(.erow .edur) {
  width: 90px;
  text-align: center;
}
:deep(.erow .edesc) {
  flex: 1;
}
</style>
