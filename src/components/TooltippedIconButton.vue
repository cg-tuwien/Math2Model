<script setup lang="ts">
const props = defineProps<{
  tooltip: string;
}>();
const emit = defineEmits<{
  click: [MouseEvent];
}>();
defineOptions({
  inheritAttrs: false,
});
</script>
<template>
  <n-tooltip trigger="hover" placement="bottom">
    <template #trigger>
      <n-button
        text
        @click="(e: MouseEvent) => emit('click', e)"
        v-bind="$attrs"
      >
        <template #icon> <slot></slot> </template>
      </n-button>
    </template>
    {{ props.tooltip }}
  </n-tooltip>
</template>
