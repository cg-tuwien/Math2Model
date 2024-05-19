<script setup lang="ts">
import { computed, ref } from "vue";

const props = defineProps<{
  modelValue: number;
}>();
const emit = defineEmits<{
  "update:modelValue": [value: number];
}>();

const sliding = ref<null | {
  x: number;
  y: number;
}>(null);

function round(value: number, decimals: number) {
  const factor = Math.pow(10, decimals);
  return Math.round(value * factor) / factor;
}

const showValue = computed(() =>
  round(props.modelValue + (sliding.value?.x ?? 0), 4)
);

async function onPointerDown(event: PointerEvent) {
  const element = event.currentTarget;
  if (element === null) return;
  if (!(element instanceof HTMLElement)) {
    return;
  }
  await element?.requestPointerLock();
  sliding.value = {
    x: 0,
    y: 0,
  };
}
function onPointerMove(event: PointerEvent) {
  const element = event.currentTarget;
  if (element === null) return;
  if (!(element instanceof HTMLElement)) {
    return;
  }
  if (document.pointerLockElement !== element) return;
  // Maybe this should be screen width dependent? Or screen dpi dependent?
  const speed = 0.1;
  sliding.value = {
    x: (sliding.value?.x ?? 0) + event.movementX * speed,
    y: (sliding.value?.y ?? 0) + event.movementY * speed,
  };
}
function onPointerUp(event: PointerEvent) {
  const element = event.currentTarget;
  if (element === null) return;
  if (!(element instanceof HTMLElement)) {
    return;
  }
  document.exitPointerLock();
  emit("update:modelValue", props.modelValue + (sliding.value?.x ?? 0));
  sliding.value = null;
}
</script>
<template>
  <n-input-group>
    <n-input-number
      type="number"
      :value="showValue"
      @update:value="($event) => emit('update:modelValue', $event ?? 0)"
      :update-value-on-input="false"
      :show-button="false"
      size="small"
      class="grow"
    ></n-input-number>
    <n-input-group-label
      size="small"
      class="hover:cursor-col-resize flex justify-center items-center px-0"
      @pointerdown="onPointerDown"
      @pointermove="onPointerMove"
      @pointerup="onPointerUp"
      @pointercancel="onPointerUp"
      ><mdi-arrow-left-right></mdi-arrow-left-right
    ></n-input-group-label>
  </n-input-group>
</template>
