<script setup>
import changelogRaw from "../../../CHANGELOG.md?raw";
import { parseChangelog } from "../../lib/changelog.js";

const emit = defineEmits(["close"]);
const versions = parseChangelog(changelogRaw);
</script>

<template>
  <div class="modal-backdrop" @click.self="emit('close')">
    <div class="modal">
      <header>
        <h2>What's new</h2>
        <button class="ghost" @click="emit('close')">Done</button>
      </header>
      <div class="body">
        <section v-for="v in versions" :key="v.version" class="ver">
          <div class="ver-head">
            <span class="vnum">{{ v.version === "Unreleased" ? "Unreleased" : "v" + v.version }}</span>
            <span v-if="v.date" class="vdate">{{ v.date }}</span>
          </div>
          <ul class="notes">
            <template v-for="(t, i) in v.tokens" :key="i">
              <h4 v-if="t.type === 'h'">{{ t.text }}</h4>
              <li v-else-if="t.type === 'li'">{{ t.text }}</li>
              <p v-else-if="t.type === 'p'">{{ t.text }}</p>
            </template>
          </ul>
        </section>
      </div>
    </div>
  </div>
</template>

<style scoped>
.ver {
  padding: 0 0 16px;
  border-bottom: 1px solid var(--border);
  margin-bottom: 16px;
}
.ver:last-child {
  border-bottom: none;
  margin-bottom: 0;
}
.ver-head {
  display: flex;
  align-items: baseline;
  gap: 10px;
  margin-bottom: 8px;
}
.vnum {
  font-weight: 700;
  font-size: 15px;
}
.vdate {
  color: var(--text-faint);
  font-size: 12px;
  font-variant-numeric: tabular-nums;
}
.notes {
  list-style: none;
  padding: 0;
  margin: 0;
}
.notes h4 {
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--text-dim);
  margin: 10px 0 4px;
}
.notes li {
  position: relative;
  padding-left: 16px;
  margin: 4px 0;
  font-size: 13px;
  line-height: 1.45;
}
.notes li::before {
  content: "•";
  position: absolute;
  left: 3px;
  color: var(--accent);
}
.notes p {
  font-size: 13px;
  color: var(--text-dim);
  margin: 4px 0;
}
</style>
