<script setup lang="ts">
import type { WebGPUEngine } from "@babylonjs/core";
import { MyFirstScene } from "@/scenes/MyFirstScene";
import { BaseScene } from "@/scenes/BaseScene";
import CodeEditor from "@/components/CodeEditor.vue";

import {
  ref,
  shallowRef,
  watch,
  watchEffect,
  onUnmounted,
  computed,
} from "vue";
import { useDebounceFn, useElementSize } from "@vueuse/core";
import { useStore } from "@/stores/store";
import { assert } from "@stefnotch/typestef/assert";
import { ReactiveSceneFiles } from "@/filesystem/scene-files";

// Unchanging props! No need to watch them.
const props = defineProps<{
  files: ReactiveSceneFiles;
  canvas: HTMLCanvasElement;
  engine: WebGPUEngine;
}>();

interface KeyedCode {
  readonly id: string;
  readonly code: string;
  readonly file: string;
}
function readFile(file: string): KeyedCode {
  return {
    id: crypto.randomUUID(),
    code: props.files.readFile(file) ?? "",
    file,
  };
}

const store = useStore();

const canvasContainer = ref<HTMLDivElement | null>(null);
const keyedCode = ref<KeyedCode>(readFile("customVertexShader"));
const baseScene = shallowRef(new BaseScene(props.engine));
const scene = shallowRef(new MyFirstScene(baseScene.value, props.files));
const fileNames = computed(() => [...props.files.fileNames.value.keys()]);
const fileNameOptions = computed(() =>
  fileNames.value.map((name) => ({
    label: name,
    value: name,
  }))
);

// Re-read the code after loading the user's scene
keyedCode.value = readFile(keyedCode.value.file);

// Attach the canvas to the DOM
watchEffect(() => {
  canvasContainer.value?.appendChild(props.canvas);
});

// Resize the engine when the canvas size changes
const { width, height } = useElementSize(() => props.canvas);
watch(
  [width, height],
  useDebounceFn(() => {
    props.engine.resize();
  }, 100)
);

props.engine.runRenderLoop(renderLoop);
function renderLoop() {
  baseScene.value.update();
  baseScene.value.render();
}

function reloadScene() {
  scene.value.dispose();
  scene.value = new MyFirstScene(baseScene.value, props.files);
}

const setNewCode = useDebounceFn((newCode: () => string) => {
  const value = newCode();
  if (!scene.value) {
    console.error("No scene, but want to update code");
    return;
  }

  props.files.writeFile(keyedCode.value.file, value);

  reloadScene();
}, 500);

onUnmounted(() => {
  scene.value.dispose();
  baseScene.value.dispose();
});
</script>

<template>
  <main class="min-h-full">
    <div class="flex" style="height: 90vh">
      <div
        ref="canvasContainer"
        class="self-stretch flex-1 overflow-hidden"
      ></div>
      <CodeEditor
        class="self-stretch flex-1 overflow-hidden"
        :keyed-code="{
          id: keyedCode.id,
          code: keyedCode.code,
        }"
        :is-dark="store.isDark"
        @update="setNewCode($event)"
      >
      </CodeEditor>
    </div>

    <n-select
      :value="keyedCode.file"
      @update:value="(v) => (keyedCode = readFile(v))"
      filterable
      placeholder="Select a file"
      :options="fileNameOptions"
    />
  </main>
</template>

<style scoped></style>
