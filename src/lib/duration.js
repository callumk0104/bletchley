// Parse the loose duration shorthand a human types. A BARE number is HOURS
// (so "2" = 2h, "1.5" = 1h30m). Minutes are entered with a unit or h:mm:
//   "2"   -> 2h          "1.5" -> 1h30m       "0.5" -> 30m
//   "0:02"-> 2m          "1:30" -> 1h30m
//   "90m" -> 90m         "30m"  -> 30m
//   "2h"  -> 2h          "1h30m" / "1h30" -> 1h30m
// Returns an integer number of minutes, or null if it can't be parsed.
export function parseDuration(input) {
  if (input == null) return null;
  const s = String(input).trim().toLowerCase();
  if (!s) return null;

  // h:mm  -> "0:02" = 2m, "1:30" = 90m
  const colon = s.match(/^(\d+):([0-5]?\d)$/);
  if (colon) {
    const mins = parseInt(colon[1], 10) * 60 + parseInt(colon[2], 10);
    return mins > 0 ? mins : null;
  }

  // Explicit units — only when an h or m is actually present.
  if (/[hm]/.test(s)) {
    const u = s.match(/^(?:(\d+(?:\.\d+)?)\s*h)?\s*(?:(\d+)\s*m?)?$/);
    if (u && (u[1] || u[2])) {
      let mins = 0;
      if (u[1]) mins += Math.round(parseFloat(u[1]) * 60);
      if (u[2]) mins += parseInt(u[2], 10);
      return mins > 0 ? mins : null;
    }
    return null;
  }

  // Bare number => HOURS (integer or decimal).
  const bare = s.match(/^(\d+(?:\.\d+)?)$/);
  if (bare) {
    const mins = Math.round(parseFloat(bare[1]) * 60);
    return mins > 0 ? mins : null;
  }

  return null;
}

// 90 -> "1h 30m", 60 -> "1h", 45 -> "45m", 0 -> "0m"
export function formatDuration(minutes) {
  const mins = Math.max(0, Math.round(minutes || 0));
  const h = Math.floor(mins / 60);
  const m = mins % 60;
  if (h && m) return `${h}h ${m}m`;
  if (h) return `${h}h`;
  return `${m}m`;
}

// 90 -> "1.50" (decimal hours, the shape Replicon wants per day)
export function formatHours(minutes) {
  return ((minutes || 0) / 60).toFixed(2);
}
