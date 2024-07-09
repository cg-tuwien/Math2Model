<script setup lang="ts">
import type { ImportProjectDialog } from "@/stores/store";

const props = defineProps<{
  dialog: ImportProjectDialog;
}>();
const emit = defineEmits<{
  finish: ["files" | "project" | "cancel"];
}>();
</script>
<template>
  <n-card
    class="w-full sm:w-1/2 lg:w-1/3"
    title="Import"
    closable
    @close="emit('finish', 'cancel')"
  >
    Import as <em>new project</em> or <em>add files to project</em>?
    <template #action>
      <div class="flex justify-around">
        <n-button type="primary" @click="emit('finish', 'project')">
          New Project
        </n-button>
        <n-button type="primary" @click="emit('finish', 'files')">
          Add Files
        </n-button>
      </div>
    </template>
  </n-card>
</template>
