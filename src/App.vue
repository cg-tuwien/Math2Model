<script setup lang="ts">
import { darkTheme, lightTheme } from "naive-ui";
import { RouterView } from "vue-router";
import { useStore } from "./stores/store";
import { computed } from "vue";

const store = useStore();
const theme = computed(() => (store.isDark ? darkTheme : lightTheme));
</script>

<template>
  <div class="bg-white dark:bg-slate-800">
    <n-config-provider :theme="theme">
      <TopBar></TopBar>

      <RouterView />
    </n-config-provider>
  </div>
</template>

<style scoped></style>
