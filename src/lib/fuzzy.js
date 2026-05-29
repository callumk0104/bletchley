// Lightweight subsequence fuzzy matcher. No dependency — the timecode list is
// only a few dozen items. Scores contiguous and word-boundary matches higher,
// so "chevqa" lands "William Smith / Chevron Kits / QA" and "willqa" lands the
// same row via the client name.

function scoreOne(query, target) {
  if (!query) return 0;
  const q = query.toLowerCase();
  const t = target.toLowerCase();

  let score = 0;
  let qi = 0;
  let prevMatch = -2;
  let streak = 0;

  for (let ti = 0; ti < t.length && qi < q.length; ti++) {
    if (t[ti] === q[qi]) {
      // Base point for the match.
      score += 1;
      // Bonus for consecutive matches (contiguous substring feel).
      if (ti === prevMatch + 1) {
        streak += 1;
        score += streak * 2;
      } else {
        streak = 0;
      }
      // Bonus for matching at a word boundary (start, after space or "/").
      const before = ti === 0 ? "" : t[ti - 1];
      if (ti === 0 || before === " " || before === "/" || before === "-") {
        score += 3;
      }
      prevMatch = ti;
      qi += 1;
    }
  }

  // Only a match if every query char was consumed.
  if (qi < q.length) return -1;

  // Prefer shorter targets when scores are otherwise close.
  score -= t.length * 0.01;
  return score;
}

// items: array of objects. keyFn returns the searchable string for an item.
// Returns the matching items sorted best-first.
export function fuzzySearch(query, items, keyFn) {
  const q = (query || "").trim();
  if (!q) return items.slice();
  const scored = [];
  for (const item of items) {
    const s = scoreOne(q, keyFn(item));
    if (s >= 0) scored.push({ item, s });
  }
  scored.sort((a, b) => b.s - a.s);
  return scored.map((x) => x.item);
}
