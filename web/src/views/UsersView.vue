<script setup lang="ts">
import { onMounted, reactive, ref } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import { ApiError, apiFetch, type PageResult } from "@/api/http";
import { formatDate } from "@/utils/formatDate";

interface Row {
  id: number;
  uuid: string;
  userName: string;
  nickName: string;
  phone: string;
  email: string;
  enable: number;
  authorityId: number;
  createdAt?: string;
  updatedAt?: string;
}

const loading = ref(false);
const rows = ref<Row[]>([]);
const total = ref(0);
const page = ref(1);
const pageSize = ref(20);
const keyword = ref("");

const dialogVisible = ref(false);
const dialogCreate = ref(false);
const editingId = ref<number | null>(null);

const createForm = reactive({
  userName: "",
  password: "",
  nickName: "",
  phone: "",
  email: "",
});

const editForm = reactive({
  nickName: "",
  phone: "",
  email: "",
  enable: 1,
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
    const data = await apiFetch<PageResult<Row>>(`/api/users?${q}`);
    rows.value = data.list;
    total.value = data.total;
  } catch (e) {
    ElMessage.error(errMsg(e));
  } finally {
    loading.value = false;
  }
}

function openCreate() {
  dialogCreate.value = true;
  editingId.value = null;
  createForm.userName = "";
  createForm.password = "";
  createForm.nickName = "";
  createForm.phone = "";
  createForm.email = "";
  dialogVisible.value = true;
}

function openEdit(row: Row) {
  dialogCreate.value = false;
  editingId.value = row.id;
  editForm.nickName = row.nickName ?? "";
  editForm.phone = row.phone ?? "";
  editForm.email = row.email ?? "";
  editForm.enable = row.enable === 2 ? 2 : 1;
  dialogVisible.value = true;
}

async function saveCreate() {
  if (!createForm.userName.trim() || !createForm.password) {
    ElMessage.warning("用户名和密码必填");
    return;
  }
  try {
    await apiFetch<{ id: number }>("/api/users", {
      method: "POST",
      body: JSON.stringify({
        userName: createForm.userName.trim(),
        password: createForm.password,
        nickName: createForm.nickName.trim(),
        phone: createForm.phone.trim(),
        email: createForm.email.trim(),
      }),
    });
    ElMessage.success("已创建");
    dialogVisible.value = false;
    await load();
  } catch (e) {
    ElMessage.error(errMsg(e));
  }
}

async function saveEdit() {
  if (editingId.value == null) return;
  try {
    await apiFetch(`/api/users/${editingId.value}`, {
      method: "PUT",
      body: JSON.stringify({
        nickName: editForm.nickName.trim(),
        phone: editForm.phone.trim(),
        email: editForm.email.trim(),
        enable: editForm.enable,
      }),
    });
    ElMessage.success("已保存");
    dialogVisible.value = false;
    await load();
  } catch (e) {
    ElMessage.error(errMsg(e));
  }
}

async function removeRow(row: Row) {
  try {
    await ElMessageBox.confirm(`确定删除用户「${row.userName}」？`, "确认", {
      type: "warning",
    });
    await apiFetch(`/api/users/${row.id}`, { method: "DELETE" });
    ElMessage.success("已删除");
    await load();
  } catch (e) {
    if (e === "cancel") return;
    ElMessage.error(errMsg(e));
  }
}

async function resetPwd(row: Row) {
  try {
    const { value } = await ElMessageBox.prompt("请输入新密码", "重置密码", {
      confirmButtonText: "确定",
      cancelButtonText: "取消",
      inputType: "password",
      inputPlaceholder: "新密码",
    });
    if (!value?.trim()) {
      ElMessage.warning("密码不能为空");
      return;
    }
    await apiFetch(`/api/users/${row.id}/password`, {
      method: "PUT",
      body: JSON.stringify({ newPassword: value.trim() }),
    });
    ElMessage.success("密码已更新");
  } catch (e) {
    if (e === "cancel") return;
    ElMessage.error(errMsg(e));
  }
}

onMounted(load);
</script>

