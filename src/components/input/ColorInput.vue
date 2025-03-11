<script setup lang="ts">
import { ObjectUpdate } from "./object-update";
import { useThrottleFn } from "@vueuse/core";
import { rgba, toHexString } from "seemly";
import { ReadonlyVector3 } from "@/scenes/scene-state";
import { computed } from "vue";

const props = defineProps<{
  value: ReadonlyVector3;
}>();
const valueAsColor = computed(() =>
  toHexString([props.value.x * 255, props.value.y * 255, props.value.z * 255])
);
const emit = defineEmits<{
  update: [value: ObjectUpdate<ReadonlyVector3>];
}>();
const emitSliding = useThrottleFn((value: ReadonlyVector3) => {
  emit(
    "update",
    ObjectUpdate.sliding<ReadonlyVector3>([], () => value)
  );
}, 150);
function emitFinal(value: ReadonlyVector3) {
  emit("update", new ObjectUpdate([], () => value));
}
function toRgb(value: string): ReadonlyVector3 {
  const rgbaColor = rgba(value);
  return new ReadonlyVector3(
    rgbaColor[0] / 255,
    rgbaColor[1] / 255,
    rgbaColor[2] / 255
  );
}
</script>
<template>
  <n-color-picker
    :show-alpha="false"
    :show-preview="true"
    :value="valueAsColor"
    @update:value="(v: string) => emitSliding(toRgb(v))"
    @confirm="(v: string) => emitFinal(toRgb(v))"
  />
</template>
