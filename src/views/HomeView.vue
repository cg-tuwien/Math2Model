<script setup lang="ts">
import { ReactiveFiles } from "@/filesystem/reactive-files";
import EditorAndOutput from "@/components/EditorAndOutput.vue";
import { markRaw, shallowRef } from "vue";
import { sceneFilesPromise, canvasElement, enginePromise } from "@/globals";
import type { Engine } from "@/engine/engine";

const sceneFiles = shallowRef<ReactiveFiles | null>(null);
sceneFilesPromise.then((v) => {
  sceneFiles.value = markRaw(v);
});
const engine = shallowRef<Engine | null>(null);
enginePromise.then((v) => {
  engine.value = markRaw(v);
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
