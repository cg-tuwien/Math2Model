<script setup lang="ts">
import {
  ReactiveSceneFiles,
  SceneFilesWithFilesystem,
} from "@/filesystem/scene-files";
import EditorAndOutput from "@/components/EditorAndOutput.vue";
import { markRaw, shallowRef } from "vue";
import { WebGPUEngine } from "@babylonjs/core";
import { canvasElement, engine } from "@/engine/engine";

const sceneFiles = shallowRef<ReactiveSceneFiles | null>(null);

WebGPUEngine.IsSupportedAsync.then((supported) => {
  if (!supported) {
    alert("WebGPU not supported");
  }
});
(async () => {
  let fs = await SceneFilesWithFilesystem.create("some-key");
  let files = await ReactiveSceneFiles.create(fs);
  sceneFiles.value = markRaw(files);
})();
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
