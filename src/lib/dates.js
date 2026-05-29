// All week logic is Monday-anchored, Mon-Fri (per the chosen config).
// Dates are handled as local-time ISO yyyy-mm-dd strings to avoid timezone
// drift when storing/querying in SQLite.

export function isoDate(d) {
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  return `${y}-${m}-${day}`;
}

export function fromIso(s) {
  const [y, m, d] = s.split("-").map(Number);
  return new Date(y, m - 1, d);
}

export function addDays(d, n) {
  const out = new Date(d);
  out.setDate(out.getDate() + n);
  return out;
}

// Monday of the week containing `d`.
export function startOfWeek(d) {
  const date = new Date(d);
  const day = date.getDay(); // 0=Sun .. 6=Sat
  const diff = day === 0 ? -6 : 1 - day; // shift back to Monday
  return addDays(date, diff);
}

// The five weekday columns (Mon-Fri) starting from a Monday.
export function weekDays(monday) {
  return Array.from({ length: 5 }, (_, i) => {
    const d = addDays(monday, i);
    return {
      iso: isoDate(d),
      label: d.toLocaleDateString(undefined, { weekday: "short" }),
      dayNum: d.getDate(),
      monthLabel: d.toLocaleDateString(undefined, { month: "short" }),
    };
  });
}

export function weekRangeLabel(monday) {
  const friday = addDays(monday, 4);
  const opts = { month: "short", day: "numeric" };
  return `${monday.toLocaleDateString(undefined, opts)} – ${friday.toLocaleDateString(
    undefined,
    opts
  )}`;
}

export const TARGET_DAILY_MINUTES = 450; // 7.5h
