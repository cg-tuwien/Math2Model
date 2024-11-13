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
  <div class="bg-white dark:bg-slate-800 h-full">
    <n-config-provider
      :theme="theme"
      class="h-full flex items-stretch flex-col"
    >
      <TopBar></TopBar>
      <n-modal
        :show="fsStore.importProjectDialog !== null"
        :mask-closable="false"
      >
        <ImportDialog
          v-if="fsStore.importProjectDialog !== null"
          :dialog="fsStore.importProjectDialog"
          @finish="(v) => fsStore.finishImport(v)"
        ></ImportDialog>
        <div v-else></div>
      </n-modal>

      <RouterView> </RouterView>
    </n-config-provider>
  </div>
</template>

<style scoped></style>
