<script setup lang="ts">
import CodeMirrorPane from "@/components/CodeMirrorPane.vue";
import { computed, onMounted, reactive, ref } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import { getDictionary, type DictItem } from "@/api/dictionary";
import { ApiError, apiFetch, type PageResult } from "@/api/http";
import { dictLabel } from "@/utils/dictLabel";
import { formatDate } from "@/utils/formatDate";

type DialogMode = "view" | "create" | "edit";

interface Row {
  id: number;
  name: string;
  alias: string;
  cate: number | null;
  introduction: string;
  sql: string;
  createdAt?: string;
  updatedAt?: string;
}

const cateOptions = ref<DictItem[]>([]);

const loading = ref(false);
const rows = ref<Row[]>([]);
const total = ref(0);
const page = ref(1);
const pageSize = ref(20);
const keyword = ref("");

const dialogVisible = ref(false);
const mode = ref<DialogMode>("create");
const editingId = ref<number | null>(null);
const form = reactive({
  name: "",
  alias: "",
  cate: null as number | null,
  introduction: "",
  sql: "",
});

const isView = computed(() => mode.value === "view");

const dialogTitle = computed(() => {
  if (mode.value === "view") return "查看";
  if (mode.value === "edit") return "变更";
  return "新增";
});

function errMsg(e: unknown) {
  if (e instanceof ApiError) return e.message;
  if (e instanceof Error) return e.message;
  return String(e);
}

async function load() {
  loading.value = true;
  try {
    const q = new URLSearchParams({
      page: String(page.value),
      page_size: String(pageSize.value),
    });
    if (keyword.value.trim()) q.set("keyword", keyword.value.trim());
    const data = await apiFetch<PageResult<Row>>(`/api/sql-datasources?${q}`);
    rows.value = data.list;
    total.value = data.total;
  } catch (e) {
    ElMessage.error(errMsg(e));
  } finally {
    loading.value = false;
  }
}

function resetForm() {
  editingId.value = null;
  form.name = "";
  form.alias = "";
  form.cate = null;
  form.introduction = "";
  form.sql = "";
}

function openView(row: Row) {
  mode.value = "view";
  editingId.value = row.id;
  form.name = row.name;
  form.alias = row.alias ?? "";
  form.cate = row.cate;
  form.introduction = row.introduction ?? "";
  form.sql = row.sql ?? "";
  dialogVisible.value = true;
}

function openCreate() {
  mode.value = "create";
  resetForm();
  dialogVisible.value = true;
}

function openEdit(row: Row) {
  mode.value = "edit";
  editingId.value = row.id;
  form.name = row.name;
  form.alias = row.alias ?? "";
  form.cate = row.cate;
  form.introduction = row.introduction ?? "";
  form.sql = row.sql ?? "";
  dialogVisible.value = true;
}

function closeDialog() {
  dialogVisible.value = false;
}

function onDialogClosed() {
  resetForm();
}

async function save() {
  if (!form.name.trim()) {
    ElMessage.warning("请填写名称");
    return;
  }
  const body = {
    name: form.name,
    alias: form.alias,
    cate: form.cate,
    introduction: form.introduction,
    sql: form.sql,
  };
  try {
    if (mode.value === "create") {
      await apiFetch("/api/sql-datasources", {
        method: "POST",
        body: JSON.stringify(body),
      });
      ElMessage.success("已创建");
    } else if (mode.value === "edit" && editingId.value != null) {
      await apiFetch(`/api/sql-datasources/${editingId.value}`, {
        method: "PUT",
        body: JSON.stringify(body),
      });
      ElMessage.success("已保存");
    }
    closeDialog();
    await load();
  } catch (e) {
    ElMessage.error(errMsg(e));
  }
}

async function removeRow(row: Row) {
  try {
    await ElMessageBox.confirm(`删除「${row.name}」？`, "确认", {
      type: "warning",
    });
    await apiFetch(`/api/sql-datasources/${row.id}`, { method: "DELETE" });
    ElMessage.success("已删除");
    await load();
  } catch (e) {
    if (e === "cancel") return;
    ElMessage.error(errMsg(e));
  }
}

function search() {
  page.value = 1;
  load();
}

function onSizeChange() {
  page.value = 1;
  load();
}

