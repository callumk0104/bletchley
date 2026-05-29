import { reactive } from "vue";
import * as api from "../api/index.js";
import { isoDate, startOfWeek, addDays } from "../lib/dates.js";

// Small shared store. The app is tiny, so a single reactive object that
// components read from and refresh is simpler than prop-drilling or Pinia.
export const store = reactive({
  timecodes: [], // all, including retired (needed to resolve old entries)
  activeTimecodes: [], // for the typeahead
  recents: [], // last-used, for the empty-state shortcut
  unresolved: [], // entries with no timecode yet
  byId: {}, // id -> timecode
  lastUsedId: null, // sticky default for capture
  dataVersion: 0, // bump to tell the grid to reload

  settings: {}, // raw key/value settings from the DB
  todayMinutes: 0,
  weekMinutes: 0,

  get dailyTargetMinutes() {
    const n = parseInt(this.settings.daily_target_minutes, 10);
    return Number.isFinite(n) && n > 0 ? n : 450;
  },

  timecodeLabel(id) {
    if (id == null) return null;
    return this.byId[id]?.label ?? `#${id}`;
  },

  async loadTimecodes() {
    const all = await api.listTimecodes(false);
    this.timecodes = all;
    this.activeTimecodes = all.filter((t) => t.active);
    this.byId = Object.fromEntries(all.map((t) => [t.id, t]));
  },

  async loadRecents() {
    this.recents = await api.recentTimecodes(5);
  },

  async loadUnresolved() {
    this.unresolved = await api.unresolvedEntries();
  },

  async loadSettings() {
    this.settings = await api.getSettings();
    this.applyTheme();
  },

  async saveSetting(key, value) {
    await api.setSetting(key, String(value));
    this.settings = { ...this.settings, [key]: String(value) };
    if (key === "theme") this.applyTheme();
  },

  applyTheme() {
    const t = this.settings.theme || "auto";
    if (t === "auto") document.documentElement.removeAttribute("data-theme");
    else document.documentElement.setAttribute("data-theme", t);
  },

  async loadTotals() {
    const today = isoDate(new Date());
    const monday = startOfWeek(new Date());
    const res = await api.totals(today, isoDate(monday), isoDate(addDays(monday, 4)));
    this.todayMinutes = res.today_minutes;
    this.weekMinutes = res.week_minutes;
    try {
      const h = (this.todayMinutes / 60).toFixed(1);
      const t = (this.dailyTargetMinutes / 60).toFixed(1);
      await api.updateTray(`Bletchley  ·  Today ${h}h / ${t}h`);
    } catch (e) {
      /* tray not available (e.g. web build) */
    }
  },

  async refreshAll() {
    await Promise.all([
      this.loadSettings(),
      this.loadTimecodes(),
      this.loadRecents(),
      this.loadUnresolved(),
      this.loadTotals(),
    ]);
  },

  // Call after any mutation so dependent views refetch.
  bump() {
    this.dataVersion += 1;
    this.loadTotals(); // keep the totals strip live
  },
});
