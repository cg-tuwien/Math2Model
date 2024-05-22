<script setup lang="ts">
import NumberInput from "./NumberInput.vue";

const props = defineProps<{
  modelValue: number[];
}>();
const emit = defineEmits<{
  "update:modelValue": [value: number[]];
}>();
function updateValue(newValue: number, index: number) {
  const newValues = props.modelValue.slice();
  newValues[index] = newValue;
  emit("update:modelValue", newValues);
}
</script>
<template>
  <template v-for="(value, index) in props.modelValue" :key="index">
    <NumberInput
      :modelValue="value"
      @update:modelValue="(newValue) => updateValue(newValue, index)"
    ></NumberInput>
  </template>
</template>
