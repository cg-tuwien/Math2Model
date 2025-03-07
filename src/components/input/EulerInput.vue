<script setup lang="ts">
import AngleInput from "./AngleInput.vue";
import type { ObjectUpdate } from "./object-update";

const props = defineProps<{
  value: [number, number, number];
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
      <AngleInput
        :value="value"
        @update="(newValue) => updateValue(newValue, index)"
      ></AngleInput>
    </template>
  </div>
</template>
