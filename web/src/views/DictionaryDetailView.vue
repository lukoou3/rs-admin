<script setup lang="ts">
import { onMounted, reactive, ref, watch } from "vue";
import { useRoute } from "vue-router";
import { ElMessage, ElMessageBox } from "element-plus";
import { ApiError, apiFetch, type PageResult } from "@/api/http";

interface Row {
  id: number;
  createdAt?: string;
  label: string;
  value: number;
  status: boolean;
  sort: number;
  sysDictionaryID: number;
}

const route = useRoute();
const dictId = ref(Number(route.params.id));

const loading = ref(false);
const rows = ref<Row[]>([]);
const total = ref(0);
const page = ref(1);
const pageSize = ref(10);
const searchInfo = reactive({
  label: "",
  value: "",
  status: "" as "" | "true" | "false",
});

const dialogVisible = ref(false);
const isCreate = ref(true);
const editingId = ref<number | null>(null);
const form = reactive({
  label: "",
  value: 0,
  status: true,
  sort: 0,
});

function errMsg(e: unknown) {
  if (e instanceof ApiError) return e.message;
  if (e instanceof Error) return e.message;
  return String(e);
}

function buildQuery() {
  const q = new URLSearchParams({
    page: String(page.value),
    pageSize: String(pageSize.value),
    sysDictionaryID: String(dictId.value),
  });
  if (searchInfo.label.trim()) q.set("label", searchInfo.label.trim());
  if (searchInfo.value.trim()) {
    const n = Number(searchInfo.value.trim());
    if (!Number.isNaN(n)) q.set("value", String(n));
  }
  if (searchInfo.status === "true" || searchInfo.status === "false") {
    q.set("status", searchInfo.status);
  }
  return q;
}

async function load() {
  if (!dictId.value || Number.isNaN(dictId.value)) {
    ElMessage.error("无效的字典 ID");
    return;
  }
  loading.value = true;
  try {
    const data = await apiFetch<PageResult<Row>>(
      `/api/sys-dictionary-details?${buildQuery()}`
    );
    rows.value = data.list;
    total.value = data.total;
  } catch (e) {
    ElMessage.error(errMsg(e));
  } finally {
    loading.value = false;
  }
}

function resetSearch() {
  searchInfo.label = "";
  searchInfo.value = "";
  searchInfo.status = "";
}

function openCreate() {
  isCreate.value = true;
  editingId.value = null;
  form.label = "";
  form.value = 0;
  form.status = true;
  form.sort = 0;
  dialogVisible.value = true;
}

function openEdit(row: Row) {
  isCreate.value = false;
  editingId.value = row.id;
  form.label = row.label;
  form.value = row.value;
  form.status = row.status;
  form.sort = row.sort;
  dialogVisible.value = true;
}

async function save() {
  if (!form.label.trim()) {
    ElMessage.warning("请填写展示值");
    return;
  }
  try {
    if (isCreate.value) {
      await apiFetch("/api/sys-dictionary-details", {
        method: "POST",
        body: JSON.stringify({
          label: form.label.trim(),
          value: form.value,
          status: form.status,
          sort: form.sort,
          sysDictionaryID: dictId.value,
        }),
      });
      ElMessage.success("已创建");
    } else if (editingId.value != null) {
      await apiFetch(`/api/sys-dictionary-details/${editingId.value}`, {
        method: "PUT",
        body: JSON.stringify({
          label: form.label.trim(),
          value: form.value,
          status: form.status,
          sort: form.sort,
        }),
      });
      ElMessage.success("已保存");
    }
    dialogVisible.value = false;
    await load();
  } catch (e) {
    ElMessage.error(errMsg(e));
  }
}

async function removeRow(row: Row) {
  try {
    await ElMessageBox.confirm(`确定删除字典项「${row.label}」？`, "确认", {
      type: "warning",
    });
    await apiFetch(`/api/sys-dictionary-details/${row.id}`, {
      method: "DELETE",
    });
    ElMessage.success("已删除");
    await load();
  } catch (e) {
    if (e === "cancel") return;
    ElMessage.error(errMsg(e));
  }
}

watch(
  () => route.params.id,
  (id) => {
    dictId.value = Number(id);
    page.value = 1;
    load();
  }
);

onMounted(load);
</script>

<template>
  <div class="page-head">
    <h2 class="page-title">字典详情（ID: {{ dictId }}）</h2>
  </div>

  <div class="search-bar">
    <el-form :inline="true" @submit.prevent="load">
      <el-form-item label="展示值">
        <el-input v-model="searchInfo.label" clearable placeholder="搜索" />
      </el-form-item>
      <el-form-item label="字典值">
        <el-input v-model="searchInfo.value" clearable placeholder="精确" />
      </el-form-item>
      <el-form-item label="状态">
        <el-select v-model="searchInfo.status" clearable placeholder="全部" style="width: 120px">
          <el-option label="开启" value="true" />
          <el-option label="停用" value="false" />
        </el-select>
      </el-form-item>
      <el-form-item>
        <el-button type="primary" native-type="submit">查询</el-button>
        <el-button
          @click="
            resetSearch();
            page = 1;
            load();
          "
        >
          重置
        </el-button>
        <el-button type="success" @click="openCreate">新增字典项</el-button>
      </el-form-item>
    </el-form>
  </div>

  <el-table v-loading="loading" :data="rows" stripe border style="width: 100%">
    <el-table-column label="日期" width="170">
      <template #default="{ row }">
        {{ row.createdAt ? new Date(row.createdAt).toLocaleString() : "—" }}
      </template>
    </el-table-column>
    <el-table-column prop="label" label="展示值" min-width="140" />
    <el-table-column prop="value" label="字典值" width="120" />
    <el-table-column label="启用状态" width="110">
      <template #default="{ row }">
        <el-tag :type="row.status ? 'success' : 'info'" size="small">
          {{ row.status ? "开启" : "停用" }}
        </el-tag>
      </template>
    </el-table-column>
    <el-table-column prop="sort" label="排序" width="88" />
    <el-table-column label="操作" width="160" fixed="right">
      <template #default="{ row }">
        <el-button link type="primary" @click="openEdit(row)">变更</el-button>
        <el-button link type="danger" @click="removeRow(row)">删除</el-button>
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

  <el-dialog
    v-model="dialogVisible"
    :title="isCreate ? '新增字典项' : '变更字典项'"
    width="480px"
    destroy-on-close
  >
    <el-form label-width="100px">
      <el-form-item label="展示值" required>
        <el-input v-model="form.label" />
      </el-form-item>
      <el-form-item label="字典值" required>
        <el-input-number v-model="form.value" :step="1" style="width: 100%" />
      </el-form-item>
      <el-form-item label="启用状态">
        <el-switch v-model="form.status" active-text="开启" inactive-text="停用" />
      </el-form-item>
      <el-form-item label="排序">
        <el-input-number v-model="form.sort" :step="1" style="width: 100%" />
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="dialogVisible = false">取消</el-button>
      <el-button type="primary" @click="save">确定</el-button>
    </template>
  </el-dialog>
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
  margin-bottom: 16px;
}
.pager {
  margin-top: 16px;
  display: flex;
  justify-content: flex-end;
}
</style>
