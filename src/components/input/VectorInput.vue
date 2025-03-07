<script setup lang="ts">
import NumberInput from "./NumberInput.vue";
import type { ObjectUpdate } from "./object-update";
import { VectorLabels } from "./vector-labels";

const props = defineProps<{
  value: number[];
  step?: number;
}>();
const emit = defineEmits<{
  update: [value: ObjectUpdate<number>];
}>();
function updateValue(newValue: ObjectUpdate<number>, index: number) {
  emit("update", newValue.addPath(index));
}
</script>
<template>
  <div class="flex">
    <template v-for="(value, index) in props.value" :key="index">
      <NumberInput
        :value="value"
        :step="props.step"
        :label="VectorLabels[index]"
        @update="(newValue) => updateValue(newValue, index)"
      ></NumberInput>
    </template>
  </div>
</template>
