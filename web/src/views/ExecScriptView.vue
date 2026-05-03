<script setup lang="ts">
/**
 * 对齐 gin-vue-admin `view/code/execScript/execScript.vue`：
 * CRUD + runExecScript / getExecScriptRunInfo / isExecScriptRunning。
 */
import CodeMirrorPane from "@/components/CodeMirrorPane.vue";
import { computed, onBeforeUnmount, ref, watch } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import { getDictionary, type DictItem } from "@/api/dictionary";
import { ApiError, apiFetch, type PageResult } from "@/api/http";
import { dictLabel } from "@/utils/dictLabel";
import { formatDate } from "@/utils/formatDate";

type DialogMode = "view" | "create" | "edit";

interface Row {
  id: number;
  name: string;
  cate: number | null;
  interpreter: string;
  encoding: string;
  defaultParams: string;
  content: string;
  desc: string;
  lastExecStartTime?: string | null;
  lastExecEndTime?: string | null;
  lastExecParams?: string | null;
  lastExecInfo?: string | null;
  createdAt?: string;
  updatedAt?: string;
}

interface FormState {
  id?: number;
  name: string;
  cate: number | undefined;
  interpreter: string;
  encoding: string;
  defaultParams: string;
  content: string;
  desc: string;
  lastExecStartTime?: string | null;
  lastExecEndTime?: string | null;
  lastExecParams?: string | null;
  lastExecInfo?: string | null;
}

interface RunInfo {
  finished: number;
  rstCode: number;
  err?: string | null;
  startTime?: string | null;
  endTime?: string | null;
  params: string;
  stdin: string;
  outStr: string;
  errStr: string;
}

const cateOptions = ref<DictItem[]>([]);
const encodingOptions = [
  { value: "utf-8", label: "utf-8" },
  { value: "gbk", label: "gbk" },
];

const loading = ref(false);
const rows = ref<Row[]>([]);
const total = ref(0);
const page = ref(1);
const pageSize = ref(20);
const searchName = ref("");
const searchCate = ref<number | null>(null);
const searchInterpreter = ref("");
const searchContent = ref("");
const searchDesc = ref("");

const dialogVisible = ref(false);
const mode = ref<DialogMode>("create");
const editingId = ref<number | null>(null);

const form = ref<FormState>({
  id: undefined,
  name: "",
  cate: undefined,
  interpreter: "",
  encoding: "utf-8",
  defaultParams: "{}",
  content: "",
  desc: "",
});

const lastExecStartTime = ref("");
const lastExecEndTime = ref("");
const lastExecParams = ref("");
const execStdin = ref("");
const lastExecStdOut = ref("");
const lastExecErrOut = ref("");

let pollTimer: ReturnType<typeof setTimeout> | null = null;

/** 与旧 gin execScript 列表展示一致，过长再悬停看全文 */
const MAX_LAST_EXEC_PARAMS_PREVIEW = 120;

const lastExecParamsPreview = computed(() => {
  const s = lastExecParams.value;
  if (!s) return "";
  if (s.length <= MAX_LAST_EXEC_PARAMS_PREVIEW) return s;
  return `${s.slice(0, MAX_LAST_EXEC_PARAMS_PREVIEW)}…`;
});

const lastExecParamsOverflow = computed(
  () => (lastExecParams.value?.length ?? 0) > MAX_LAST_EXEC_PARAMS_PREVIEW
);

const isView = computed(() => mode.value === "view");
const dialogTitle = computed(() => {
  if (mode.value === "view") return "查看";
  if (mode.value === "edit") return "变更";
  return "新增";
});

const readonlyFields = computed(() => mode.value === "view");

function errMsg(e: unknown) {
  if (e instanceof ApiError) return e.message;
  if (e instanceof Error) return e.message;
  return String(e);
}

function clearPoll() {
  if (pollTimer != null) {
    clearTimeout(pollTimer);
    pollTimer = null;
  }
}

