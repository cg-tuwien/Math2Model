<script setup lang="ts">
import { ReactiveFiles } from "@/filesystem/reactive-files";
import EditorAndOutput from "@/components/EditorAndOutput.vue";
import { markRaw, shallowRef } from "vue";
import { WebGPUEngine } from "@babylonjs/core";
import { sceneFilesPromise, canvasElement, enginePromise } from "@/globals";

const sceneFiles = shallowRef<ReactiveFiles | null>(null);
sceneFilesPromise.then((v) => {
  sceneFiles.value = markRaw(v);
});
const engine = shallowRef<WebGPUEngine | null>(null);
enginePromise.then((v) => {
  engine.value = markRaw(v);
});

WebGPUEngine.IsSupportedAsync.then((supported) => {
  if (!supported) {
    alert("WebGPU not supported");
  }
});
</script>

<template>
  <EditorAndOutput
    v-if="sceneFiles !== null && engine !== null"
    :files="sceneFiles"
    :canvas="canvasElement"
    :engine="engine"
  ></EditorAndOutput>
  <span v-else>Loading...</span>
</template>

<style scoped></style>
