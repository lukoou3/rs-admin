<script setup lang="ts">
import CodeMirrorPane from "@/components/CodeMirrorPane.vue";
import { computed, ref } from "vue";
import { ElMessage } from "element-plus";

const jsonText = ref("{\n  \"name\": \"rs-admin\",\n  \"ok\": true\n}");
const status = ref("");

function parseJson() {
  return JSON.parse(jsonText.value);
}

function sortJsonValue(value: unknown): unknown {
  if (Array.isArray(value)) return value.map(sortJsonValue);
  if (value && typeof value === "object") {
    const obj = value as Record<string, unknown>;
    return Object.keys(obj)
      .sort((a, b) => a.localeCompare(b))
      .reduce<Record<string, unknown>>((acc, key) => {
        acc[key] = sortJsonValue(obj[key]);
        return acc;
      }, {});
  }
  return value;
}

function setJson(value: unknown, space: number) {
  jsonText.value = JSON.stringify(value, null, space);
  status.value = "JSON 有效";
}

function errMsg(e: unknown) {
  if (e instanceof Error) return e.message;
  return String(e);
}

function formatJson() {
  try {
    setJson(parseJson(), 2);
    ElMessage.success("已格式化");
  } catch (e) {
    status.value = errMsg(e);
    ElMessage.error("JSON 无效");
  }
}

function minifyJson() {
  try {
    setJson(parseJson(), 0);
    ElMessage.success("已压缩");
  } catch (e) {
    status.value = errMsg(e);
    ElMessage.error("JSON 无效");
  }
}

function sortKeys() {
  try {
    setJson(sortJsonValue(parseJson()), 2);
    ElMessage.success("已排序");
  } catch (e) {
    status.value = errMsg(e);
    ElMessage.error("JSON 无效");
  }
}

function validateJson() {
  try {
    parseJson();
    status.value = "JSON 有效";
    ElMessage.success("JSON 有效");
  } catch (e) {
    status.value = errMsg(e);
    ElMessage.error("JSON 无效");
  }
}

function escapeJsonString() {
  jsonText.value = JSON.stringify(jsonText.value);
  status.value = "";
  ElMessage.success("已转义");
}

function unescapeJsonString() {
  try {
    const v = JSON.parse(jsonText.value);
    if (typeof v !== "string") {
      ElMessage.warning("当前内容不是 JSON 字符串");
      return;
    }
    jsonText.value = v;
    status.value = "";
    ElMessage.success("已反转义");
  } catch (e) {
    status.value = errMsg(e);
    ElMessage.error("反转义失败");
  }
}

async function copyInfo() {
  try {
    await navigator.clipboard.writeText(jsonText.value);
    ElMessage.success("已复制");
  } catch {
    ElMessage.error("复制失败");
  }
}

async function pasteInfo() {
  try {
    jsonText.value = await navigator.clipboard.readText();
    status.value = "";
    ElMessage.success("已粘贴");
  } catch {
    ElMessage.error("粘贴失败");
  }
}

function clearJson() {
  jsonText.value = "";
  status.value = "";
}

const meta = computed(() => {
  const bytes = new TextEncoder().encode(jsonText.value).length;
  return `${jsonText.value.length} chars / ${bytes} bytes`;
});
</script>

<template>
  <div class="json-tool-page">
    <h2 class="page-title">JSON 工具</h2>
    <CodeMirrorPane
      v-model="jsonText"
      language="javascript"
      height="460px"
    />
    <div class="actions">
      <el-button type="primary" size="small" @click="formatJson">格式化</el-button>
      <el-button type="primary" size="small" @click="minifyJson">压缩</el-button>
      <el-button type="primary" size="small" @click="sortKeys">排序</el-button>
      <el-button size="small" @click="validateJson">校验</el-button>
      <el-button size="small" @click="escapeJsonString">转义</el-button>
      <el-button size="small" @click="unescapeJsonString">反转义</el-button>
      <el-button size="small" @click="copyInfo">复制</el-button>
      <el-button size="small" @click="pasteInfo">粘贴</el-button>
      <el-button size="small" @click="clearJson">清空</el-button>
      <span class="meta">{{ meta }}</span>
    </div>
    <pre v-if="status" class="status">{{ status }}</pre>
  </div>
</template>

<style scoped>
.json-tool-page {
  max-width: 100%;
}
.page-title {
  margin: 0 0 8px;
  font-size: 18px;
  font-weight: 600;
}
.actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  align-items: center;
  margin-top: 12px;
}
.meta {
  margin-left: 4px;
  font-size: 13px;
  color: var(--el-text-color-secondary);
}
.status {
  margin: 12px 0 0;
  white-space: pre-wrap;
  color: var(--el-text-color-secondary);
}
</style>