async function loadDict() {
  try {
    cateOptions.value = await getDictionary("exec_cript_cate");
  } catch {
    cateOptions.value = [];
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
    if (searchCate.value != null) q.set("cate", String(searchCate.value));
    if (searchInterpreter.value.trim()) {
      q.set("interpreter", searchInterpreter.value.trim());
    }
    if (searchContent.value.trim()) {
      q.set("content", searchContent.value.trim());
    }
    if (searchDesc.value.trim()) {
      q.set("desc", searchDesc.value.trim());
    }
    const data = await apiFetch<PageResult<Row>>(`/api/exec-scripts?${q}`);
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
  form.value = {
    id: undefined,
    name: "",
    cate: undefined,
    interpreter: "",
    encoding: "utf-8",
    defaultParams: "{}",
    content: "",
    desc: "",
  };
  lastExecStartTime.value = "";
  lastExecEndTime.value = "";
  lastExecParams.value = "";
  execStdin.value = "";
  lastExecStdOut.value = "";
  lastExecErrOut.value = "";
  clearPoll();
}

function fillExecOutputsFromStoredRow(r: Row) {
  lastExecStartTime.value = r.lastExecStartTime
    ? formatDate(String(r.lastExecStartTime))
    : "";
  lastExecEndTime.value = r.lastExecEndTime
    ? formatDate(String(r.lastExecEndTime))
    : "";
  lastExecParams.value = r.lastExecParams ?? "";
  const info = r.lastExecInfo;
  if (info) {
    try {
      const o = JSON.parse(info) as { stdin?: string; outStr?: string; errStr?: string };
      execStdin.value = o.stdin ?? "";
      lastExecStdOut.value = o.outStr ?? "";
      lastExecErrOut.value = o.errStr ?? "";
    } catch {
      execStdin.value = "";
      lastExecStdOut.value = "";
      lastExecErrOut.value = "";
    }
  } else {
    execStdin.value = "";
    lastExecStdOut.value = "";
    lastExecErrOut.value = "";
  }
}

function fillFormFromRow(r: Row) {
  form.value = {
    id: r.id,
    name: r.name,
    cate: r.cate ?? undefined,
    interpreter: r.interpreter,
    encoding: r.encoding || "utf-8",
    defaultParams: r.defaultParams ?? "{}",
    content: r.content ?? "",
    desc: r.desc ?? "",
    lastExecStartTime: r.lastExecStartTime,
    lastExecEndTime: r.lastExecEndTime,
    lastExecParams: r.lastExecParams,
    lastExecInfo: r.lastExecInfo,
  };
  editingId.value = r.id;
  fillExecOutputsFromStoredRow(r);
}

async function openView(row: Row) {
  mode.value = "view";
  try {
    const data = await apiFetch<{ rescript: Row }>(
      `/api/exec-scripts/${row.id}`
    );
    const r = data.rescript;
    fillFormFromRow(r);
    dialogVisible.value = true;
    const running = await apiFetch<boolean>("/api/exec-scripts/is-running", {
      method: "POST",
      body: JSON.stringify({ id: r.id }),
    });
    if (running) {
      schedulePollRunInfo();
    }
  } catch (e) {
    ElMessage.error(errMsg(e));
  }
}

async function openCreate() {
  mode.value = "create";
  resetForm();
  dialogVisible.value = true;
}

async function openEdit(row: Row) {
  mode.value = "edit";
  try {
    const data = await apiFetch<{ rescript: Row }>(
      `/api/exec-scripts/${row.id}`
    );
    const r = data.rescript;
    fillFormFromRow(r);
    dialogVisible.value = true;
  } catch (e) {
    ElMessage.error(errMsg(e));
  }
}

function closeDialog() {
  clearPoll();
  dialogVisible.value = false;
}

function onDialogClosed() {
  resetForm();
}

async function save() {
  const f = form.value;
  if (!f.name.trim()) {
    ElMessage.warning("请填写名称");
    return;
  }
  if (f.cate == null) {
    ElMessage.warning("请选择类型");
    return;
  }
  if (!f.interpreter.trim()) {
    ElMessage.warning("请填写解释器");
    return;
  }
  try {
    JSON.parse(f.defaultParams.trim() || "{}");
  } catch {
    ElMessage.warning("默认参数须为合法 JSON");
    return;
  }
  if (!f.content.trim()) {
    ElMessage.warning("请填写内容");
    return;
  }
  const body = {
    name: f.name.trim(),
    cate: f.cate,
    interpreter: f.interpreter.trim(),
    encoding: f.encoding || "utf-8",
    defaultParams: f.defaultParams.trim() || "{}",
    content: f.content,
    desc: f.desc ?? "",
  };
  try {
    if (mode.value === "create") {
      await apiFetch("/api/exec-scripts", {
        method: "POST",
        body: JSON.stringify(body),
      });
      ElMessage.success("已创建");
    } else if (mode.value === "edit" && editingId.value != null) {
      await apiFetch(`/api/exec-scripts/${editingId.value}`, {
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
    await apiFetch(`/api/exec-scripts/${row.id}`, { method: "DELETE" });
    ElMessage.success("已删除");
    await load();
  } catch (e) {
    if (e === "cancel") return;
    ElMessage.error(errMsg(e));
  }
}

const multipleSelection = ref<Row[]>([]);
function handleSelectionChange(val: Row[]) {
  multipleSelection.value = val;
}

async function batchDelete() {
  if (!multipleSelection.value.length) {
    ElMessage.warning("请选择要删除的数据");
    return;
  }
  try {
    await ElMessageBox.confirm("确定批量删除？", "确认", { type: "warning" });
    const ids = multipleSelection.value.map((r) => r.id);
    await apiFetch("/api/exec-scripts/delete-by-ids", {
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

async function doGetExecScriptRunInfo() {
  if (!dialogVisible.value) return;
  const id = form.value.id ?? 0;
  try {
    const enc = form.value.encoding || "utf-8";
    const rst = await apiFetch<RunInfo>("/api/exec-scripts/run-info", {
      method: "POST",
      body: JSON.stringify({ id, encoding: enc }),
    });
    lastExecParams.value = rst.params;
    lastExecStartTime.value = rst.startTime
      ? formatDate(rst.startTime)
      : "";
    lastExecEndTime.value = rst.endTime ? formatDate(rst.endTime) : "";
    execStdin.value = rst.stdin;
    lastExecStdOut.value = rst.outStr;
    lastExecErrOut.value = rst.errStr;
    if (!rst.finished) {
      pollTimer = setTimeout(doGetExecScriptRunInfo, 2000);
    }
  } catch (e) {
    lastExecErrOut.value = errMsg(e);
  }
}

function schedulePollRunInfo() {
  clearPoll();
  pollTimer = setTimeout(doGetExecScriptRunInfo, 300);
}

async function doRunExecScript() {
  const id = form.value.id ?? 0;
  const cate = form.value.cate;
  if (cate == null) {
    ElMessage.warning("请先选择类型");
    return;
  }
  try {
    await apiFetch("/api/exec-scripts/run", {
      method: "POST",
      body: JSON.stringify({
        id,
        cate,
        interpreter: form.value.interpreter,
        params: form.value.defaultParams || "{}",
        stdin: execStdin.value,
        content: form.value.content,
        encoding: form.value.encoding || "utf-8",
      }),
    });
    lastExecErrOut.value = "";
    schedulePollRunInfo();
  } catch (e) {
    lastExecErrOut.value = errMsg(e);
  }
}

async function execCopy() {
  try {
    await navigator.clipboard.writeText(lastExecStdOut.value);
    ElMessage.success("已复制");
  } catch {
    ElMessage.error("复制失败");
  }
}

async function execPaste() {
  try {
    const t = await navigator.clipboard.readText();
    lastExecStdOut.value = t;
    ElMessage.success("已粘贴");
  } catch {
    ElMessage.error("粘贴失败");
  }
}

watch(dialogVisible, (v) => {
  if (!v) clearPoll();
});

onBeforeUnmount(() => clearPoll());

loadDict();
load();
</script>

<template>
  <div class="exec-script-page">
    <h2 class="page-title">执行脚本</h2>

    <div class="toolbar">
      <el-input
        v-model="searchName"
        placeholder="名称"
        clearable
        style="width: 140px"
        @keyup.enter="load"
      />
      <el-select
        v-model="searchCate"
        placeholder="类型"
        clearable
        style="width: 140px"
      >
        <el-option
          v-for="(item, key) in cateOptions"
          :key="key"
          :label="item.label"
          :value="item.value"
        />
      </el-select>
      <el-input
        v-model="searchInterpreter"
        placeholder="解释器"
        clearable
        style="width: 140px"
        @keyup.enter="load"
      />
      <el-input
        v-model="searchContent"
        placeholder="内容"
        clearable
        style="width: 140px"
        @keyup.enter="load"
      />
      <el-input
        v-model="searchDesc"
        placeholder="描述"
        clearable
        style="width: 140px"
        @keyup.enter="load"
      />
      <el-button type="primary" @click="() => { page = 1; load(); }">
        查询
      </el-button>
      <el-button @click="searchName = ''; searchCate = null; searchInterpreter = ''; searchContent = ''; searchDesc = ''; page = 1; load();">
        重置
      </el-button>
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
      <el-table-column prop="name" label="名称" min-width="120" show-overflow-tooltip />
      <el-table-column label="类型" width="90">
        <template #default="{ row }">
          {{ dictLabel(row.cate, cateOptions) }}
        </template>
      </el-table-column>
      <el-table-column prop="interpreter" label="解释器" width="100" />
      <el-table-column prop="encoding" label="编码" width="80" />
      <el-table-column prop="desc" label="描述" min-width="140" show-overflow-tooltip />
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
      @size-change="() => { page = 1; load(); }"
    />

    <el-dialog
      v-model="dialogVisible"
      :title="dialogTitle"
      width="70%"
      destroy-on-close
      class="gva-dialog"
      @closed="onDialogClosed"
    >
      <el-form label-width="140px" label-position="right">
        <el-form-item label="名称">
          <el-input v-model="form.name" :readonly="readonlyFields" clearable />
        </el-form-item>
        <el-form-item label="类型">
          <el-select
            v-model="form.cate"
            placeholder="请选择"
            style="width: 100%"
            clearable
            :disabled="readonlyFields"
          >
            <el-option
              v-for="(item, key) in cateOptions"
              :key="key"
              :label="item.label"
              :value="item.value"
            />
          </el-select>
        </el-form-item>
        <el-form-item label="解释器">
          <el-input v-model="form.interpreter" :readonly="readonlyFields" clearable />
        </el-form-item>
        <el-form-item label="编码">
          <el-select
            v-model="form.encoding"
            placeholder="请选择"
            style="width: 100%"
            :disabled="readonlyFields"
          >
            <el-option
              v-for="item in encodingOptions"
              :key="item.value"
              :label="item.label"
              :value="item.value"
            />
          </el-select>
        </el-form-item>
        <el-form-item label="默认参数 (JSON)" class="params-item">
          <CodeMirrorPane
            v-if="dialogVisible"
            :key="`default-params-${editingId ?? 'n'}-${mode}`"
            v-model="form.defaultParams"
            language="javascript"
            :readonly="readonlyFields"
            height="100px"
          />
        </el-form-item>
        <el-form-item label="内容" class="content-item">
          <CodeMirrorPane
            v-if="dialogVisible"
            :key="`content-${editingId}-${mode}`"
            v-model="form.content"
            language="shell"
            :readonly="readonlyFields"
            height="200px"
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
        <el-form-item label="最近执行开始">
          <span class="muted">{{ lastExecStartTime }}</span>
        </el-form-item>
        <el-form-item label="最近执行结束">
          <span class="muted">{{ lastExecEndTime }}</span>
        </el-form-item>
        <el-form-item label="最近执行参数">
          <template v-if="lastExecParams">
            <el-tooltip
              :disabled="!lastExecParamsOverflow"
              :content="lastExecParams"
              placement="top"
              :show-after="200"
            >
              <span
                class="muted exec-params-preview"
                :class="{ 'is-truncated': lastExecParamsOverflow }"
              >
                {{ lastExecParamsPreview }}
              </span>
            </el-tooltip>
          </template>
          <span v-else class="muted">—</span>
        </el-form-item>
        <el-form-item label="stdout">
          <CodeMirrorPane
            v-if="dialogVisible"
            v-model="lastExecStdOut"
            language="shell"
            readonly
            height="160px"
          />
          <div class="run-actions">
            <el-button type="primary" size="small" @click="doRunExecScript">运行</el-button>
            <el-button size="small" @click="execCopy">复制</el-button>
            <el-button size="small" @click="execPaste">粘贴</el-button>
          </div>
        </el-form-item>
        <el-form-item label="stderr">
          <CodeMirrorPane
            v-if="dialogVisible"
            v-model="lastExecErrOut"
            language="shell"
            readonly
            height="120px"
          />
        </el-form-item>
        <el-form-item label="stdin">
          <CodeMirrorPane
            v-if="dialogVisible"
            v-model="execStdin"
            language="shell"
            :readonly="readonlyFields"
            height="80px"
          />
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
.exec-script-page {
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
.muted {
  color: var(--el-text-color-secondary);
  font-size: 13px;
  word-break: break-all;
}
.exec-params-preview {
  display: inline-block;
  max-width: 100%;
  line-height: 1.5;
  vertical-align: top;
}
.exec-params-preview.is-truncated {
  cursor: help;
}
.run-actions {
  margin-top: 8px;
  display: flex;
  gap: 8px;
}
.content-item :deep(.el-form-item__content),
.params-item :deep(.el-form-item__content) {
  min-width: 0;
}
</style>

<style>
.gva-dialog .el-dialog__body {
  padding-top: 8px;
  overflow-x: hidden;
}
</style>
