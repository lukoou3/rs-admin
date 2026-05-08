<script setup lang="ts">
import { computed, ref } from "vue";
import { ElMessage } from "element-plus";
import { apiFetch } from "@/api/http";

interface HashResp {
  md5?: string;
  sha1?: string;
  sha256?: string;
  sha512?: string;
  size: number;
  fileName?: string | null;
}

const hashOptions = [
  { label: "MD5", value: "md5", key: "md5" },
  { label: "SHA-1", value: "sha1", key: "sha1" },
  { label: "SHA-256", value: "sha256", key: "sha256" },
  { label: "SHA-512", value: "sha512", key: "sha512" },
] as const;

const activeTab = ref("url");

const urlInput = ref("https://example.com/search?q=rs admin&tag=笔记");
const urlOutput = ref("");

const base64Input = ref("rs-admin 笔记工具");
const base64Output = ref("");

const hashText = ref("rs-admin");
const selectedHashAlgorithms = ref<string[]>(["md5"]);
const hashResult = ref<HashResp | null>(null);
const hashLoading = ref(false);
const fileInput = ref<HTMLInputElement | null>(null);
const selectedFile = ref<File | null>(null);
const dragging = ref(false);

function bytesToBase64(bytes: Uint8Array) {
  let binary = "";
  const chunkSize = 0x8000;
  for (let i = 0; i < bytes.length; i += chunkSize) {
    binary += String.fromCharCode(...bytes.subarray(i, i + chunkSize));
  }
  return btoa(binary);
}

function base64ToText(value: string) {
  const binary = atob(value.replace(/\s+/g, ""));
  const bytes = Uint8Array.from(binary, (char) => char.charCodeAt(0));
  return new TextDecoder().decode(bytes);
}

function encodeUrl() {
  urlOutput.value = encodeURIComponent(urlInput.value);
}

function decodeUrl() {
  try {
    urlOutput.value = decodeURIComponent(urlInput.value);
  } catch {
    ElMessage.error("URL 编码格式无效");
  }
}

function encodeBase64() {
  const bytes = new TextEncoder().encode(base64Input.value);
  base64Output.value = bytesToBase64(bytes);
}

function decodeBase64() {
  try {
    base64Output.value = base64ToText(base64Input.value);
  } catch {
    ElMessage.error("Base64 内容无效");
  }
}

async function copyText(value: string) {
  try {
    await navigator.clipboard.writeText(value);
    ElMessage.success("已复制");
  } catch {
    ElMessage.error("复制失败");
  }
}

async function pasteTo(target: "url" | "base64" | "hash") {
  try {
    const text = await navigator.clipboard.readText();
    if (target === "url") urlInput.value = text;
    if (target === "base64") base64Input.value = text;
    if (target === "hash") hashText.value = text;
    ElMessage.success("已粘贴");
  } catch {
    ElMessage.error("粘贴失败");
  }
}

async function hashTextValue() {
  if (!ensureHashAlgorithm()) return;
  hashLoading.value = true;
  try {
    hashResult.value = await apiFetch<HashResp>("/api/tools/encode/hash/text", {
      method: "POST",
      body: JSON.stringify({
        text: hashText.value,
        algorithms: selectedHashAlgorithms.value,
      }),
    });
    ElMessage.success("计算完成");
  } finally {
    hashLoading.value = false;
  }
}

function chooseFile() {
  fileInput.value?.click();
}

function setFile(file?: File) {
  selectedFile.value = file ?? null;
  hashResult.value = null;
}

function onFileChange(e: Event) {
  const input = e.target as HTMLInputElement;
  setFile(input.files?.[0]);
}

function onDrop(e: DragEvent) {
  dragging.value = false;
  setFile(e.dataTransfer?.files?.[0]);
}

async function hashFileValue() {
  if (!selectedFile.value) {
    ElMessage.warning("先选择一个文件");
    return;
  }
  if (!ensureHashAlgorithm()) return;
  const params = new URLSearchParams({
    fileName: selectedFile.value.name,
    algorithms: selectedHashAlgorithms.value.join(","),
  });
  hashLoading.value = true;
  try {
    hashResult.value = await apiFetch<HashResp>(
      `/api/tools/encode/hash/file?${params.toString()}`,
      {
        method: "POST",
        body: selectedFile.value,
      }
    );
    ElMessage.success("计算完成");
  } finally {
    hashLoading.value = false;
  }
}

