<script setup lang="ts">
import { javascript } from "@codemirror/lang-javascript";
import { sql } from "@codemirror/lang-sql";
import { StreamLanguage } from "@codemirror/language";
import { shell } from "@codemirror/legacy-modes/mode/shell";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { dracula } from "@uiw/codemirror-theme-dracula";
import { basicSetup } from "codemirror";
import {
  onBeforeUnmount,
  onMounted,
  ref,
  shallowRef,
  watch,
} from "vue";

const props = withDefaults(
  defineProps<{
    modelValue: string;
    /** shell：脚本内容；javascript：默认参数 JSON（对齐 gin execScript 的 codemirror） */
    language: "sql" | "shell" | "javascript";
    readonly?: boolean;
    /** 与旧 gin-vue-admin 一致，默认 400px */
    height?: string;
  }>(),
  { readonly: false, height: "400px" }
);

const emit = defineEmits<{ "update:modelValue": [v: string] }>();

const host = ref<HTMLElement | null>(null);
const view = shallowRef<EditorView | null>(null);

const shellSupport = StreamLanguage.define(shell);

function buildExtensions() {
  const lang =
    props.language === "sql"
      ? sql()
      : props.language === "javascript"
        ? javascript()
        : shellSupport;
  // 与旧 gin-vue-admin format/querySql 一致：`[sql(), dracula]`，此处保留 basicSetup（行号等）
  return [
    basicSetup,
    lang,
    dracula,
    EditorView.theme({
      "&": {
        height: "100%",
        width: "100%",
        maxWidth: "100%",
        minWidth: "0",
        boxSizing: "border-box",
      },
      ".cm-scroller": {
        overflow: "auto",
        fontFamily: "Consolas, 'Courier New', monospace",
        fontSize: "13px",
      },
      ".cm-content": {
        minWidth: "0",
        padding: "8px 0",
      },
    }),
    EditorView.updateListener.of((u) => {
      if (u.docChanged && !props.readonly)
        emit("update:modelValue", u.state.doc.toString());
    }),
    ...(props.readonly ? [EditorState.readOnly.of(true)] : []),
  ];
}

function createEditor() {
  view.value?.destroy();
  view.value = null;
  if (!host.value) return;
  const state = EditorState.create({
    doc: props.modelValue,
    extensions: buildExtensions(),
  });
  view.value = new EditorView({ state, parent: host.value });
}

onMounted(() => {
  createEditor();
});

onBeforeUnmount(() => {
  view.value?.destroy();
  view.value = null;
});

watch(
  () => [props.language, props.readonly],
  () => {
    createEditor();
  }
);

watch(
  () => props.modelValue,
  (v) => {
    const ed = view.value;
    if (!ed) return;
    const cur = ed.state.doc.toString();
    if (v !== cur) {
      ed.dispatch({
        changes: { from: 0, to: ed.state.doc.length, insert: v },
      });
    }
  }
);
</script>

<template>
  <div
    ref="host"
    class="cm-host"
    :style="{ height: props.height, width: '100%' }"
  />
</template>

<style scoped>
/* 与旧项目 codemirror :style width 100% 一致，避免长行把弹窗撑开 */
.cm-host {
  width: 100%;
  max-width: 100%;
  min-width: 0;
  box-sizing: border-box;
  border: 1px solid var(--el-border-color);
  border-radius: 4px;
  overflow: hidden;
}
.cm-host :deep(.cm-editor) {
  width: 100% !important;
  max-width: 100%;
  min-width: 0;
  height: 100%;
  outline: none;
}
.cm-host :deep(.cm-editor.cm-focused) {
  outline: none;
}
.cm-host :deep(.cm-scroller) {
  max-width: 100%;
}
</style>
