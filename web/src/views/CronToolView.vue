<script setup lang="ts">
/**
 * 对齐 gin `view/tools/cron/cron.vue`，Cron 表达式生成（含复制）。
 */
import Crontab from "@/components/Crontab/index.vue";
import { ElMessage } from "element-plus";
import { ref } from "vue";

const cronExpression = ref("0 0 0 * * ?");

async function onFill(crontabValue: string) {
  try {
    await navigator.clipboard.writeText(crontabValue);
    ElMessage.success("已复制 Cron 表达式");
  } catch {
    ElMessage.error("复制失败");
  }
}
</script>

<template>
  <div class="cron-tool-page">
    <h2 class="page-title">Cron 表达式</h2>
    <p class="hint">
      可视化生成 Quartz 风格 Cron；点击「确定」复制表达式到剪贴板。
    </p>
    <Crontab activetab="hour" :expression="cronExpression" @fill="onFill" />
  </div>
</template>

<style scoped>
.cron-tool-page {
  width: 100%;
}
.page-title {
  margin: 0 0 8px;
  font-size: 18px;
  font-weight: 600;
}
.hint {
  margin: 0 0 16px;
  font-size: 13px;
  color: var(--el-text-color-secondary);
}
</style>
