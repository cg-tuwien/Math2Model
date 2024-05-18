<script setup lang="ts">
import { ReactiveFiles } from "@/filesystem/reactive-files";
import EditorAndOutput from "@/components/EditorAndOutput.vue";
import { markRaw, shallowRef } from "vue";
import { sceneFilesPromise } from "@/globals";
import type { Engine } from "@/engine/engine";
import { getEngine } from "@/engine/babylon-engine-promise";

const sceneFiles = shallowRef<ReactiveFiles | null>(null);
sceneFilesPromise.then((v) => {
  sceneFiles.value = markRaw(v);
});
const engine = shallowRef<Engine | null>(null);
const canvas = shallowRef<HTMLCanvasElement | null>(null);
getEngine().then((v) => {
  canvas.value = v.canvas;
  engine.value = markRaw(v.engine);
});
</script>

<template>
  <EditorAndOutput
    v-if="sceneFiles !== null && engine !== null && canvas !== null"
    :files="sceneFiles"
    :canvas="canvas"
    :engine="engine"
  ></EditorAndOutput>
  <span v-else>Loading...</span>
</template>

<style scoped></style>
