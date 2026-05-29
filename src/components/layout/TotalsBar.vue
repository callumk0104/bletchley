<script setup>
import { computed } from "vue";
import { store } from "../../store/index.js";

const fmt = (m) => (m / 60).toFixed(m % 60 === 0 ? 0 : 1);

const dayTarget = computed(() => store.dailyTargetMinutes);
const weekTarget = computed(() => store.dailyTargetMinutes * 5);

const todayClass = computed(() =>
  store.todayMinutes >= dayTarget.value ? "ok" : store.todayMinutes > 0 ? "under" : "empty"
);
const weekClass = computed(() =>
  store.weekMinutes >= weekTarget.value ? "ok" : store.weekMinutes > 0 ? "under" : "empty"
);
</script>

<template>
  <div class="totals">
    <div class="t">
      <span class="lbl">Today</span>
      <span class="val mono" :class="todayClass">{{ fmt(store.todayMinutes) }}</span>
      <span class="tgt mono">/ {{ fmt(dayTarget) }}h</span>
    </div>
    <div class="sep"></div>
    <div class="t">
      <span class="lbl">Week</span>
      <span class="val mono" :class="weekClass">{{ fmt(store.weekMinutes) }}</span>
      <span class="tgt mono">/ {{ fmt(weekTarget) }}h</span>
    </div>
  </div>
</template>

<style scoped>
.totals {
  display: flex;
  align-items: center;
  gap: 12px;
}
.t {
  display: flex;
  align-items: baseline;
  gap: 5px;
}
.lbl {
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: var(--text-faint);
}
.val {
  font-weight: 700;
  font-size: 15px;
}
.val.ok {
  color: var(--ok);
}
.val.under {
  color: var(--warn);
}
.val.empty {
  color: var(--text-faint);
}
.tgt {
  color: var(--text-dim);
  font-size: 12px;
}
.sep {
  width: 1px;
  height: 22px;
  background: var(--border);
}
</style>