function clearHash() {
  hashText.value = "";
  hashResult.value = null;
  selectedFile.value = null;
  if (fileInput.value) fileInput.value.value = "";
}

function ensureHashAlgorithm() {
  if (selectedHashAlgorithms.value.length > 0) return true;
  ElMessage.warning("至少选择一个 Hash 算法");
  return false;
}

function formatSize(size: number) {
  if (size < 1024) return `${size} B`;
  if (size < 1024 * 1024) return `${(size / 1024).toFixed(2)} KB`;
  return `${(size / 1024 / 1024).toFixed(2)} MB`;
}

const hashRows = computed(() => {
  const result = hashResult.value;
  if (!result) return [];
  return hashOptions
    .map((option) => ({
      name: option.label,
      value: result[option.key],
    }))
    .filter((row): row is { name: string; value: string } => Boolean(row.value));
});
</script>

<template>
  <div class="encode-tool-page">
    <h2 class="page-title">编码工具</h2>
    <el-tabs v-model="activeTab" class="tool-tabs">
      <el-tab-pane label="URL" name="url">
        <div class="tool-grid">
          <section class="panel">
            <div class="panel-head">
              <h3>输入</h3>
              <div class="panel-actions">
                <el-button size="small" @click="pasteTo('url')">粘贴</el-button>
                <el-button size="small" @click="urlInput = ''">清空</el-button>
              </div>
            </div>
            <el-input
              v-model="urlInput"
              type="textarea"
              :rows="12"
              resize="vertical"
              placeholder="输入需要编码或解码的 URL 文本"
            />
            <div class="actions">
              <el-button type="primary" size="small" @click="encodeUrl">编码</el-button>
              <el-button type="primary" size="small" @click="decodeUrl">解码</el-button>
            </div>
          </section>
          <section class="panel">
            <div class="panel-head">
              <h3>输出</h3>
              <el-button size="small" @click="copyText(urlOutput)">复制</el-button>
            </div>
            <el-input v-model="urlOutput" type="textarea" :rows="15" resize="vertical" />
          </section>
        </div>
      </el-tab-pane>

      <el-tab-pane label="Base64" name="base64">
        <div class="tool-grid">
          <section class="panel">
            <div class="panel-head">
              <h3>输入</h3>
              <div class="panel-actions">
                <el-button size="small" @click="pasteTo('base64')">粘贴</el-button>
                <el-button size="small" @click="base64Input = ''">清空</el-button>
              </div>
            </div>
            <el-input
              v-model="base64Input"
              type="textarea"
              :rows="12"
              resize="vertical"
              placeholder="输入普通文本或 Base64 内容"
            />
            <div class="actions">
              <el-button type="primary" size="small" @click="encodeBase64">文本转 Base64</el-button>
              <el-button type="primary" size="small" @click="decodeBase64">Base64 转文本</el-button>
            </div>
          </section>
          <section class="panel">
            <div class="panel-head">
              <h3>输出</h3>
              <el-button size="small" @click="copyText(base64Output)">复制</el-button>
            </div>
            <el-input v-model="base64Output" type="textarea" :rows="15" resize="vertical" />
          </section>
        </div>
      </el-tab-pane>

      <el-tab-pane label="Hash" name="hash">
        <div class="hash-layout">
          <section class="panel algorithm-panel">
            <div class="panel-head">
              <h3>算法</h3>
            </div>
            <el-checkbox-group v-model="selectedHashAlgorithms" class="algorithm-options">
              <el-checkbox
                v-for="option in hashOptions"
                :key="option.value"
                :label="option.value"
              >
                {{ option.label }}
              </el-checkbox>
            </el-checkbox-group>
          </section>

          <section class="panel">
            <div class="panel-head">
              <h3>文本</h3>
              <div class="panel-actions">
                <el-button size="small" @click="pasteTo('hash')">粘贴</el-button>
                <el-button size="small" @click="clearHash">清空</el-button>
              </div>
            </div>
            <el-input
              v-model="hashText"
              type="textarea"
              :rows="8"
              resize="vertical"
              placeholder="输入需要计算 Hash 的文本"
            />
            <div class="actions">
              <el-button type="primary" size="small" :loading="hashLoading" @click="hashTextValue">
                计算文本 Hash
              </el-button>
            </div>
          </section>

          <section class="panel">
            <div class="panel-head">
              <h3>文件</h3>
              <el-button size="small" @click="chooseFile">选择文件</el-button>
            </div>
            <input ref="fileInput" class="file-input" type="file" @change="onFileChange" />
            <div
              class="drop-zone"
              :class="{ dragging }"
              @click="chooseFile"
              @dragover.prevent="dragging = true"
              @dragleave.prevent="dragging = false"
              @drop.prevent="onDrop"
            >
              <div class="file-name">{{ selectedFile?.name || "点击选择或拖入文件" }}</div>
              <div class="file-meta">
                {{ selectedFile ? formatSize(selectedFile.size) : "文件会上传到本地 Rust 服务计算" }}
              </div>
            </div>
            <div class="actions">
              <el-button type="primary" size="small" :loading="hashLoading" @click="hashFileValue">
                计算文件 Hash
              </el-button>
            </div>
          </section>

          <section class="panel result-panel">
            <div class="panel-head">
              <h3>结果</h3>
              <span v-if="hashResult" class="result-size">
                {{ hashResult.fileName || "文本" }} / {{ formatSize(hashResult.size) }}
              </span>
            </div>
            <el-empty v-if="!hashResult" description="暂无结果" />
            <div v-else class="hash-results">
              <div v-for="row in hashRows" :key="row.name" class="hash-row">
                <div class="hash-name">{{ row.name }}</div>
                <code>{{ row.value }}</code>
                <el-button size="small" @click="copyText(row.value)">复制</el-button>
              </div>
            </div>
          </section>
        </div>
      </el-tab-pane>
    </el-tabs>
  </div>
