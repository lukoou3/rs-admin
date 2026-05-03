<script setup lang="ts">
import { onMounted, ref } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import { ApiError, apiFetch, type PageResult } from "@/api/http";
import { formatDate } from "@/utils/formatDate";

interface OpUser {
  userName: string;
  nickName: string;
}

interface Row {
  id: number;
  createdAt?: string;
  ip: string;
  method: string;
  path: string;
  status: number;
  latency: number;
  agent: string;
  errorMessage: string;
  body: string;
  resp: string;
  userId: number;
  user?: OpUser | null;
}

const loading = ref(false);
const rows = ref<Row[]>([]);
const total = ref(0);
const page = ref(1);
const pageSize = ref(10);
const searchInfo = ref({
  method: "",
  path: "",
  status: "" as string | number | "",
});

const multipleSelection = ref<Row[]>([]);

function errMsg(e: unknown) {
  if (e instanceof ApiError) return e.message;
  if (e instanceof Error) return e.message;
  return String(e);
}

function fmtBody(raw: string): string {
  if (!raw?.trim()) return "";
  try {
    const j = JSON.parse(raw);
    return JSON.stringify(j, null, 2);
  } catch {
    return raw;
  }
}

function latencyMs(latency: number): string {
  if (latency == null || Number.isNaN(latency)) return "—";
  return (latency / 1_000_000).toFixed(2);
}

function buildQuery() {
  const q = new URLSearchParams({
    page: String(page.value),
    pageSize: String(pageSize.value),
  });
  if (searchInfo.value.method.trim()) q.set("method", searchInfo.value.method.trim());
  if (searchInfo.value.path.trim()) q.set("path", searchInfo.value.path.trim());
  if (searchInfo.value.status !== "" && searchInfo.value.status != null) {
    const n = Number(searchInfo.value.status);
    if (!Number.isNaN(n)) q.set("status", String(n));
  }
  return q;
}

async function load() {
  loading.value = true;
  try {
    const data = await apiFetch<PageResult<Row>>(
      `/api/operation-records?${buildQuery()}`
    );
    rows.value = data.list;
    total.value = data.total;
  } catch (e) {
    ElMessage.error(errMsg(e));
  } finally {
    loading.value = false;
  }
}

function onReset() {
  searchInfo.value = { method: "", path: "", status: "" };
}

function onSubmit() {
  page.value = 1;
  load();
}

function handleSelectionChange(val: Row[]) {
  multipleSelection.value = val;
}

async function deleteOne(row: Row) {
  await apiFetch(`/api/operation-records/${row.id}`, { method: "DELETE" });
  ElMessage.success("已删除");
  await load();
}

async function confirmDelete(row: Row) {
  try {
    await ElMessageBox.confirm("确定删除该条记录？", "确认", { type: "warning" });
    await deleteOne(row);
  } catch (e) {
    if (e === "cancel") return;
    ElMessage.error(errMsg(e));
  }
}

async function onBatchDelete() {
  try {
    await ElMessageBox.confirm(
      `确定删除选中的 ${multipleSelection.value.length} 条记录？`,
      "确认",
      { type: "warning" }
    );
    const ids = multipleSelection.value.map((r) => r.id);
    await apiFetch("/api/operation-records/delete-by-ids", {
      method: "POST",
      body: JSON.stringify({ ids }),
    });
    ElMessage.success("已删除");
    await load();
  } catch (e) {
    if (e === "cancel") return;
    ElMessage.error(errMsg(e));
  }
}

onMounted(load);
</script>

