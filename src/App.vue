<script setup lang="ts">
import { darkTheme, lightTheme } from "naive-ui";
import { RouterView } from "vue-router";
import { useStore } from "./stores/store";
import { computed, markRaw, shallowRef } from "vue";
import type { ReactiveFiles } from "./filesystem/reactive-files";
import { sceneFilesPromise } from "./globals";

const store = useStore();
const theme = computed(() => (store.isDark ? darkTheme : lightTheme));
const sceneFiles = shallowRef<ReactiveFiles | null>(null);
sceneFilesPromise.then((v) => {
  sceneFiles.value = markRaw(v);
});
</script>

<template>
  <div class="bg-white dark:bg-slate-800 h-full">
    <n-config-provider
      :theme="theme"
      class="h-full flex items-stretch flex-col"
    >
      <TopBar :files="sceneFiles"></TopBar>

      <RouterView> </RouterView>
    </n-config-provider>
  </div>
</template>

<style scoped></style>
