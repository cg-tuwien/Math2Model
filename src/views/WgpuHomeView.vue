<script setup lang="ts">
import { ReactiveFilesystem } from "@/filesystem/reactive-files";
import { markRaw, shallowRef } from "vue";
import { sceneFilesPromise, takeCanvas } from "@/globals";
import type { Engine } from "@/engine/engine";
import { WgpuEngine } from "@/engine/wgpu-engine";

const sceneFiles = shallowRef<ReactiveFilesystem | null>(null);
sceneFilesPromise.then((v) => {
  sceneFiles.value = markRaw(v);
});
const engine = shallowRef<Engine | null>(null);
const canvasElement = takeCanvas();
if (canvasElement === null) {
  window.location.reload();
  throw new Error("Canvas element already used, reloading the site.");
}
WgpuEngine.createEngine(canvasElement).then((v) => {
  engine.value = markRaw(v);
});
</script>

<template>
  <EditorAndOutput
    v-if="sceneFiles !== null && engine !== null && canvasElement !== null"
    :fs="sceneFiles"
    :canvas="canvasElement"
    :engine="engine"
  ></EditorAndOutput>
  <span v-else>Loading...</span>
</template>

<style scoped></style>
