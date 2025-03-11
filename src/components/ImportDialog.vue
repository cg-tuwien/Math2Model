<script setup lang="ts">
import type { ImportProjectDialog } from "@/stores/fs-store";

const props = defineProps<{
  dialog: ImportProjectDialog;
}>();
const emit = defineEmits<{
  finish: ["files" | "project" | "cancel"];
}>();
</script>
<template>
  <div class="w-full max-w-2xl">
    <n-card title="Import" closable @close="emit('finish', 'cancel')">
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
  </div>
</template>