<template>
  <div class="page-head">
    <h2 class="page-title">操作历史</h2>
  </div>

  <div class="search-bar">
    <el-form :inline="true" @submit.prevent="onSubmit">
      <el-form-item label="请求方法">
        <el-input v-model="searchInfo.method" clearable placeholder="如 GET" />
      </el-form-item>
      <el-form-item label="请求路径">
        <el-input v-model="searchInfo.path" clearable placeholder="模糊" />
      </el-form-item>
      <el-form-item label="状态码">
        <el-input v-model="searchInfo.status" clearable placeholder="如 200" />
      </el-form-item>
      <el-form-item>
        <el-button type="primary" native-type="submit">查询</el-button>
        <el-button
          @click="
            onReset();
            onSubmit();
          "
        >
          重置
        </el-button>
      </el-form-item>
    </el-form>
  </div>

  <div class="toolbar">
    <el-button :disabled="!multipleSelection.length" @click="onBatchDelete">删除</el-button>
  </div>

  <el-table
    v-loading="loading"
    :data="rows"
    stripe
    border
    style="width: 100%"
    row-key="id"
    @selection-change="handleSelectionChange"
  >
    <el-table-column type="selection" width="48" />
    <el-table-column label="操作人" width="160">
      <template #default="{ row }">
        <span v-if="row.user">
          {{ row.user.userName }}（{{ row.user.nickName }}）
        </span>
        <span v-else class="muted">—</span>
      </template>
    </el-table-column>
    <el-table-column label="日期" width="170">
      <template #default="{ row }">
        {{ formatDate(row.createdAt) }}
      </template>
    </el-table-column>
    <el-table-column label="状态码" width="100">
      <template #default="{ row }">
        <el-tag type="success" size="small">{{ row.status }}</el-tag>
      </template>
    </el-table-column>
    <el-table-column prop="ip" label="请求 IP" width="130" />
    <el-table-column prop="method" label="方法" width="88" />
    <el-table-column prop="path" label="路径" min-width="200" show-overflow-tooltip />
    <el-table-column label="耗时(ms)" width="100">
      <template #default="{ row }">
        {{ latencyMs(row.latency) }}
      </template>
    </el-table-column>
    <el-table-column label="请求" width="72" align="center">
      <template #default="{ row }">
        <el-popover v-if="row.body" placement="left-start" trigger="click" :width="420">
          <pre class="popover-pre">{{ fmtBody(row.body) }}</pre>
          <template #reference>
            <el-button link type="primary" class="link-btn">查看</el-button>
          </template>
        </el-popover>
        <span v-else class="muted">无</span>
      </template>
    </el-table-column>
    <el-table-column label="响应" width="72" align="center">
      <template #default="{ row }">
        <el-popover v-if="row.resp" placement="left-start" trigger="click" :width="420">
          <pre class="popover-pre">{{ fmtBody(row.resp) }}</pre>
          <template #reference>
            <el-button link type="primary" class="link-btn">查看</el-button>
          </template>
        </el-popover>
        <span v-else class="muted">无</span>
      </template>
    </el-table-column>
    <el-table-column label="操作" width="88" fixed="right">
      <template #default="{ row }">
        <el-button link type="danger" @click="confirmDelete(row)">删除</el-button>
      </template>
    </el-table-column>
  </el-table>

  <div class="pager">
    <el-pagination
      v-model:current-page="page"
      v-model:page-size="pageSize"
      :total="total"
      :page-sizes="[10, 30, 50, 100]"
      layout="total, sizes, prev, pager, next, jumper"
      background
      @current-change="load"
      @size-change="load"
    />
  </div>
</template>

<style scoped>
.page-head {
  margin-bottom: 16px;
}
.page-title {
  margin: 0;
  font-size: 20px;
  font-weight: 600;
}
.search-bar {
  margin-bottom: 12px;
}
.toolbar {
  margin-bottom: 12px;
}
.pager {
  margin-top: 16px;
  display: flex;
  justify-content: flex-end;
}
.popover-pre {
  margin: 0;
  max-height: 320px;
  overflow: auto;
  font-size: 12px;
  white-space: pre-wrap;
  word-break: break-all;
}
.link-btn {
  padding: 0;
  min-height: auto;
}
.muted {
  color: var(--el-text-color-secondary);
}
</style>
