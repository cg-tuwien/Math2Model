<script setup lang="ts">
import { darkTheme, lightTheme, useThemeVars } from "naive-ui";
import { RouterView } from "vue-router";
import { useStore } from "./stores/store";
import { computed } from "vue";

const store = useStore();
const theme = computed(() => (store.isDark ? darkTheme : lightTheme));
const themeVars = useThemeVars();
</script>

<template>
  <div class="bg-white dark:bg-slate-800 h-full">
    <n-config-provider
      :theme="theme"
      class="h-full flex items-stretch flex-col"
    >
      <TopBar></TopBar>
      <n-modal
        :show="store.importProjectDialog !== null"
        :mask-closable="false"
      >
        <ImportDialog
          v-if="store.importProjectDialog !== null"
          :dialog="store.importProjectDialog"
          @finish="(v) => store.finishImport(v)"
        ></ImportDialog>
        <div v-else></div>
      </n-modal>

      <RouterView> </RouterView>
    </n-config-provider>
  </div>
</template>

<style scoped></style>
