<script setup lang="ts">
import { ReactiveFilesystem } from "@/filesystem/reactive-files";
import { GpuDevicePromise } from "@/webgpu-hook";
import { markRaw, shallowRef } from "vue";
import { sceneFilesPromise, takeCanvas } from "@/globals";
import { WgpuEngine } from "@/engine/wgpu-engine";

const sceneFiles = shallowRef<ReactiveFilesystem | null>(null);
sceneFilesPromise.then((v) => {
  sceneFiles.value = markRaw(v);
});
const engine = shallowRef<WgpuEngine | null>(null);
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
    :fs="sceneFiles"
    :canvas="canvasElement"
    :engine="engine"
    :visual="true"
    :gpuDevice="gpuDevice"
  ></EditorAndOutput>
  <span v-else>Loading...</span>
</template>

<style scoped></style>