async function loadCateDict() {
  try {
    cateOptions.value = await getDictionary("sql_datasource_cate");
  } catch (e) {
    ElMessage.error(errMsg(e));
  }
}

onMounted(async () => {
  await loadCateDict();
  await load();
});
</script>

<template>
  <div class="toolbar">
    <el-input
      v-model="keyword"
      placeholder="按名称搜索"
      clearable
      style="width: 220px"
      @keyup.enter="search"
    />
    <el-button type="primary" @click="search">搜索</el-button>
    <el-button type="success" @click="openCreate">新建</el-button>
  </div>

  <el-table v-loading="loading" :data="rows" stripe style="width: 100%">
    <el-table-column prop="id" label="ID" width="80" />
    <el-table-column prop="name" label="名称" min-width="140" show-overflow-tooltip />
    <el-table-column prop="alias" label="别名" width="100" show-overflow-tooltip />
    <el-table-column label="分类" width="100" show-overflow-tooltip>
      <template #default="{ row }">{{
        dictLabel(row.cate, cateOptions)
      }}</template>
    </el-table-column>
    <el-table-column prop="introduction" label="简介" min-width="200" show-overflow-tooltip />
    <el-table-column label="创建时间" width="168" align="left">
      <template #default="{ row }">{{ formatDate(row.createdAt) }}</template>
    </el-table-column>
    <el-table-column label="修改时间" width="168" align="left">
      <template #default="{ row }">{{ formatDate(row.updatedAt) }}</template>
    </el-table-column>
    <el-table-column label="操作" width="220" fixed="right">
      <template #default="{ row }">
        <el-button link type="primary" @click="openView(row)">查看</el-button>
        <el-button link type="primary" @click="openEdit(row)">编辑</el-button>
        <el-button link type="danger" @click="removeRow(row)">删除</el-button>
      </template>
    </el-table-column>
  </el-table>

  <el-pagination
    v-model:current-page="page"
    v-model:page-size="pageSize"
    :total="total"
    :page-sizes="[10, 20, 50, 100]"
    layout="total, sizes, prev, pager, next"
    style="margin-top: 16px"
    @current-change="load"
    @size-change="onSizeChange"
  />

  <el-dialog
    v-model="dialogVisible"
    :title="dialogTitle"
    width="70%"
    destroy-on-close
    class="code-dialog gva-dialog"
    @closed="onDialogClosed"
  >
    <el-form
      :model="form"
      label-position="right"
      label-width="100px"
      class="code-dialog-form"
    >
      <el-form-item label="数据源名称:">
        <el-input v-model="form.name" clearable :readonly="isView" />
      </el-form-item>
      <el-form-item label="数据源别名:">
        <el-input v-model="form.alias" clearable :readonly="isView" />
      </el-form-item>
      <el-form-item label="分类:">
        <el-select
          v-model="form.cate"
          placeholder="请选择"
          style="width: 100%"
          clearable
          :disabled="isView"
        >
          <el-option
            v-for="item in cateOptions"
            :key="item.value"
            :label="item.label"
            :value="item.value"
          />
        </el-select>
      </el-form-item>
      <!-- 与旧项目顺序一致：sql 在简介前 -->
      <el-form-item label="sql:" class="code-form-item">
        <CodeMirrorPane
          v-if="dialogVisible"
          :key="`${mode}-${editingId ?? 'n'}`"
          v-model="form.sql"
          language="sql"
          :readonly="isView"
          height="400px"
        />
      </el-form-item>
      <el-form-item label="简介:">
        <el-input
          v-model="form.introduction"
          type="textarea"
          :rows="4"
          :readonly="isView"
        />
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="closeDialog">{{ isView ? "关 闭" : "取 消" }}</el-button>
      <el-button v-if="!isView" type="primary" @click="save">确 定</el-button>
    </template>
  </el-dialog>
</template>

<style scoped>
.toolbar {
  display: flex;
  gap: 8px;
  margin-bottom: 12px;
  flex-wrap: wrap;
  align-items: center;
}
</style>

<style>
.gva-dialog .el-dialog__body {
  padding-top: 8px;
  overflow-x: hidden;
}
.code-dialog-form {
  max-width: 100%;
}
.code-dialog-form .code-form-item :deep(.el-form-item__content) {
  min-width: 0;
  max-width: 100%;
}
</style>
