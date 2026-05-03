<script setup lang="ts">
import { useRoute } from "vue-router";

const route = useRoute();
</script>

<template>
  <el-container class="layout">
    <el-aside class="sidebar" width="232px">
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
  background: linear-gradient(180deg, #f8fafc 0%, #f3f4f6 100%);
}
.sidebar {
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
  background: linear-gradient(180deg, #f8fafc 0%, #eef2f7 100%);
  border-right: 1px solid #d7dde7;
  box-shadow: 6px 0 22px rgba(15, 23, 42, 0.06);
  position: relative;
  z-index: 1;
}
.brand {
  flex-shrink: 0;
  padding: 22px 20px 18px;
  border-bottom: 1px solid rgba(148, 163, 184, 0.25);
  background: linear-gradient(165deg, rgba(255, 255, 255, 0.9) 0%, rgba(241, 245, 249, 0.92) 100%);
}
.brand-title {
  font-size: 18px;
  font-weight: 700;
  letter-spacing: 0.02em;
  color: #0f172a;
  line-height: 1.3;
}
.brand-sub {
  margin-top: 6px;
  font-size: 12px;
  font-weight: 500;
  letter-spacing: 0.02em;
  color: #64748b;
}
.sidebar-menu {
  flex: 1;
  border-right: none !important;
  padding: 12px 10px 20px;
  --el-menu-bg-color: transparent;
  --el-menu-hover-bg-color: rgba(59, 130, 246, 0.08);
  --el-menu-active-color: #1d4ed8;
}
.sidebar-menu.el-menu {
  background-color: transparent !important;
}
.sidebar .submenu-title {
  font-weight: 600;
  font-size: 13px;
  letter-spacing: 0.01em;
  color: #334155;
}
.sidebar :deep(.el-sub-menu__title) {
  border-radius: 10px;
  margin: 4px 0;
  height: 42px;
  color: #334155;
  font-weight: 600;
}
.sidebar :deep(.el-sub-menu__title:hover) {
  background-color: rgba(59, 130, 246, 0.08);
}
.sidebar :deep(.el-menu-item) {
  border-radius: 10px;
  margin: 2px 0;
  height: 38px;
  color: #475569;
}
.sidebar :deep(.el-menu--vertical.el-menu) {
  border-right: none;
}
.sidebar :deep(.el-sub-menu .el-menu-item) {
  padding-left: 16px !important;
  min-width: auto;
}
.sidebar :deep(.el-menu-item.is-active) {
  font-weight: 700;
  color: #1d4ed8 !important;
  background-color: rgba(59, 130, 246, 0.12) !important;
  box-shadow: inset 3px 0 0 #1d4ed8;
}
.sidebar :deep(.el-menu-item:hover) {
  color: #1d4ed8;
  background-color: rgba(59, 130, 246, 0.08);
}
.main-area {
  padding: 20px 24px 24px;
  background: #f8fafc;
  min-width: 0;
}
@media (max-width: 768px) {
  .main-area {
    padding: 16px;
  }
}
</style>
