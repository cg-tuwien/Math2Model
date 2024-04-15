<script setup lang="ts">
import type { WebGPUEngine } from "@babylonjs/core";
import { ModelDisplayVirtualScene } from "@/scenes/ModelDisplayVirtualScene";
import { BaseScene } from "@/scenes/BaseScene";
import CodeEditor, { type KeyedCode } from "@/components/CodeEditor.vue";

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
import {
  ReactiveFiles,
  makeFilePath,
  type FilePath,
} from "@/filesystem/reactive-files";
import { showError } from "@/notification";

// Unchanging props! No need to watch them.
const props = defineProps<{
  files: ReactiveFiles;
  canvas: HTMLCanvasElement;
  engine: WebGPUEngine;
}>();

function readFile(name: FilePath): KeyedCode | null {
  let code = props.files.readFile(name);
  if (code === null) {
    return null;
  }
  return {
    id: crypto.randomUUID(),
    code,
    name,
  };
}

const store = useStore();

const canvasContainer = ref<HTMLDivElement | null>(null);
const baseScene = shallowRef(new BaseScene(props.engine));
const scene = shallowRef(
  new ModelDisplayVirtualScene(baseScene.value, props.files)
);
// Read the file after the scene is created
const keyedCode = ref<KeyedCode | null>(
  readFile(makeFilePath("my-shader.vert"))
);

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
  scene.value.update();
  baseScene.value.render();
}

// TODO: Depending on the edited file, we can also just reload a few actors instead of the whole scene
function reloadScene() {
  scene.value[Symbol.dispose]();
  scene.value = new ModelDisplayVirtualScene(baseScene.value, props.files);
}

const setNewCode = useDebounceFn((newCode: () => string) => {
  const value = newCode();
  if (keyedCode.value === null) {
    showError("No file selected", new Error("No file selected"));
    return;
  }
  props.files.writeFile(keyedCode.value.name, value);

  reloadScene();
}, 500);

onUnmounted(() => {
  scene.value[Symbol.dispose]();
  baseScene.value.dispose();
});

function openFiles(v: FilePath[]) {
  if (v.length > 0) {
    keyedCode.value = readFile(v[0]);
  }
}
function addFiles(files: FilePath[]) {
  files.forEach((file) => {
    if (props.files.hasFile(file)) return;
    props.files.writeFile(file, "");
  });
}
function renameFile(oldName: FilePath, newName: FilePath) {
  if (oldName === newName) return;
  const fileData = props.files.readFile(oldName);
  if (fileData === null) return;
  props.files.deleteFile(oldName);
  props.files.writeFile(newName, fileData);

  if (oldName === keyedCode.value?.name) {
    keyedCode.value = readFile(newName);
  }
}
function deleteFiles(files: FilePath[]) {
  files.forEach((file) => {
    props.files.deleteFile(file);
    if (file === keyedCode.value?.name) {
      keyedCode.value = null;
    }
  });
}
</script>

<template>
  <main class="min-h-full">
    <div class="flex" style="height: 80vh">
      <div
        ref="canvasContainer"
        class="self-stretch flex-1 overflow-hidden"
      ></div>
      <CodeEditor
        class="self-stretch flex-1 overflow-hidden"
        :keyed-code="keyedCode"
        :is-dark="store.isDark"
        @update="setNewCode($event)"
      >
      </CodeEditor>
    </div>
    <FileBrowser
      :files="props.files"
      :open-files="keyedCode !== null ? [keyedCode.name] : []"
      @update:open-files="openFiles($event)"
      @add-files="addFiles($event)"
      @rename-file="(oldName, newName) => renameFile(oldName, newName)"
      @delete-files="deleteFiles($event)"
    ></FileBrowser>
  </main>
</template>

<style scoped></style>
