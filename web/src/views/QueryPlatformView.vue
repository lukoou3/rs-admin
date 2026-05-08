<script setup lang="ts">
import CodeMirrorPane from "@/components/CodeMirrorPane.vue";
import { ApiError, apiFetch } from "@/api/http";
import { formatSqlLikeLegacy } from "@/utils/formatSql";
import { computed, ref } from "vue";
import { ElMessage } from "element-plus";

interface QueryResp {
  columns: string[];
  data: Record<string, unknown>[];
}

const sql = ref("");
const activeTab = ref("result");
const loading = ref(false);
const columns = ref<string[]>([]);
const rows = ref<Record<string, unknown>[]>([]);
const info = ref("...");

const displayColumns = computed(() =>
  columns.value.map((name) => ({ name, label: name }))
);

function errMsg(e: unknown) {
  if (e instanceof ApiError) return e.message;
  if (e instanceof Error) return e.message;
  return String(e);
}

function formatSql() {
  try {
    sql.value = formatSqlLikeLegacy(sql.value);
    ElMessage.success("已格式化");
  } catch (e) {
    ElMessage.error(errMsg(e));
  }
}

async function copyInfo() {
  try {
    await navigator.clipboard.writeText(sql.value);
    ElMessage.success("复制成功");
  } catch {
    ElMessage.error("复制失败");
  }
}

async function pasteInfo() {
  try {
    sql.value = await navigator.clipboard.readText();
    ElMessage.success("粘贴成功");
  } catch {
    ElMessage.error("粘贴失败");
  }
}

async function execQuery() {
  if (!sql.value.trim()) {
    ElMessage.warning("请输入 SQL");
    return;
  }
  loading.value = true;
  try {
    const rst = await apiFetch<QueryResp>("/api/query-platform/query", {
      method: "POST",
      body: JSON.stringify({ sql: sql.value }),
    });
    columns.value = rst.columns;
    rows.value = rst.data;
    info.value = `查询成功，返回 ${rst.data.length} 行`;
    activeTab.value = "result";
  } catch (e) {
    info.value = errMsg(e);
    activeTab.value = "info";
  } finally {
    loading.value = false;
  }
}
</script>

<template>
  <div class="query-platform-page">
    <h2 class="page-title">查询平台</h2>
    <CodeMirrorPane
      v-model="sql"
      language="sql"
      height="300px"
    />
    <div class="actions">
      <el-button type="primary" size="small" @click="formatSql">
        格式化sql
      </el-button>
      <el-button type="primary" size="small" @click="copyInfo">复制</el-button>
      <el-button type="primary" size="small" @click="pasteInfo">粘贴</el-button>
      <el-button type="primary" size="small" :loading="loading" @click="execQuery">
        查询
      </el-button>
    </div>

    <el-tabs v-model="activeTab" class="result-tabs">
      <el-tab-pane label="结果" name="result">
        <el-table
          v-loading="loading"
          :data="rows"
          border
          stripe
          style="width: 100%"
        >
          <el-table-column type="index" width="56" />
          <el-table-column
            v-for="item in displayColumns"
            :key="item.name"
            :prop="item.name"
            :label="item.label"
            min-width="140"
            show-overflow-tooltip
          />
        </el-table>
      </el-tab-pane>
      <el-tab-pane label="信息" name="info">
        <pre class="info">{{ info }}</pre>
      </el-tab-pane>
    </el-tabs>
  </div>
</template>

<style scoped>
.query-platform-page {
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
  margin: 12px 0;
}
.result-tabs {
  margin-top: 4px;
}
.info {
  margin: 0;
  white-space: pre-wrap;
  word-break: break-word;
  color: var(--el-text-color-secondary);
}
</style>
