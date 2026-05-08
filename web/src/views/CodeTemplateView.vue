<script setup lang="ts">
import CodeMirrorPane from "@/components/CodeMirrorPane.vue";
import { getDictionary, type DictItem } from "@/api/dictionary";
import { ApiError, apiFetch, type PageResult } from "@/api/http";
import { dictLabel } from "@/utils/dictLabel";
import { formatDate } from "@/utils/formatDate";
import { computed, onMounted, reactive, ref } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";

type DialogMode = "view" | "create" | "edit";

interface Row {
  id: number;
  createdAt?: string;
  updatedAt?: string;
  name: string;
  engine: number | null;
  cate: number | null;
  defaultParams: string;
  temp: string;
  desc: string;
}

const engineFallback = [
  { label: "pyfmt", value: 2 },
  { label: "minijinja", value: 3 },
];
const cateFallback = [{ label: "默认", value: 1 }];

const engineOptions = ref<DictItem[]>(engineFallback);
const cateOptions = ref<DictItem[]>(cateFallback);
const loading = ref(false);
const rows = ref<Row[]>([]);
const total = ref(0);
const page = ref(1);
const pageSize = ref(20);
const multipleSelection = ref<Row[]>([]);

const searchName = ref("");
const searchEngine = ref<number | null>(null);
const searchCate = ref<number | null>(null);
const searchTemp = ref("");
const searchDesc = ref("");

const dialogVisible = ref(false);
const mode = ref<DialogMode>("create");
const editingId = ref<number | null>(null);
const renderOutput = ref("");

const form = reactive({
  name: "",
  engine: 3 as number | undefined,
  cate: 1 as number | undefined,
  defaultParams: "{}",
  temp: "",
  desc: "",
});

const readonlyFields = computed(() => mode.value === "view");
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

async function loadDicts() {
  try {
    const engine = await getDictionary("code_template_engine");
    const merged = [...engine];
    for (const item of engineFallback) {
      if (!merged.some((row) => row.value === item.value)) merged.push(item);
    }
    engineOptions.value = merged.length ? merged : engineFallback;
  } catch {
    engineOptions.value = engineFallback;
  }
  try {
    const cate = await getDictionary("code_template_cate");
    cateOptions.value = cate.length ? cate : cateFallback;
  } catch {
    cateOptions.value = cateFallback;
  }
}

