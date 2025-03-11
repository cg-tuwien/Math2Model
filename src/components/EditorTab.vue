<script setup lang="ts">
const props = defineProps<{
  title: string;
}>();
</script>
<template>
  <div class="flex flex-col">
    <h2
      class="px-2 p-1.5 dark:text-white bg-neutral-50 dark:bg-slate-900 text-l"
    >
      {{ title }}
    </h2>
    <slot></slot>
  </div>
</template>
