<script setup lang="ts">
import { useRoute } from "vue-router";

const route = useRoute();
</script>

<template>
  <el-container class="layout">
    <el-aside class="sidebar" width="220px">
      <header class="brand">
        <div class="brand-title">note</div>
        <div class="brand-sub">rs-admin</div>
      </header>
      <el-menu
        class="sidebar-menu"
        :default-active="route.path"
        :unique-opened="true"
        router
      >
        <el-sub-menu index="code">
          <template #title>
            <span class="submenu-title">代码</span>
          </template>
          <el-menu-item index="/shell">命令</el-menu-item>
          <el-menu-item index="/exec-script">执行脚本</el-menu-item>
        </el-sub-menu>
        <el-sub-menu index="sql">
          <template #title>
            <span class="submenu-title">SQL</span>
          </template>
          <el-menu-item index="/datasource">数据源</el-menu-item>
          <el-menu-item index="/query">SQL 查询</el-menu-item>
          <el-menu-item index="/sql-format">SQL 格式化</el-menu-item>
        </el-sub-menu>
        <el-sub-menu index="tools">
          <template #title>
            <span class="submenu-title">工具</span>
          </template>
          <el-menu-item index="/tools/cron">Cron 表达式</el-menu-item>
          <el-menu-item index="/tools/clear-delete-data">清除软删数据</el-menu-item>
          <el-menu-item index="/tools/html2md">HTML 转 MD</el-menu-item>
        </el-sub-menu>
        <el-sub-menu index="system">
          <template #title>
            <span class="submenu-title">系统</span>
          </template>
          <el-menu-item index="/users">用户管理</el-menu-item>
          <el-menu-item index="/dictionary">字典管理</el-menu-item>
          <el-menu-item index="/operation-history">操作历史</el-menu-item>
        </el-sub-menu>
      </el-menu>
    </el-aside>
    <el-main class="main-area">
      <router-view />
    </el-main>
  </el-container>
</template>

<style scoped>
.layout {
  min-height: 100%;
  background: var(--el-bg-color-page);
}
.sidebar {
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
  background: var(--el-fill-color-blank);
  border-right: 1px solid var(--el-border-color-lighter);
  box-shadow: 4px 0 24px rgba(0, 0, 0, 0.04);
}
.brand {
  flex-shrink: 0;
  padding: 22px 20px 20px;
  border-bottom: 1px solid var(--el-border-color-lighter);
  background: linear-gradient(
    165deg,
    var(--el-fill-color-light) 0%,
    var(--el-fill-color-blank) 100%
  );
}
.brand-title {
  font-size: 18px;
  font-weight: 600;
  letter-spacing: 0.04em;
  color: var(--el-text-color-primary);
  line-height: 1.3;
}
.brand-sub {
  margin-top: 6px;
  font-size: 12px;
  font-weight: 500;
  letter-spacing: 0.04em;
  color: var(--el-text-color-secondary);
  opacity: 0.9;
}
.sidebar-menu {
  flex: 1;
  border-right: none !important;
  padding: 12px 10px 20px;
  --el-menu-bg-color: transparent;
  --el-menu-hover-bg-color: var(--el-fill-color-light);
  --el-menu-active-color: var(--el-color-primary);
}
.sidebar-menu.el-menu {
  background-color: transparent !important;
}
.sidebar .submenu-title {
  font-weight: 600;
  font-size: 13px;
  letter-spacing: 0.02em;
}
.sidebar :deep(.el-sub-menu__title) {
  border-radius: 8px;
  margin: 3px 0;
  height: 42px;
}
.sidebar :deep(.el-menu-item) {
  border-radius: 8px;
  margin: 2px 0;
  height: 38px;
}
.sidebar :deep(.el-menu--vertical.el-menu) {
  border-right: none;
}
.sidebar :deep(.el-sub-menu .el-menu-item) {
  padding-left: 16px !important;
  min-width: auto;
}
.sidebar :deep(.el-menu-item.is-active) {
  font-weight: 600;
  background-color: var(--el-color-primary-light-9) !important;
}
.main-area {
  padding: 20px 24px 24px;
  background: var(--el-bg-color);
  min-width: 0;
}
@media (max-width: 768px) {
  .main-area {
    padding: 16px;
  }
}
</style>
