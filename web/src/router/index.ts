import { createRouter, createWebHistory } from "vue-router";
import { getToken } from "@/api/http";

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: "/login",
      name: "login",
      component: () => import("@/views/LoginView.vue"),
      meta: { public: true },
    },
    {
      path: "/",
      component: () => import("@/layouts/MainLayout.vue"),
      meta: { requiresAuth: true },
      children: [
        { path: "", redirect: "/shell" },
        {
          path: "shell",
          component: () => import("@/views/ShellcodesView.vue"),
        },
        {
          path: "exec-script",
          component: () => import("@/views/ExecScriptView.vue"),
        },
        {
          path: "code-template",
          component: () => import("@/views/CodeTemplateView.vue"),
        },
        { path: "script", redirect: "/exec-script" },
        {
          path: "datasource",
          component: () => import("@/views/DatasourcesView.vue"),
        },
        {
          path: "query",
          component: () => import("@/views/QuerySqlView.vue"),
        },
        {
          path: "sql-format",
          component: () => import("@/views/SqlFormatView.vue"),
        },
        {
          path: "tools/cron",
          component: () => import("@/views/CronToolView.vue"),
        },
        {
          path: "tools/clear-delete-data",
          component: () => import("@/views/ClearDeleteDataView.vue"),
        },
        {
          path: "tools/html2md",
          component: () => import("@/views/Html2mdView.vue"),
        },
        {
          path: "users",
          component: () => import("@/views/UsersView.vue"),
        },
        {
          path: "dictionary",
          name: "dictionaryManage",
          component: () => import("@/views/DictionaryManageView.vue"),
        },
        {
          path: "dictionary/:id",
          name: "dictionaryDetail",
          component: () => import("@/views/DictionaryDetailView.vue"),
        },
        {
          path: "operation-history",
          component: () => import("@/views/OperationHistoryView.vue"),
        },
      ],
    },
  ],
});

router.beforeEach((to, _from, next) => {
  const token = getToken();
  if (to.meta.public) {
    if (token && to.path === "/login") {
      next("/shell");
      return;
    }
    next();
    return;
  }
  if (!token) {
    next({
      path: "/login",
      query: { redirect: to.fullPath !== "/" ? to.fullPath : undefined },
    });
    return;
  }
  next();
});

export default router;
