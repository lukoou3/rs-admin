<script setup lang="ts">
import { computed, ref } from "vue";
import { ElMessage } from "element-plus";
import MarkdownIt from "markdown-it";
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

const markdown = new MarkdownIt({
  html: true,
  linkify: true,
  breaks: false,
  typographer: false,
});

const defaultLinkOpen =
  markdown.renderer.rules.link_open ??
  ((tokens, idx, options, env, self) => self.renderToken(tokens, idx, options));

markdown.renderer.rules.link_open = (tokens, idx, options, env, self) => {
  const token = tokens[idx];
  const targetIndex = token.attrIndex("target");
  if (targetIndex < 0) {
    token.attrPush(["target", "_blank"]);
  } else {
    token.attrs![targetIndex][1] = "_blank";
  }

  const relIndex = token.attrIndex("rel");
  if (relIndex < 0) {
    token.attrPush(["rel", "noreferrer"]);
  } else {
    token.attrs![relIndex][1] = "noreferrer";
  }

  return defaultLinkOpen(tokens, idx, options, env, self);
};

const mdPreviewHtml = computed(() => markdown.render(mdCode.value));

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
    <div class="md-editor">
      <div class="md-pane">
        <div class="pane-title">Markdown</div>
        <el-input
          v-model="mdCode"
          type="textarea"
          class="mono md-textarea"
          placeholder="Markdown 输出"
        />
      </div>
      <div class="preview-pane">
        <div class="pane-title">预览</div>
        <div class="markdown-preview" v-html="mdPreviewHtml" />
      </div>
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
.md-editor {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
  min-height: 500px;
  border: 1px solid var(--el-border-color);
  border-radius: 4px;
  overflow: hidden;
  background: var(--el-bg-color);
}
.md-pane,
.preview-pane {
  min-width: 0;
  display: flex;
  flex-direction: column;
}
.md-pane {
  border-right: 1px solid var(--el-border-color);
}
.pane-title {
  height: 34px;
  line-height: 34px;
  padding: 0 12px;
  font-size: 13px;
  color: var(--el-text-color-secondary);
  background: var(--el-fill-color-lighter);
  border-bottom: 1px solid var(--el-border-color);
}
.md-textarea {
  flex: 1;
}
.md-textarea :deep(.el-textarea__inner) {
  height: 466px;
  min-height: 466px;
  resize: none;
  border: 0;
  border-radius: 0;
  box-shadow: none;
}
.markdown-preview {
  height: 466px;
  padding: 16px 18px;
  overflow: auto;
  line-height: 1.65;
  color: var(--el-text-color-primary);
  word-break: break-word;
}
.markdown-preview :deep(h1),
.markdown-preview :deep(h2),
.markdown-preview :deep(h3),
.markdown-preview :deep(h4),
.markdown-preview :deep(h5),
.markdown-preview :deep(h6) {
  margin: 18px 0 10px;
  line-height: 1.35;
}
.markdown-preview :deep(h1:first-child),
.markdown-preview :deep(h2:first-child),
.markdown-preview :deep(h3:first-child) {
  margin-top: 0;
}
.markdown-preview :deep(p) {
  margin: 0 0 12px;
}
.markdown-preview :deep(ul),
.markdown-preview :deep(ol) {
  padding-left: 24px;
  margin: 0 0 12px;
}
.markdown-preview :deep(blockquote) {
  margin: 0 0 12px;
  padding: 8px 12px;
  border-left: 4px solid var(--el-border-color);
  color: var(--el-text-color-secondary);
  background: var(--el-fill-color-lighter);
}
.markdown-preview :deep(table) {
  width: 100%;
  margin: 0 0 12px;
  border-collapse: collapse;
  font-size: 13px;
}
.markdown-preview :deep(th),
.markdown-preview :deep(td) {
  padding: 6px 8px;
  border: 1px solid var(--el-border-color);
  text-align: left;
  vertical-align: top;
}
.markdown-preview :deep(th) {
  background: var(--el-fill-color-lighter);
  font-weight: 600;
}
.markdown-preview :deep(img) {
  max-width: 100%;
  height: auto;
}
.markdown-preview :deep(pre) {
  margin: 0 0 12px;
  padding: 12px;
  overflow: auto;
  border-radius: 4px;
  background: #f6f8fa;
}
.markdown-preview :deep(code) {
  font-family: Consolas, "Courier New", monospace;
  font-size: 13px;
}
.markdown-preview :deep(p code),
.markdown-preview :deep(li code) {
  padding: 2px 4px;
  border-radius: 3px;
  background: var(--el-fill-color-light);
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
@media (max-width: 900px) {
  .md-editor {
    grid-template-columns: 1fr;
  }
  .md-pane {
    border-right: 0;
    border-bottom: 1px solid var(--el-border-color);
  }
}
</style>