<template>
  <div class="page-head">
    <h2 class="page-title">用户管理</h2>
    <div class="toolbar">
      <el-input
        v-model="keyword"
        clearable
        placeholder="搜索用户名、昵称、手机、邮箱"
        style="width: 280px"
        @clear="load"
        @keyup.enter="load"
      />
      <el-button type="primary" @click="load">查询</el-button>
      <el-button type="success" @click="openCreate">新增用户</el-button>
    </div>
  </div>

  <el-table v-loading="loading" :data="rows" stripe border style="width: 100%">
    <el-table-column prop="id" label="ID" width="72" />
    <el-table-column prop="userName" label="用户名" min-width="120" />
    <el-table-column prop="nickName" label="昵称" min-width="120" />
    <el-table-column prop="phone" label="手机" width="130" />
    <el-table-column prop="email" label="邮箱" min-width="160" show-overflow-tooltip />
    <el-table-column label="状态" width="88">
      <template #default="{ row }">
        <el-tag :type="row.enable === 1 ? 'success' : 'info'" size="small">
          {{ row.enable === 1 ? "正常" : "冻结" }}
        </el-tag>
      </template>
    </el-table-column>
    <el-table-column label="创建时间" width="170">
      <template #default="{ row }">
        {{ formatDate(row.createdAt) }}
      </template>
    </el-table-column>
    <el-table-column label="操作" width="220" fixed="right">
      <template #default="{ row }">
        <el-button link type="primary" @click="openEdit(row)">编辑</el-button>
        <el-button link type="warning" @click="resetPwd(row)">重置密码</el-button>
        <el-button link type="danger" @click="removeRow(row)">删除</el-button>
      </template>
    </el-table-column>
  </el-table>

  <div class="pager">
    <el-pagination
      v-model:current-page="page"
      v-model:page-size="pageSize"
      :total="total"
      :page-sizes="[10, 20, 50]"
      layout="total, sizes, prev, pager, next"
      background
      @current-change="load"
      @size-change="load"
    />
  </div>

  <el-dialog
    v-model="dialogVisible"
    :title="dialogCreate ? '新增用户' : '编辑用户'"
    width="480px"
    destroy-on-close
  >
    <template v-if="dialogCreate">
      <el-form label-width="88px">
        <el-form-item label="用户名" required>
          <el-input v-model="createForm.userName" autocomplete="off" />
        </el-form-item>
        <el-form-item label="密码" required>
          <el-input
            v-model="createForm.password"
            type="password"
            show-password
            autocomplete="new-password"
          />
        </el-form-item>
        <el-form-item label="昵称">
          <el-input v-model="createForm.nickName" />
        </el-form-item>
        <el-form-item label="手机">
          <el-input v-model="createForm.phone" />
        </el-form-item>
        <el-form-item label="邮箱">
          <el-input v-model="createForm.email" />
        </el-form-item>
      </el-form>
    </template>
    <template v-else>
      <el-form label-width="88px">
        <el-form-item label="昵称">
          <el-input v-model="editForm.nickName" />
        </el-form-item>
        <el-form-item label="手机">
          <el-input v-model="editForm.phone" />
        </el-form-item>
        <el-form-item label="邮箱">
          <el-input v-model="editForm.email" />
        </el-form-item>
        <el-form-item label="状态">
          <el-radio-group v-model="editForm.enable">
            <el-radio :value="1">正常</el-radio>
            <el-radio :value="2">冻结</el-radio>
          </el-radio-group>
        </el-form-item>
      </el-form>
    </template>
    <template #footer>
      <el-button @click="dialogVisible = false">取消</el-button>
      <el-button type="primary" @click="dialogCreate ? saveCreate() : saveEdit()">
        确定
      </el-button>
    </template>
  </el-dialog>
</template>

<style scoped>
.page-head {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  margin-bottom: 16px;
}
.page-title {
  margin: 0;
  font-size: 20px;
  font-weight: 600;
}
.toolbar {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  align-items: center;
}
.pager {
  margin-top: 16px;
  display: flex;
  justify-content: flex-end;
}
</style>
