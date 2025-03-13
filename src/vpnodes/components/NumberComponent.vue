<script setup lang="ts">
const props = defineProps<{
  data: {
    value: number;
    step: number;
    label: string;
    updateValue: (v: number) => void;
    change: (v?: number | null) => void;
    width?: number;
    height?: number;
  };
  seed?: number;
}>();

// console.log(props.data.options);
</script>

<template>
  <n-text>{{ props.data.label }}</n-text>
  <n-input-number
    v-model:value="props.data.value"
    :step="props.data.step"
    v-on:update-value="(v: number | null) => props.data.change(v)"
    v-on:pointerdown.stop=""
  >
  </n-input-number>
</template>

<style scoped></style>
