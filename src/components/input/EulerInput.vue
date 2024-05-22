<script setup lang="ts">
import AngleInput from "./AngleInput.vue";

const props = defineProps<{
  modelValue: [number, number, number];
}>();
const emit = defineEmits<{
  "update:modelValue": [value: [number, number, number]];
}>();
function updateValue(newValue: number, index: number) {
  const newValues = props.modelValue.slice() as [number, number, number];
  newValues[index] = newValue;
  emit("update:modelValue", newValues);
}
</script>
<template>
  <template v-for="(value, index) in props.modelValue" :key="index">
    <AngleInput
      :modelValue="value"
      @update:modelValue="(newValue) => updateValue(newValue, index)"
    ></AngleInput>
  </template>
</template>
