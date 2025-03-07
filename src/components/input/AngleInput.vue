<script setup lang="ts">
import { computed } from "vue";
import NumberInput from "./NumberInput.vue";
import { ObjectUpdate } from "./object-update";

const props = defineProps<{
  value: number;
  label?: string;
}>();
const emit = defineEmits<{
  update: [value: ObjectUpdate<number>];
}>();

const angleInDegrees = computed(() => radToDeg(props.value));

function radToDeg(rad: number): number {
  const result = (rad * 180) / Math.PI;
  // Round it to 4 decimal places
  return Math.round(result * 1e4) / 1e4;
}
function degToRad(deg: number): number {
  return (deg * Math.PI) / 180;
}

function emitValue(newValue: ObjectUpdate<number>) {
  emit(
    "update",
    new ObjectUpdate<number>(
      newValue.path,
      (v) => degToRad(newValue.newValue(v)),
      newValue.isSliding
    )
  );
}
</script>
<template>
  <NumberInput
    :value="angleInDegrees"
    :step="1.0"
    @update="(newValue) => emitValue(newValue)"
    :label="props.label"
  ></NumberInput>
</template>
