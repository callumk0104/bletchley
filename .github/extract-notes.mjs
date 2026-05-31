// Pull the changelog section for a given version (e.g. "v1.0.3" or "1.0.3")
// and emit it as the GitHub release body. Run by the release workflow.
import fs from "node:fs";

const version = (process.argv[2] || "").replace(/^v/, "").trim();
const md = fs.existsSync("CHANGELOG.md") ? fs.readFileSync("CHANGELOG.md", "utf8") : "";

const lines = md.split(/\r?\n/);
const body = [];
let capturing = false;
for (const line of lines) {
  const h = line.match(/^##\s+\[?([^\]\s]+)\]?/);
  if (h) {
    if (capturing) break; // reached the next version section
    if (h[1] === version) capturing = true;
    continue;
  }
  if (capturing) body.push(line);
}

let notes = body.join("\n").trim();
if (!notes) notes = `Release ${version}.`;
notes += "\n\n---\nDownload the `-setup.exe` to install. Existing installs update in-app.";

const out = process.env.GITHUB_OUTPUT;
if (out) {
  fs.appendFileSync(out, `body<<__NOTES_EOF__\n${notes}\n__NOTES_EOF__\n`);
}
console.log(notes);
