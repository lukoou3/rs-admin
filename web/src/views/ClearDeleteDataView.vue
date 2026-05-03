<script setup lang="ts">
/**
 * 对齐 gin `tools/clearDeleteData`：含 deleted_at 的表上预览软删数据并物理清除。
 */
import { onMounted, ref } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import { ApiError, apiFetch } from "@/api/http";

interface PreviewResp {
  columns: string[];
  data: Record<string, unknown>[];
}

const selectedTable = ref("");
const tableList = ref<string[]>([]);
const activeTabName = ref<"rst" | "info">("rst");
const infoText = ref("");
const columnsForTable = ref<{ name: string; label: string }[]>([]);
const tableRows = ref<Record<string, unknown>[]>([]);

function errMsg(e: unknown) {
  if (e instanceof ApiError) return e.message;
  if (e instanceof Error) return e.message;
  return String(e);
}

async function loadTables() {
  try {
    const list = await apiFetch<string[]>("/api/tools/clear-delete-data/tables");
    tableList.value = list ?? [];
    if (list?.length && !selectedTable.value) {
      selectedTable.value = list[0];
      await loadPreview();
    }
  } catch (e) {
    infoText.value = errMsg(e);
    activeTabName.value = "info";
    ElMessage.error(errMsg(e));
  }
}

async function loadPreview() {
  const t = selectedTable.value?.trim();
  if (!t) {
    columnsForTable.value = [];
    tableRows.value = [];
    return;
  }
  try {
    const rst = await apiFetch<PreviewResp>(
      `/api/tools/clear-delete-data/preview?table=${encodeURIComponent(t)}`
    );
    columnsForTable.value = (rst.columns ?? []).map((c) => ({
      name: c,
      label: c,
    }));
    tableRows.value = (rst.data ?? []) as Record<string, unknown>[];
    infoText.value = "查询成功";
    activeTabName.value = "rst";
  } catch (e) {
    infoText.value = errMsg(e);
    activeTabName.value = "info";
    ElMessage.error(errMsg(e));
  }
}

async function clearTable() {
  const t = selectedTable.value?.trim();
  if (!t) {
    ElMessage.warning("请选择表");
    return;
  }
  try {
    await ElMessageBox.confirm(
      `确认物理删除表「${t}」中 deleted_at > 2020-01-01 的软删记录？`,
      "确认清除",
      { type: "warning" }
    );
    const res = await apiFetch<{ deleted: number }>(
      "/api/tools/clear-delete-data",
      {
        method: "DELETE",
        body: JSON.stringify({ table: t }),
      }
    );
    ElMessage.success(`删除条数: ${res.deleted}`);
    await loadPreview();
  } catch (e) {
    if (e === "cancel") return;
    ElMessage.error(errMsg(e));
  }
}

onMounted(loadTables);
</script>

<template>
  <div class="clear-delete-page">
    <h2 class="page-title">清除软删数据</h2>
    <p class="hint">
      列出含 <code>deleted_at</code> 的表；预览 <code>deleted_at &gt; 2020-01-01</code>
      的行；清除即对该条件执行物理 DELETE。
    </p>

    <el-form class="toolbar-form">
      <el-form-item label="Table">
        <el-select
          v-model="selectedTable"
          clearable
          placeholder="选择表"
          style="width: 260px"
          @change="loadPreview"
        >
          <el-option
            v-for="item in tableList"
            :key="item"
            :label="item"
            :value="item"
          />
        </el-select>
        <el-button type="primary" style="margin-left: 12px" @click="clearTable">
          清除
        </el-button>
      </el-form-item>
    </el-form>

    <el-tabs v-model="activeTabName">
      <el-tab-pane label="结果" name="rst">
        <el-table :data="tableRows" border stripe style="width: 100%">
          <el-table-column type="index" width="50" />
          <el-table-column
            v-for="col in columnsForTable"
            :key="col.name"
            :prop="col.name"
            :label="col.label"
            min-width="120"
            show-overflow-tooltip
          />
        </el-table>
      </el-tab-pane>
      <el-tab-pane label="信息" name="info">
        <div class="info-box">{{ infoText }}</div>
      </el-tab-pane>
    </el-tabs>
  </div>
</template>

<style scoped>
.clear-delete-page {
  max-width: 1100px;
}
.page-title {
  margin: 0 0 8px;
  font-size: 18px;
  font-weight: 600;
}
.hint {
  margin: 0 0 16px;
  font-size: 13px;
  color: var(--el-text-color-secondary);
  line-height: 1.5;
}
.toolbar-form {
  margin-bottom: 12px;
}
.info-box {
  padding: 12px;
  font-size: 13px;
  color: var(--el-text-color-regular);
  white-space: pre-wrap;
  word-break: break-word;
}
</style>
