<script setup lang="ts">
import { computed, ref } from "vue";
import { ElMessage } from "element-plus";
import CodeMirrorPane from "@/components/CodeMirrorPane.vue";
import { apiFetch } from "@/api/http";

interface HeaderRow {
  key: string;
  value: string;
}

interface HttpResp {
  status: number;
  statusText: string;
  elapsedMs: number;
  headers: HeaderRow[];
  body: string;
  bodySize: number;
  truncated: boolean;
}

const methods = ["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"];
const method = ref("GET");
const url = ref("https://httpbin.org/get");
const headers = ref<HeaderRow[]>([{ key: "accept", value: "application/json" }]);
const body = ref("");
const timeoutSecs = ref(20);
const maxBodyMb = ref(2);
const loading = ref(false);
const response = ref<HttpResp | null>(null);
const activeResultTab = ref("body");

const statusType = computed(() => {
  const status = response.value?.status ?? 0;
  if (status >= 200 && status < 300) return "success";
  if (status >= 300 && status < 400) return "warning";
  if (status >= 400) return "danger";
  return "info";
});

const responseHeadersText = computed(() => {
  if (!response.value) return "";
  return response.value.headers
    .map((header) => `${header.key}: ${header.value}`)
    .join("\n");
});

function addHeader() {
  headers.value.push({ key: "", value: "" });
}

function removeHeader(index: number) {
  headers.value.splice(index, 1);
  if (headers.value.length === 0) addHeader();
}

function cleanHeaders() {
  headers.value = headers.value.filter((header) => header.key.trim());
  if (headers.value.length === 0) addHeader();
}

async function copyText(text: string) {
  try {
    await navigator.clipboard.writeText(text);
    ElMessage.success("已复制");
  } catch {
    ElMessage.error("复制失败");
  }
}

function formatBytes(size: number) {
  if (size < 1024) return `${size} B`;
  if (size < 1024 * 1024) return `${(size / 1024).toFixed(2)} KB`;
  return `${(size / 1024 / 1024).toFixed(2)} MB`;
}

async function sendRequest() {
  if (!url.value.trim()) {
    ElMessage.warning("先输入 URL");
    return;
  }
  cleanHeaders();
  loading.value = true;
  try {
    response.value = await apiFetch<HttpResp>("/api/tools/http/request", {
      method: "POST",
      body: JSON.stringify({
        method: method.value,
        url: url.value,
        headers: headers.value,
        body: body.value,
        timeoutSecs: timeoutSecs.value,
        maxBodyBytes: Math.round(maxBodyMb.value * 1024 * 1024),
      }),
    });
    activeResultTab.value = "body";
    ElMessage.success("请求完成");
  } finally {
    loading.value = false;
  }
}
</script>

<template>
  <div class="http-tool-page">
    <h2 class="page-title">HTTP 工具</h2>

    <section class="request-line">
      <el-select v-model="method" class="method-select">
        <el-option v-for="item in methods" :key="item" :label="item" :value="item" />
      </el-select>
      <el-input v-model="url" placeholder="https://example.com/api" clearable />
      <el-button type="primary" :loading="loading" @click="sendRequest">发送</el-button>
    </section>

    <div class="request-grid">
      <section class="panel">
        <div class="panel-head">
          <h3>请求头</h3>
          <div class="panel-actions">
            <el-button size="small" @click="addHeader">新增</el-button>
            <el-button size="small" @click="cleanHeaders">整理</el-button>
          </div>
        </div>
        <div class="header-list">
          <div v-for="(header, index) in headers" :key="index" class="header-row">
            <el-input v-model="header.key" placeholder="Header" />
            <el-input v-model="header.value" placeholder="Value" />
            <el-button size="small" @click="removeHeader(index)">删除</el-button>
          </div>
        </div>
        <div class="settings-row">
          <label>
            超时
            <el-input-number v-model="timeoutSecs" :min="1" :max="300" size="small" />
            秒
          </label>
          <label>
            响应体上限
            <el-input-number v-model="maxBodyMb" :min="0.1" :max="20" :step="0.5" size="small" />
            MB
          </label>
        </div>
      </section>

      <section class="panel">
        <div class="panel-head">
          <h3>请求体</h3>
          <el-button size="small" @click="body = ''">清空</el-button>
        </div>
        <CodeMirrorPane v-model="body" language="javascript" height="236px" />
      </section>
    </div>

    <section class="panel result-panel">
      <div class="panel-head">
        <h3>响应</h3>
        <div v-if="response" class="response-meta">
          <el-tag :type="statusType" effect="plain">
            {{ response.status }} {{ response.statusText }}
          </el-tag>
          <span>{{ response.elapsedMs }} ms</span>
          <span>{{ formatBytes(response.bodySize) }}</span>
          <el-tag v-if="response.truncated" type="warning" effect="plain">已截断</el-tag>
        </div>
      </div>
      <el-empty v-if="!response" description="暂无响应" />
      <el-tabs v-else v-model="activeResultTab">
        <el-tab-pane label="Body" name="body">
          <div class="result-actions">
            <el-button size="small" @click="copyText(response.body)">复制 Body</el-button>
          </div>
          <CodeMirrorPane
            :model-value="response.body"
            language="javascript"
            readonly
            height="360px"
          />
        </el-tab-pane>
        <el-tab-pane label="Headers" name="headers">
          <div class="result-actions">
            <el-button size="small" @click="copyText(responseHeadersText)">复制 Headers</el-button>
          </div>
          <CodeMirrorPane
            :model-value="responseHeadersText"
            language="shell"
            readonly
            height="360px"
          />
        </el-tab-pane>
      </el-tabs>
    </section>
  </div>
</template>

<style scoped>
.http-tool-page {
  max-width: 100%;
}
.page-title {
  margin: 0 0 8px;
  font-size: 18px;
  font-weight: 600;
}
.request-line {
  display: grid;
  grid-template-columns: 116px minmax(0, 1fr) auto;
  gap: 10px;
  align-items: center;
  margin-bottom: 14px;
}
.method-select {
  width: 116px;
}
.request-grid {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
  gap: 14px;
}
.panel {
  min-width: 0;
  border: 1px solid var(--el-border-color);
  border-radius: 6px;
  padding: 14px;
  background: var(--el-bg-color);
}
.result-panel {
  margin-top: 14px;
}
.panel-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  margin-bottom: 10px;
}
.panel-head h3 {
  margin: 0;
  font-size: 15px;
  font-weight: 600;
}
.panel-actions,
.response-meta,
.result-actions,
.settings-row {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  align-items: center;
}
.header-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.header-row {
  display: grid;
  grid-template-columns: minmax(0, 0.8fr) minmax(0, 1.2fr) auto;
  gap: 8px;
}
.settings-row {
  margin-top: 12px;
  color: var(--el-text-color-secondary);
  font-size: 13px;
}
.settings-row label {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}
.result-actions {
  justify-content: flex-end;
  margin-bottom: 8px;
}
@media (max-width: 900px) {
  .request-line,
  .request-grid,
  .header-row {
    grid-template-columns: minmax(0, 1fr);
  }
  .method-select {
    width: 100%;
  }
}
</style>
