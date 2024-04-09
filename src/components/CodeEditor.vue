<script setup lang="ts">
import * as monaco from "monaco-editor/esm/vs/editor/editor.api";
import { computed, ref, shallowRef, watch } from "vue";
import { useDebounceFn, useElementSize } from "@vueuse/core";
import type { FilePath } from "@/filesystem/scene-files";

const monacoMount = ref<HTMLDivElement | null>(null);

export interface KeyedCode {
  readonly id: string;
  readonly code: string;
  readonly name: FilePath;
}

const props = defineProps<{
  keyedCode: KeyedCode | null;
  isDark: boolean;
}>();
const emit = defineEmits<{ update: [code: () => string] }>();

const editor = shallowRef<monaco.editor.IStandaloneCodeEditor | null>(null);

const { width, height } = useElementSize(monacoMount);
watch(
  [width, height],
  useDebounceFn(() => {
    editor.value?.layout();
  }, 100)
);

let surpressChange = false;
watch(
  () => props.keyedCode?.id,
  () => {
    surpressChange = true;
    editor.value?.setValue(props.keyedCode?.code ?? "<no code>");
    surpressChange = false;
  }
);

const isReadonly = computed(() => props.keyedCode === null);
watch(isReadonly, (v) => {
  editor.value?.updateOptions({ readOnly: v });
});

const themeName = computed(() => (props.isDark ? "vs-dark" : "vs"));
watch(themeName, (v) => {
  monaco.editor.setTheme(v);
});

watch(monacoMount, (element) => {
  if (!element) return;
  editor.value = monaco.editor.create(element, {
    value: props.keyedCode?.code ?? "<no code>",
    language: "wgsl",
    contextmenu: true,
    minimap: {
      enabled: false,
    },
    theme: themeName.value,
    readOnly: isReadonly.value,
  });

  editor.value.onDidChangeModelContent((e) => {
    if (surpressChange) return;
    emit("update", () => editor.value?.getValue() ?? "");
  });
});
</script>
<template>
  <div class="flex flex-col">
    <h2 class="border border-gray-500 border-b-0 px-2">
      {{ props.keyedCode?.name ?? "No file opened" }}
    </h2>
    <div
      ref="monacoMount"
      class="border border-gray-500 self-stretch flex-1 overflow-hidden"
      :class="{ 'bg-gray-800': isReadonly }"
    ></div>
  </div>
</template>
<style scoped></style>
