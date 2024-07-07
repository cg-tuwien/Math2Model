<script setup lang="ts">
import { GpuDevicePromise } from "@/webgpu-hook";
import { ReactiveFiles } from "@/filesystem/reactive-files";
import EditorAndOutput from "@/components/EditorAndOutput.vue";
import { markRaw, shallowRef } from "vue";
import { sceneFilesPromise, takeCanvas } from "@/globals";
import type { Engine } from "@/engine/engine";
import { WgpuEngine } from "@/engine/wgpu-engine";

const sceneFiles = shallowRef<ReactiveFiles | null>(null);
sceneFilesPromise.then((v) => {
  sceneFiles.value = markRaw(v);
});
const engine = shallowRef<Engine | null>(null);
const canvasElement = takeCanvas();
const gpuDevice = shallowRef<GPUDevice | null>(null);
if (canvasElement === null) {
  window.location.reload();
  throw new Error("Canvas element already used, reloading the site.");
}
GpuDevicePromise.then((v) => {
  gpuDevice.value = markRaw(v);
});
WgpuEngine.createEngine(canvasElement).then((v) => {
  engine.value = markRaw(v);
});
</script>

<template>
  <EditorAndOutput
    v-if="
      sceneFiles !== null &&
      engine !== null &&
      canvasElement !== null &&
      gpuDevice !== null
    "
    :files="sceneFiles"
    :canvas="canvasElement"
    :engine="engine"
    :gpuDevice="gpuDevice"
  ></EditorAndOutput>
  <span v-else>Loading...</span>
</template>

<style scoped></style>
