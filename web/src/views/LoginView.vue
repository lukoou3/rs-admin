<script setup lang="ts">
import { reactive, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import { ElMessage } from "element-plus";
import { apiFetch, setToken, ApiError } from "@/api/http";

const router = useRouter();
const route = useRoute();

const loading = ref(false);
const form = reactive({
  username: "",
  password: "",
});

function errMsg(e: unknown) {
  if (e instanceof ApiError) return e.message;
  if (e instanceof Error) return e.message;
  return String(e);
}

async function submit() {
  const u = form.username.trim();
  if (!u || !form.password) {
    ElMessage.warning("请输入用户名和密码");
    return;
  }
  loading.value = true;
  try {
    const data = await apiFetch<{
      token: string;
      expiresAt: number;
      user: { id: number; userName: string; nickName: string };
    }>("/api/auth/login", {
      method: "POST",
      body: JSON.stringify({ username: u, password: form.password }),
    });
    setToken(data.token);
    const redir =
      typeof route.query.redirect === "string"
        ? route.query.redirect
        : "/shell";
    router.replace(redir || "/shell");
  } catch (e) {
    ElMessage.error(errMsg(e));
  } finally {
    loading.value = false;
  }
}
</script>

<template>
  <div class="login-page">
    <div class="card">
      <h1 class="title">登录</h1>
      <p class="hint">rs-admin · 用户名与密码</p>
      <el-form label-position="top" @submit.prevent="submit">
        <el-form-item label="用户名">
          <el-input
            v-model="form.username"
            autocomplete="username"
            placeholder="用户名"
            @keyup.enter="submit"
          />
        </el-form-item>
        <el-form-item label="密码">
          <el-input
            v-model="form.password"
            type="password"
            autocomplete="current-password"
            placeholder="密码"
            show-password
            @keyup.enter="submit"
          />
        </el-form-item>
        <el-button
          type="primary"
          class="btn"
          :loading="loading"
          native-type="submit"
        >
          登录
        </el-button>
      </el-form>
    </div>
  </div>
</template>

<style scoped>
.login-page {
  min-height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
  background: linear-gradient(
    160deg,
    var(--el-fill-color-light) 0%,
    var(--el-bg-color-page) 45%
  );
}
.card {
  width: 100%;
  max-width: 400px;
  padding: 28px 28px 32px;
  border-radius: 12px;
  background: var(--el-bg-color);
  box-shadow: var(--el-box-shadow-light);
  border: 1px solid var(--el-border-color-lighter);
}
.title {
  margin: 0 0 8px;
  font-size: 22px;
  font-weight: 600;
  color: var(--el-text-color-primary);
}
.hint {
  margin: 0 0 24px;
  font-size: 13px;
  color: var(--el-text-color-secondary);
}
.btn {
  width: 100%;
  margin-top: 8px;
}
</style>
