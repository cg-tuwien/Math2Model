<script setup lang="ts">
import * as monaco from "monaco-editor";
import { ref, shallowRef, watch } from "vue";
import { useElementSize } from "@vueuse/core";
import debounce from "debounce";

const monacoMount = ref<HTMLDivElement | null>(null);
const props = defineProps<{ startCode: string }>();
const emit = defineEmits<{ update: [code: () => string] }>();

const editor = shallowRef<monaco.editor.IStandaloneCodeEditor | null>(null);

const { width, height } = useElementSize(monacoMount);
watch(
  [width, height],
  debounce(() => {
    editor.value?.layout();
  }, 100)
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
  });

  editor.value.onDidChangeModelContent((e) => {
    emit("update", () => editor.value?.getValue() ?? "");
  });
});
</script>
<template>
  <div ref="monacoMount" class="border border-gray-500"></div>
</template>
<style scoped></style>
