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
  <div>
    <n-card
      title="Load new project?"
      closable
      @close="emit('finish', 'cancel')"
    >
      <template #action>
        <div class="flex justify-around mx-2 gap-4">
          <n-button type="primary" @click="emit('finish', 'project')">
            Load Project
          </n-button>
          <n-button type="error" @click="emit('finish', 'cancel')">
            Cancel
          </n-button>
        </div>
      </template>
    </n-card>
  </div>
</template>
