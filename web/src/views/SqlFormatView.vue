<script setup lang="ts">
import CodeMirrorPane from "@/components/CodeMirrorPane.vue";
import { ref } from "vue";
import { ElMessage } from "element-plus";
import { formatSqlLikeLegacy } from "@/utils/formatSql";

/**
 * 对应旧项目 `view/sql/format/format.vue`：独立 SQL 格式化页，仅前端，无后台。
 */
const formSql = ref("");

function errMsg(e: unknown) {
  if (e instanceof Error) return e.message;
  return String(e);
}

function formatSql() {
  try {
    formSql.value = formatSqlLikeLegacy(formSql.value);
    ElMessage.success("已格式化");
  } catch (e) {
    ElMessage.error(errMsg(e));
  }
}

async function copyInfo() {
  try {
    await navigator.clipboard.writeText(formSql.value);
    ElMessage.success("复制成功");
  } catch {
    ElMessage.error("复制失败");
  }
}

async function pasteInfo() {
  try {
    const t = await navigator.clipboard.readText();
    formSql.value = t;
    ElMessage.success("粘贴成功");
  } catch {
    ElMessage.error("粘贴失败");
  }
}
</script>

<template>
  <div class="sql-format-page">
    <h2 class="page-title">SQL 格式化</h2>
    <div class="editor-wrap">
      <CodeMirrorPane
        v-model="formSql"
        language="sql"
        height="400px"
      />
    </div>
    <div class="actions">
      <el-button type="primary" size="small" @click="formatSql">
        格式化sql
      </el-button>
      <el-button type="primary" size="small" @click="copyInfo">复制</el-button>
      <el-button type="primary" size="small" @click="pasteInfo">粘贴</el-button>
    </div>
  </div>
</template>

<style scoped>
.sql-format-page {
  max-width: 100%;
}
.page-title {
  margin: 0 0 8px;
  font-size: 18px;
  font-weight: 600;
}
.editor-wrap {
  width: 100%;
  max-width: 100%;
  margin-bottom: 12px;
}
.actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}
</style>
