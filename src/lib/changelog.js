// Minimal changelog/notes helpers — no markdown lib, just the small subset we
// author: "## [version] - date" sections, "### Group" subheads, and "- " bullets.

export function tokenize(text) {
  const tokens = [];
  for (const raw of (text || "").split(/\r?\n/)) {
    const line = raw.trim();
    if (!line) continue;
    let m;
    if ((m = line.match(/^#{3,}\s+(.*)$/))) tokens.push({ type: "h", text: m[1] });
    else if ((m = line.match(/^[-*]\s+(.*)$/))) tokens.push({ type: "li", text: m[1] });
    else if (/^[-–]{3,}$/.test(line)) tokens.push({ type: "hr" });
    else tokens.push({ type: "p", text: line });
  }
  return tokens;
}

export function parseChangelog(raw) {
  const out = [];
  let cur = null;
  for (const line of (raw || "").split(/\r?\n/)) {
    const h = line.match(/^##\s+\[?([^\]\s]+)\]?\s*(?:[-–]\s*(.+))?$/);
    if (h) {
      cur = { version: h[1], date: (h[2] || "").trim(), body: [] };
      out.push(cur);
    } else if (cur) {
      cur.body.push(line);
    }
  }
  return out.map((v) => ({ version: v.version, date: v.date, tokens: tokenize(v.body.join("\n")) }));
}