async function load() {
  loading.value = true;
  try {
    const q = new URLSearchParams({
      page: String(page.value),
      pageSize: String(pageSize.value),
    });
    if (searchName.value.trim()) q.set("name", searchName.value.trim());
    if (searchEngine.value != null) q.set("engine", String(searchEngine.value));
    if (searchCate.value != null) q.set("cate", String(searchCate.value));
    if (searchTemp.value.trim()) q.set("temp", searchTemp.value.trim());
    if (searchDesc.value.trim()) q.set("desc", searchDesc.value.trim());
    const data = await apiFetch<PageResult<Row>>(`/api/code-templates?${q}`);
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
  form.engine = 3;
  form.cate = 1;
  form.defaultParams = "{}";
  form.temp = "";
  form.desc = "";
  renderOutput.value = "";
}

function fillForm(row: Row) {
  editingId.value = row.id;
  form.name = row.name;
  form.engine = row.engine ?? 3;
  form.cate = row.cate ?? 1;
  form.defaultParams = row.defaultParams || "{}";
  form.temp = row.temp || "";
  form.desc = row.desc || "";
  renderOutput.value = "";
}

async function fetchRow(id: number) {
  return apiFetch<Row>(`/api/code-templates/${id}`);
}

async function openView(row: Row) {
  mode.value = "view";
  try {
    fillForm(await fetchRow(row.id));
    dialogVisible.value = true;
  } catch (e) {
    ElMessage.error(errMsg(e));
  }
}

function openCreate() {
  mode.value = "create";
  resetForm();
  dialogVisible.value = true;
}

async function openEdit(row: Row) {
  mode.value = "edit";
  try {
    fillForm(await fetchRow(row.id));
    dialogVisible.value = true;
  } catch (e) {
    ElMessage.error(errMsg(e));
  }
}

function closeDialog() {
  dialogVisible.value = false;
}

function onDialogClosed() {
  resetForm();
}

function validateForm() {
  if (!form.name.trim()) {
    ElMessage.warning("请填写名称");
    return false;
  }
  if (form.engine == null) {
    ElMessage.warning("请选择引擎");
    return false;
  }
  if (form.cate == null) {
    ElMessage.warning("请选择类型");
    return false;
  }
  try {
    const data = JSON.parse(form.defaultParams || "{}");
    if (!data || typeof data !== "object" || Array.isArray(data)) {
      ElMessage.warning("默认参数必须是 JSON 对象");
      return false;
    }
  } catch {
    ElMessage.warning("默认参数必须是合法 JSON");
    return false;
  }
  if (!form.temp.trim()) {
    ElMessage.warning("请填写模板");
    return false;
  }
  return true;
}

function body() {
  return {
    name: form.name.trim(),
    engine: form.engine,
    cate: form.cate,
    defaultParams: form.defaultParams.trim() || "{}",
    temp: form.temp,
    desc: form.desc,
  };
}

async function save() {
  if (!validateForm()) return;
  try {
    if (mode.value === "create") {
      await apiFetch("/api/code-templates", {
        method: "POST",
        body: JSON.stringify(body()),
      });
      ElMessage.success("已创建");
    } else if (mode.value === "edit" && editingId.value != null) {
      await apiFetch(`/api/code-templates/${editingId.value}`, {
        method: "PUT",
        body: JSON.stringify(body()),
      });
      ElMessage.success("已保存");
    }
    closeDialog();
    await load();
  } catch (e) {
    ElMessage.error(errMsg(e));
  }
}

async function renderTemplate() {
  if (!validateForm()) return;
  try {
    const data = await apiFetch<{ rst: string }>("/api/code-templates/render", {
      method: "POST",
      body: JSON.stringify({
        engine: form.engine,
        params: form.defaultParams.trim() || "{}",
        temp: form.temp,
      }),
    });
    renderOutput.value = data.rst;
  } catch (e) {
    renderOutput.value = errMsg(e);
  }
}

async function copyOutput() {
  try {
    await navigator.clipboard.writeText(renderOutput.value);
    ElMessage.success("已复制");
  } catch {
    ElMessage.error("复制失败");
  }
}

async function removeRow(row: Row) {
  try {
    await ElMessageBox.confirm(`删除「${row.name}」？`, "确认", {
      type: "warning",
    });
    await apiFetch(`/api/code-templates/${row.id}`, { method: "DELETE" });
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
      `确定删除选中的 ${multipleSelection.value.length} 条数据？`,
      "确认",
      { type: "warning" }
    );
    const ids = multipleSelection.value.map((r) => r.id);
    await apiFetch("/api/code-templates/delete-by-ids", {
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

function search() {
  page.value = 1;
  load();
}

function resetSearch() {
  searchName.value = "";
  searchEngine.value = null;
  searchCate.value = null;
  searchTemp.value = "";
  searchDesc.value = "";
  search();
}

function onSizeChange() {
  page.value = 1;
  load();
}

onMounted(() => {
  loadDicts();
  load();
});
</script>

<template>
  <div class="code-template-page">
    <h2 class="page-title">模板</h2>

    <div class="toolbar">
      <el-input
        v-model="searchName"
        placeholder="名称"
        clearable
        style="width: 150px"
        @keyup.enter="search"
      />
      <el-select
        v-model="searchEngine"
        placeholder="引擎"
        clearable
        style="width: 130px"
      >
        <el-option
          v-for="item in engineOptions"
          :key="item.value"
          :label="item.label"
          :value="item.value"
        />
      </el-select>
      <el-select
        v-model="searchCate"
        placeholder="类型"
        clearable
        style="width: 130px"
      >
        <el-option
          v-for="item in cateOptions"
          :key="item.value"
          :label="item.label"
          :value="item.value"
        />
      </el-select>
      <el-input
        v-model="searchTemp"
        placeholder="模板"
        clearable
        style="width: 150px"
        @keyup.enter="search"
      />
      <el-input
        v-model="searchDesc"
        placeholder="描述"
        clearable
        style="width: 150px"
        @keyup.enter="search"
      />
      <el-button type="primary" @click="search">查询</el-button>
      <el-button @click="resetSearch">重置</el-button>
      <el-button type="success" @click="openCreate">新增</el-button>
      <el-button type="danger" :disabled="!multipleSelection.length" @click="batchDelete">
        删除
      </el-button>
    </div>

    <el-table
      v-loading="loading"
      :data="rows"
      stripe
      style="width: 100%"
      @selection-change="handleSelectionChange"
    >
      <el-table-column type="selection" width="48" />
      <el-table-column prop="id" label="ID" width="70" />
      <el-table-column prop="name" label="名称" min-width="150" show-overflow-tooltip />
      <el-table-column label="引擎" width="110">
        <template #default="{ row }">
          {{ dictLabel(row.engine, engineOptions) }}
        </template>
      </el-table-column>
      <el-table-column label="类型" width="100">
        <template #default="{ row }">
          {{ dictLabel(row.cate, cateOptions) }}
        </template>
      </el-table-column>
      <el-table-column prop="defaultParams" label="默认参数" min-width="180" show-overflow-tooltip />
      <el-table-column prop="temp" label="模板" min-width="220" show-overflow-tooltip />
      <el-table-column prop="desc" label="描述" min-width="160" show-overflow-tooltip />
      <el-table-column label="创建时间" width="168">
        <template #default="{ row }">{{ formatDate(row.createdAt) }}</template>
      </el-table-column>
      <el-table-column label="操作" width="200" fixed="right">
        <template #default="{ row }">
          <el-button link type="primary" @click="openView(row)">查看</el-button>
          <el-button link type="primary" @click="openEdit(row)">变更</el-button>
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
      width="76%"
      destroy-on-close
      class="gva-dialog"
      @closed="onDialogClosed"
    >
      <el-form label-width="110px" label-position="right">
        <el-form-item label="名称">
          <el-input v-model="form.name" :readonly="readonlyFields" clearable />
        </el-form-item>
        <el-form-item label="引擎">
          <el-select
            v-model="form.engine"
            placeholder="请选择"
            style="width: 100%"
            :disabled="readonlyFields"
          >
            <el-option
              v-for="item in engineOptions"
              :key="item.value"
              :label="item.label"
              :value="item.value"
            />
          </el-select>
        </el-form-item>
        <el-form-item label="类型">
          <el-select
            v-model="form.cate"
            placeholder="请选择"
            style="width: 100%"
            :disabled="readonlyFields"
          >
            <el-option
              v-for="item in cateOptions"
              :key="item.value"
              :label="item.label"
              :value="item.value"
            />
          </el-select>
        </el-form-item>
        <el-form-item label="默认参数">
          <CodeMirrorPane
            v-if="dialogVisible"
            :key="`params-${editingId ?? 'n'}-${mode}`"
            v-model="form.defaultParams"
            language="javascript"
            :readonly="readonlyFields"
            height="120px"
          />
        </el-form-item>
        <el-form-item label="模板">
          <CodeMirrorPane
            v-if="dialogVisible"
            :key="`temp-${editingId ?? 'n'}-${mode}`"
            v-model="form.temp"
            language="jinja2"
            :readonly="readonlyFields"
            height="280px"
          />
        </el-form-item>
        <el-form-item label="描述">
          <el-input
            v-model="form.desc"
            type="textarea"
            :rows="2"
            :readonly="readonlyFields"
          />
        </el-form-item>
        <el-form-item label="输出">
          <CodeMirrorPane
            v-if="dialogVisible"
            v-model="renderOutput"
            language="sql"
            readonly
            height="140px"
          />
          <div class="render-actions">
            <el-button type="primary" size="small" @click="renderTemplate">转换</el-button>
            <el-button size="small" @click="copyOutput">复制</el-button>
          </div>
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="closeDialog">{{ readonlyFields ? "关 闭" : "取 消" }}</el-button>
        <el-button v-if="!readonlyFields" type="primary" @click="save">确 定</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<style scoped>
.code-template-page {
  padding: 8px 0;
}
.page-title {
  margin: 0 0 12px;
  font-size: 18px;
  font-weight: 600;
}
.toolbar {
  display: flex;
  gap: 8px;
  margin-bottom: 12px;
  flex-wrap: wrap;
  align-items: center;
}
.render-actions {
  margin-top: 8px;
  display: flex;
  gap: 8px;
}
</style>

<style>
.gva-dialog .el-dialog__body {
  padding-top: 8px;
  overflow-x: hidden;
}
</style>
