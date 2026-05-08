<script setup lang="ts">
import { computed, onBeforeUnmount, ref } from "vue";
import { ElMessage } from "element-plus";

const timestampInput = ref("");
const datetimeText = ref("");
const offsetAmount = ref(1);
const offsetUnit = ref<"second" | "minute" | "hour" | "day">("day");

const now = ref(new Date());
const timer = window.setInterval(() => {
  now.value = new Date();
}, 1000);

onBeforeUnmount(() => window.clearInterval(timer));

const currentSec = computed(() => Math.floor(now.value.getTime() / 1000));
const currentMs = computed(() => now.value.getTime());
const currentLocal = computed(() => formatDateTime(now.value));
const currentUtc = computed(() => now.value.toISOString().replace("T", " ").slice(0, 19));

const converted = computed(() => {
  const s = timestampInput.value.trim();
  if (!s) return null;
  const n = Number(s);
  if (!Number.isFinite(n)) return null;
  const ms = Math.abs(n) < 100000000000 ? n * 1000 : n;
  const d = new Date(ms);
  if (Number.isNaN(d.getTime())) return null;
  return {
    seconds: Math.floor(d.getTime() / 1000),
    milliseconds: d.getTime(),
    local: formatDateTime(d),
    utc: d.toISOString().replace("T", " ").slice(0, 19),
  };
});

function pad(n: number) {
  return String(n).padStart(2, "0");
}

function formatDateTime(d: Date) {
  return [
    d.getFullYear(),
    "-",
    pad(d.getMonth() + 1),
    "-",
    pad(d.getDate()),
    " ",
    pad(d.getHours()),
    ":",
    pad(d.getMinutes()),
    ":",
    pad(d.getSeconds()),
  ].join("");
}

function parseLocalDateTime(s: string) {
  const m = s
    .trim()
    .match(/^(\d{4})-(\d{2})-(\d{2})(?:[ T](\d{2}):(\d{2})(?::(\d{2}))?)?$/);
  if (!m) return null;
  const d = new Date(
    Number(m[1]),
    Number(m[2]) - 1,
    Number(m[3]),
    Number(m[4] ?? 0),
    Number(m[5] ?? 0),
    Number(m[6] ?? 0)
  );
  if (Number.isNaN(d.getTime())) return null;
  return d;
}

const datetimeConverted = computed(() => {
  const d = parseLocalDateTime(datetimeText.value);
  if (!d) return null;
  return {
    seconds: Math.floor(d.getTime() / 1000),
    milliseconds: d.getTime(),
    local: formatDateTime(d),
    utc: d.toISOString().replace("T", " ").slice(0, 19),
  };
});

function fillNow() {
  const d = new Date();
  timestampInput.value = String(d.getTime());
  datetimeText.value = formatDateTime(d);
}

function fillTimestamp(unit: "s" | "ms") {
  const d = new Date();
  timestampInput.value = String(unit === "s" ? Math.floor(d.getTime() / 1000) : d.getTime());
}

function addOffset(sign: 1 | -1) {
  const base = parseLocalDateTime(datetimeText.value) ?? new Date();
  const amount = offsetAmount.value * sign;
  const d = new Date(base);
  if (offsetUnit.value === "second") d.setSeconds(d.getSeconds() + amount);
  if (offsetUnit.value === "minute") d.setMinutes(d.getMinutes() + amount);
  if (offsetUnit.value === "hour") d.setHours(d.getHours() + amount);
  if (offsetUnit.value === "day") d.setDate(d.getDate() + amount);
  datetimeText.value = formatDateTime(d);
}

async function copyText(text: string | number) {
  try {
    await navigator.clipboard.writeText(String(text));
    ElMessage.success("已复制");
  } catch {
    ElMessage.error("复制失败");
  }
}
</script>

<template>
  <div class="timestamp-page">
    <h2 class="page-title">时间戳</h2>

    <div class="current-grid">
      <div class="kv">
        <span>当前秒</span>
        <strong>{{ currentSec }}</strong>
        <el-button size="small" @click="copyText(currentSec)">复制</el-button>
      </div>
      <div class="kv">
        <span>当前毫秒</span>
        <strong>{{ currentMs }}</strong>
        <el-button size="small" @click="copyText(currentMs)">复制</el-button>
      </div>
      <div class="kv">
        <span>本地时间</span>
        <strong>{{ currentLocal }}</strong>
        <el-button size="small" @click="copyText(currentLocal)">复制</el-button>
      </div>
      <div class="kv">
        <span>UTC</span>
        <strong>{{ currentUtc }}</strong>
        <el-button size="small" @click="copyText(currentUtc)">复制</el-button>
      </div>
    </div>

    <section class="section">
      <h3>时间戳转时间</h3>
      <div class="row">
        <el-input
          v-model="timestampInput"
          clearable
          placeholder="秒或毫秒时间戳"
          style="max-width: 360px"
        />
        <el-button @click="fillTimestamp('s')">当前秒</el-button>
        <el-button @click="fillTimestamp('ms')">当前毫秒</el-button>
      </div>
      <div v-if="converted" class="result-grid">
        <div>秒：<strong>{{ converted.seconds }}</strong></div>
        <div>毫秒：<strong>{{ converted.milliseconds }}</strong></div>
        <div>本地：<strong>{{ converted.local }}</strong></div>
        <div>UTC：<strong>{{ converted.utc }}</strong></div>
      </div>
      <div v-else class="empty">—</div>
    </section>

    <section class="section">
      <h3>时间转时间戳</h3>
      <div class="row">
        <el-input
          v-model="datetimeText"
          clearable
          placeholder="YYYY-MM-DD HH:mm:ss"
          style="max-width: 360px"
        />
        <el-button @click="fillNow">当前时间</el-button>
        <el-input-number v-model="offsetAmount" :min="0" :max="9999" size="default" />
        <el-select v-model="offsetUnit" style="width: 110px">
          <el-option value="second" label="秒" />
          <el-option value="minute" label="分钟" />
          <el-option value="hour" label="小时" />
          <el-option value="day" label="天" />
        </el-select>
        <el-button @click="addOffset(-1)">减</el-button>
        <el-button @click="addOffset(1)">加</el-button>
      </div>
      <div v-if="datetimeConverted" class="result-grid">
        <div>秒：<strong>{{ datetimeConverted.seconds }}</strong></div>
        <div>毫秒：<strong>{{ datetimeConverted.milliseconds }}</strong></div>
        <div>本地：<strong>{{ datetimeConverted.local }}</strong></div>
        <div>UTC：<strong>{{ datetimeConverted.utc }}</strong></div>
      </div>
      <div v-else class="empty">—</div>
    </section>
  </div>
</template>

<style scoped>
.timestamp-page {
  max-width: 100%;
}
.page-title {
  margin: 0 0 12px;
  font-size: 18px;
  font-weight: 600;
}
.current-grid,
.result-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
  gap: 10px;
}
.kv,
.result-grid > div {
  display: grid;
  grid-template-columns: 72px minmax(0, 1fr) auto;
  gap: 10px;
  align-items: center;
  padding: 8px 0;
  border-bottom: 1px solid var(--el-border-color-lighter);
}
.kv span,
.empty {
  color: var(--el-text-color-secondary);
}
.kv strong,
.result-grid strong {
  font-family: Consolas, "Courier New", monospace;
  word-break: break-all;
}
.section {
  margin-top: 24px;
}
.section h3 {
  margin: 0 0 10px;
  font-size: 15px;
  font-weight: 600;
}
.row {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  align-items: center;
  margin-bottom: 10px;
}
.empty {
  padding: 8px 0;
}
</style>
