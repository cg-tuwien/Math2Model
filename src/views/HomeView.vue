<script setup lang="ts">
import {
  ReactiveSceneFiles,
  SceneFilesWithFilesystem,
} from "@/filesystem/scene-files";
import EditorAndOutput from "@/components/EditorAndOutput.vue";
import { markRaw, shallowRef } from "vue";

const sceneFiles = shallowRef<ReactiveSceneFiles | null>(null);

SceneFilesWithFilesystem.create("some-key")
  .then((fs) => ReactiveSceneFiles.create(fs))
  .then((files) => {
    sceneFiles.value = markRaw(files);
  });
</script>

<template>
  <EditorAndOutput
    v-if="sceneFiles !== null"
    :files="sceneFiles"
  ></EditorAndOutput>
  <span v-else>Loading...</span>
</template>

<style scoped></style>
