<script setup lang="ts">
import { onMounted, reactive, ref } from "vue";
import { useRouter } from "vue-router";
import { ElMessage, ElMessageBox } from "element-plus";
import { ApiError, apiFetch, type PageResult } from "@/api/http";
import { formatDate } from "@/utils/formatDate";

interface Row {
  id: number;
  createdAt?: string;
  name: string;
  type: string;
  status: boolean;
  desc: string;
}

const router = useRouter();

const loading = ref(false);
const rows = ref<Row[]>([]);
const total = ref(0);
const page = ref(1);
const pageSize = ref(10);
const searchInfo = reactive({
  name: "",
  type: "",
  desc: "",
  status: "" as "" | "true" | "false",
});
const multipleSelection = ref<Row[]>([]);

const dialogVisible = ref(false);
const isCreate = ref(true);
const editingId = ref<number | null>(null);
const form = reactive({
  name: "",
  type: "",
  status: true,
  desc: "",
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
  });
  if (searchInfo.name.trim()) q.set("name", searchInfo.name.trim());
  if (searchInfo.type.trim()) q.set("type", searchInfo.type.trim());
  if (searchInfo.desc.trim()) q.set("desc", searchInfo.desc.trim());
  if (searchInfo.status === "true" || searchInfo.status === "false") {
    q.set("status", searchInfo.status);
  }
  return q;
}

async function load() {
  loading.value = true;
  try {
    const data = await apiFetch<PageResult<Row>>(
      `/api/sys-dictionaries?${buildQuery()}`
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
  searchInfo.name = "";
  searchInfo.type = "";
  searchInfo.desc = "";
  searchInfo.status = "";
}

function openCreate() {
  isCreate.value = true;
  editingId.value = null;
  form.name = "";
  form.type = "";
  form.status = true;
  form.desc = "";
  dialogVisible.value = true;
}

function openEdit(row: Row) {
  isCreate.value = false;
  editingId.value = row.id;
  form.name = row.name;
  form.type = row.type;
  form.status = row.status;
  form.desc = row.desc ?? "";
  dialogVisible.value = true;
}

async function save() {
  if (!form.name.trim() || !form.type.trim()) {
    ElMessage.warning("请填写字典名（中）与字典名（英）");
    return;
  }
  try {
    if (isCreate.value) {
      await apiFetch("/api/sys-dictionaries", {
        method: "POST",
        body: JSON.stringify({
          name: form.name.trim(),
          type: form.type.trim(),
          status: form.status,
          desc: form.desc.trim(),
        }),
      });
      ElMessage.success("已创建");
    } else if (editingId.value != null) {
      await apiFetch(`/api/sys-dictionaries/${editingId.value}`, {
        method: "PUT",
        body: JSON.stringify({
          name: form.name.trim(),
          type: form.type.trim(),
          status: form.status,
          desc: form.desc.trim(),
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
    await ElMessageBox.confirm(`确定删除字典「${row.name}」及其全部字典项？`, "确认", {
      type: "warning",
    });
    await apiFetch(`/api/sys-dictionaries/${row.id}`, { method: "DELETE" });
    ElMessage.success("已删除");
    await load();
  } catch (e) {
    if (e === "cancel") return;
    ElMessage.error(errMsg(e));
  }
}

function handleSelectionChange(val: Row[]) {
  multipleSelection.value = val;
}

async function batchDelete() {
  if (!multipleSelection.value.length) {
    ElMessage.warning("请选择要删除的数据");
    return;
  }
  try {
    await ElMessageBox.confirm(
      `确定删除选中的 ${multipleSelection.value.length} 个字典及其全部字典项？`,
      "确认",
      { type: "warning" }
    );
    const ids = multipleSelection.value.map((r) => r.id);
    await apiFetch("/api/sys-dictionaries/delete-by-ids", {
      method: "POST",
      body: JSON.stringify({ ids }),
    });
    ElMessage.success("已删除");
    multipleSelection.value = [];
    await load();
  } catch (e) {
    if (e === "cancel") return;
    ElMessage.error(errMsg(e));
  }
}

function goDetail(row: Row) {
  router.push({ name: "dictionaryDetail", params: { id: String(row.id) } });
}

onMounted(load);
</script>

<template>
  <div class="page-head">
    <h2 class="page-title">字典管理</h2>
  </div>

  <div class="search-bar">
    <el-form :inline="true" @submit.prevent="load">
      <el-form-item label="字典名（中）">
        <el-input v-model="searchInfo.name" clearable placeholder="搜索" />
      </el-form-item>
      <el-form-item label="字典名（英）">
        <el-input v-model="searchInfo.type" clearable placeholder="搜索" />
      </el-form-item>
      <el-form-item label="状态">
        <el-select v-model="searchInfo.status" clearable placeholder="全部" style="width: 120px">
          <el-option label="开启" value="true" />
          <el-option label="停用" value="false" />
        </el-select>
      </el-form-item>
      <el-form-item label="描述">
        <el-input v-model="searchInfo.desc" clearable placeholder="搜索" />
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
        <el-button type="success" @click="openCreate">新增</el-button>
        <el-button type="danger" :disabled="!multipleSelection.length" @click="batchDelete">
          删除
        </el-button>
      </el-form-item>
    </el-form>
  </div>

  <el-table
    v-loading="loading"
    :data="rows"
    stripe
    border
    style="width: 100%"
    @selection-change="handleSelectionChange"
  >
    <el-table-column type="selection" width="48" />
    <el-table-column label="日期" width="170">
      <template #default="{ row }">
        {{ formatDate(row.createdAt) }}
      </template>
    </el-table-column>
    <el-table-column prop="name" label="字典名（中）" min-width="140" />
    <el-table-column prop="type" label="字典名（英）" width="140" />
    <el-table-column label="状态" width="100">
      <template #default="{ row }">
        <el-tag :type="row.status ? 'success' : 'info'" size="small">
          {{ row.status ? "开启" : "停用" }}
        </el-tag>
      </template>
    </el-table-column>
    <el-table-column prop="desc" label="描述" min-width="200" show-overflow-tooltip />
    <el-table-column label="操作" width="220" fixed="right">
      <template #default="{ row }">
        <el-button link type="primary" @click="goDetail(row)">详情</el-button>
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
    :title="isCreate ? '新增字典' : '变更字典'"
    width="520px"
    destroy-on-close
  >
    <el-form label-width="110px">
      <el-form-item label="字典名（中）" required>
        <el-input v-model="form.name" />
      </el-form-item>
      <el-form-item label="字典名（英）" required>
        <el-input v-model="form.type" />
      </el-form-item>
      <el-form-item label="状态">
        <el-switch v-model="form.status" active-text="开启" inactive-text="停用" />
      </el-form-item>
      <el-form-item label="描述">
        <el-input v-model="form.desc" type="textarea" :rows="3" />
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
