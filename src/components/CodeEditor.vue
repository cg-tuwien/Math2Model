<script setup lang="ts">
import * as monaco from "monaco-editor/esm/vs/editor/editor.api";
import { computed, ref, shallowRef, watch, type DeepReadonly } from "vue";
import { watchDebounced, useElementSize } from "@vueuse/core";
import type { FilePath } from "@/filesystem/reactive-files";
import { showInfo } from "@/notification";

export type Marker = monaco.editor.IMarkerData;

const monacoMount = ref<HTMLDivElement | null>(null);

export interface KeyedCode {
  readonly id: string;
  readonly code: string;
  readonly name: FilePath;
}

const props = defineProps<{
  keyedCode: DeepReadonly<KeyedCode> | null;
  isDark: boolean;
  markers: monaco.editor.IMarkerData[];
  isReadonly: boolean;
}>();
const emit = defineEmits<{ update: [code: () => string]; graph: [] }>();

const editor = shallowRef<monaco.editor.IStandaloneCodeEditor | null>(null);

const { width, height } = useElementSize(monacoMount);
watchDebounced([width, height], () => editor.value?.layout(), {
  debounce: 100,
});

let surpressChange = false;
watch(
  () => props.keyedCode?.id,
  () => {
    surpressChange = true;
    editor.value?.setValue(props.keyedCode?.code ?? "<no code>");
    surpressChange = false;
  }
);

const isReadonly = computed(() => props.keyedCode === null || props.isReadonly);
watch(isReadonly, (v) => {
  editor.value?.updateOptions({ readOnly: v });
});

const themeName = computed(() => (props.isDark ? "vs-dark" : "vs"));
watch(themeName, (v) => {
  monaco.editor.setTheme(v);
});

watch(
  () => props.markers,
  (v) => {
    const model = editor.value?.getModel();
    if (!model) return;
    monaco.editor.setModelMarkers(model, "owner", v);
  },
  { deep: true }
);

watch(
  () => props.keyedCode?.name,
  (v) => {
    const model = editor.value?.getModel();
    if (!model) return;
    monaco.editor.setModelLanguage(model, guessLanguage(v ?? ""));
  }
);

function guessLanguage(name: string): "wgsl" | "json" {
  // pop off the last extension
  const ext = (name.match(/^[^]+\.(\w+)$/)?.[1] ?? "").toLowerCase();
  if (ext === "wgsl") return "wgsl";
  if (ext === "json") return "json";

  return "wgsl";
}

watch(monacoMount, (element) => {
  if (!element) return;
  editor.value = monaco.editor.create(element, {
    value: props.keyedCode?.code ?? "<no code>",
    language: guessLanguage(props.keyedCode?.name ?? ""),
    contextmenu: true,
    minimap: {
      enabled: false,
    },
    theme: themeName.value,
    readOnly: isReadonly.value,
  });

  editor.value.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, () => {
    showInfo("You don't need to save!");
  });

  editor.value.onDidChangeModelContent((e) => {
    if (surpressChange) return;
    emit("update", () => editor.value?.getValue() ?? "");
  });
});
</script>
<template>
  <div class="flex self-stretch flex-1 relative">
    <div
      ref="monacoMount"
      class="border border-gray-500 self-stretch flex-1 overflow-hidden"
      :class="{ 'bg-gray-800': isReadonly }"
    ></div>
    <div
      class="absolute top-2 right-2"
      v-if="props.keyedCode?.name.includes('.graph')"
    >
      <n-button
        quaternary
        circle
        type="primary"
        @click="emit('graph')"
        class="text-xl"
      >
        <template #icon>
          <n-icon size="32px"><mdi-transit-connection-variant /></n-icon>
        </template>
      </n-button>
    </div>
  </div>
</template>
<style scoped></style>