</template>

<style scoped>
.encode-tool-page {
  max-width: 100%;
}
.page-title {
  margin: 0 0 8px;
  font-size: 18px;
  font-weight: 600;
}
.tool-tabs {
  max-width: 100%;
}
.tool-grid {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
  gap: 14px;
}
.hash-layout {
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
  grid-column: 1 / -1;
}
.algorithm-panel {
  grid-column: 1 / -1;
}
.algorithm-options {
  display: flex;
  flex-wrap: wrap;
  gap: 10px 18px;
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
.actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  align-items: center;
}
.actions {
  margin-top: 10px;
}
.file-input {
  display: none;
}
.drop-zone {
  display: flex;
  min-height: 132px;
  cursor: pointer;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  border: 1px dashed var(--el-border-color);
  border-radius: 6px;
  background: var(--el-fill-color-lighter);
  transition: border-color 0.15s ease, background 0.15s ease;
}
.drop-zone.dragging {
  border-color: var(--el-color-primary);
  background: var(--el-color-primary-light-9);
}
.file-name {
  max-width: 100%;
  padding: 0 16px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-weight: 600;
  color: var(--el-text-color-primary);
}
.file-meta,
.result-size {
  font-size: 13px;
  color: var(--el-text-color-secondary);
}
.hash-results {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.hash-row {
  display: grid;
  grid-template-columns: 86px minmax(0, 1fr) auto;
  gap: 10px;
  align-items: center;
}
.hash-name {
  color: var(--el-text-color-secondary);
  font-size: 13px;
}
.hash-row code {
  min-width: 0;
  overflow-wrap: anywhere;
  border-radius: 4px;
  padding: 8px;
  background: var(--el-fill-color-lighter);
  color: var(--el-text-color-primary);
  font-family: Consolas, "Courier New", monospace;
  font-size: 12px;
}
@media (max-width: 900px) {
  .tool-grid,
  .hash-layout {
    grid-template-columns: minmax(0, 1fr);
  }
  .hash-row {
    grid-template-columns: 1fr;
  }
}
</style>
