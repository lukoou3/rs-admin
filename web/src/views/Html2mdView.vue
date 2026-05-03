<script setup lang="ts">
import { ref } from "vue";
import { ElMessage } from "element-plus";
import { ApiError, apiFetch } from "@/api/http";

const mdCode = ref("## title");
const imgCnt = ref(0);

const lineAppends = [
  { value: "\n", label: "1" },
  { value: "\n\n", label: "2" },
  { value: "\n\n\n", label: "3" },
];

const formData = ref({
  titlePrefix: "##",
  lineAppend: "\n\n",
  defaultLang: "go",
  imgPath: "D:\\files\\assets",
  downloadImg: 0 as 0 | 1,
  type: 0 as 0 | 1,
  html: "",
});

function errMsg(e: unknown) {
  if (e instanceof ApiError) return e.message;
  if (e instanceof Error) return e.message;
  return String(e);
}

async function execHtml2md() {
  try {
    const data = await apiFetch<{ md: string; imgCnt: number }>(
      "/api/tools/html2md",
      {
        method: "POST",
        body: JSON.stringify(formData.value),
      }
    );
    mdCode.value = data.md;
    imgCnt.value = data.imgCnt;
  } catch (e) {
    mdCode.value = errMsg(e);
    ElMessage.error(errMsg(e));
  }
}

function execClear() {
  formData.value.html = "";
  mdCode.value = "";
  imgCnt.value = 0;
}

async function execCopy() {
  try {
    await navigator.clipboard.writeText(mdCode.value);
    ElMessage.success("已复制");
  } catch (e) {
    ElMessage.error(errMsg(e));
  }
}

async function execPaste() {
  try {
    formData.value.html = await navigator.clipboard.readText();
  } catch (e) {
    ElMessage.error(errMsg(e));
  }
}
</script>

<template>
  <div class="html2md-page">
    <div class="md-pane">
      <el-input
        v-model="mdCode"
        type="textarea"
        :autosize="{ minRows: 22 }"
        class="mono"
        placeholder="Markdown 输出"
      />
    </div>

    <div class="toolbar">
      <el-button type="primary" size="small" @click="execHtml2md">转换</el-button>
      <el-button type="primary" size="small" @click="execClear">清空</el-button>
      <el-button type="primary" size="small" @click="execCopy">复制</el-button>
      <el-button type="primary" size="small" @click="execPaste">粘贴</el-button>
      <div class="toolbar-right">
        <span class="meta">下载图片数: {{ imgCnt }}</span>
        <span class="label">类型：</span>
        <el-select v-model="formData.type" size="small" style="width: 72px">
          <el-option :value="0" label="html" />
          <el-option :value="1" label="md" />
        </el-select>
        <span class="label">下载图片：</span>
        <el-radio-group v-model="formData.downloadImg" size="small" class="ml-8">
          <el-radio :value="1">yes</el-radio>
          <el-radio :value="0">no</el-radio>
        </el-radio-group>
        <span class="label">一级标题：</span>
        <el-select v-model="formData.titlePrefix" size="small" style="width: 76px">
          <el-option value="#" label="#" />
          <el-option value="##" label="##" />
          <el-option value="###" label="###" />
        </el-select>
        <span class="label">换行数：</span>
        <el-select v-model="formData.lineAppend" size="small" style="width: 76px">
          <el-option
            v-for="item in lineAppends"
            :key="item.label"
            :label="item.label"
            :value="item.value"
          />
        </el-select>
        <span class="label">默认语言：</span>
        <el-select v-model="formData.defaultLang" size="small" style="width: 88px">
          <el-option value="python" />
          <el-option value="java" />
          <el-option value="scala" />
          <el-option value="go" />
          <el-option value="js" />
          <el-option value="sh" />
          <el-option value="xml" />
        </el-select>
        <span class="label">路径：</span>
        <el-input v-model="formData.imgPath" clearable style="width: 220px" size="small" />
      </div>
    </div>

    <div class="html-pane">
      <el-input
        v-model="formData.html"
        type="textarea"
        :autosize="{ minRows: 14 }"
        class="mono"
        placeholder="粘贴 HTML（类型为 md 时此处为 Markdown 文本）"
      />
    </div>
  </div>
</template>

<style scoped>
.html2md-page {
  display: flex;
  flex-direction: column;
  gap: 10px;
  max-width: 1200px;
}
.toolbar {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
}
.toolbar-right {
  margin-left: auto;
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
  justify-content: flex-end;
}
.meta {
  padding-right: 12px;
  font-size: 13px;
  color: var(--el-text-color-regular);
}
.label {
  font-size: 13px;
  color: var(--el-text-color-secondary);
}
.ml-8 {
  margin-right: 8px;
}
.mono :deep(textarea) {
  font-family: Consolas, "Courier New", monospace;
  font-size: 13px;
  line-height: 1.45;
}
</style>
