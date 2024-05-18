<script setup lang="ts">
import { computed, ref, watch } from "vue";

const props = defineProps<{
  modelValue: number;
}>();
const emit = defineEmits<{
  "update:modelValue": [value: number];
}>();

const angleInDegrees = computed({
  get: () => radToDeg(props.modelValue),
  set: (value: number) => {
    emit("update:modelValue", degToRad(value));
  },
});

function radToDeg(rad: number): number {
  const result = (rad * 180) / Math.PI;
  // Round it to 4 decimal places
  return Math.round(result * 1e4) / 1e4;
}
function degToRad(deg: number): number {
  return (deg * Math.PI) / 180;
}
</script>
<template>
  <n-input-number
    v-model:value="angleInDegrees"
    :update-value-on-input="false"
    type="number"
    clearable
    :show-button="false"
    size="small"
  />
</template>
