<script setup lang="ts">
import { darkTheme, lightTheme } from "naive-ui";
import { RouterView } from "vue-router";
import { useStore } from "./stores/store";
import { computed } from "vue";
import { useFsStore } from "./stores/fs-store";

const store = useStore();
const fsStore = useFsStore();
const theme = computed(() => (store.isDark ? darkTheme : lightTheme));
</script>

<template>
  <div class="bg-neutral-200 dark:bg-slate-800 h-full">
    <n-config-provider
      :theme="theme"
      class="h-full flex items-stretch flex-col"
    >
      <TopBar></TopBar>
      <RouterView> </RouterView>
    </n-config-provider>
  </div>
</template>

<style scoped></style>
