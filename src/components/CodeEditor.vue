<script setup lang="ts">
import * as monaco from "monaco-editor/esm/vs/editor/editor.api";
import { computed, ref, shallowRef, watch } from "vue";
import { useDebounceFn, useElementSize } from "@vueuse/core";

const monacoMount = ref<HTMLDivElement | null>(null);
const props = defineProps<{
  startCode: string;
  isDark: boolean;
}>();
const emit = defineEmits<{ update: [code: () => string] }>();

const themeName = computed(() => (props.isDark ? "vs-dark" : "vs"));

const editor = shallowRef<monaco.editor.IStandaloneCodeEditor | null>(null);

const { width, height } = useElementSize(monacoMount);
watch(
  [width, height],
  useDebounceFn(() => {
    editor.value?.layout();
  }, 100)
);

watch(
  () => props.startCode,
  () => {
    editor.value?.setValue(props.startCode);
  }
);

watch(monacoMount, (element) => {
  if (!element) return;
  editor.value = monaco.editor.create(element, {
    value: props.startCode,
    language: "wgsl",
    contextmenu: true,
    minimap: {
      enabled: false,
    },
    theme: themeName.value,
  });

  editor.value.onDidChangeModelContent((e) => {
    emit("update", () => editor.value?.getValue() ?? "");
  });
});

watch(themeName, (v) => {
  monaco.editor.setTheme(v);
});
</script>
<template>
  <div ref="monacoMount" class="border border-gray-500"></div>
</template>
<style scoped></style>
