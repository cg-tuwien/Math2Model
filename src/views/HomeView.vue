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
const canvasElement = takeCanvas();
if (canvasElement === null) {
  window.location.reload();
  throw new Error("Canvas element already used, reloading the site.");
}
const engine = shallowRef<WgpuEngine>(
  markRaw(WgpuEngine.createEngine(canvasElement))
);
const gpuDevice = shallowRef<GPUDevice | null>(null);
GpuDevicePromise.then((v) => {
  gpuDevice.value = markRaw(v);
});
</script>

<template>
  <EditorAndOutput
    v-if="sceneFiles !== null && canvasElement !== null && gpuDevice !== null"
    :fs="sceneFiles"
    :canvas="canvasElement"
    :engine="engine"
    :gpuDevice="gpuDevice"
  ></EditorAndOutput>
  <span v-else>Loading...</span>
</template>

<style scoped></style>
