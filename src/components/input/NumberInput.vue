<script setup lang="ts">
import { computed, ref } from "vue";
import { ObjectUpdate } from "./object-update";
import { useThrottleFn } from "@vueuse/core";

const props = defineProps<{
  value: number;
  step?: number;
}>();
const emit = defineEmits<{
  update: [value: ObjectUpdate<number>];
}>();

const speed = computed(() => (props.step ?? 1.0) / 10.0);

const slidingX = ref(0);

const emitSliding = useThrottleFn((value: number) => {
  emit(
    "update",
    ObjectUpdate.sliding<number>([], () => value)
  );
}, 150);

function round(value: number, decimals: number) {
  const factor = Math.pow(10, decimals);
  return Math.round(value * factor) / factor;
}

const showValue = computed(() => round(props.value + slidingX.value, 4));

async function onPointerDown(event: PointerEvent) {
  const element = event.currentTarget;
  if (element === null) return;
  if (!(element instanceof HTMLElement)) {
    return;
  }
  await element?.requestPointerLock();
  slidingX.value = 0;
}
function onPointerMove(event: PointerEvent) {
  const element = event.currentTarget;
  if (element === null) return;
  if (!(element instanceof HTMLElement)) {
    return;
  }
  if (document.pointerLockElement !== element) return;
  slidingX.value = slidingX.value + event.movementX * speed.value;

  emitSliding(props.value + slidingX.value);
}
function onPointerUp(event: PointerEvent) {
  const element = event.currentTarget;
  if (element === null) return;
  if (!(element instanceof HTMLElement)) {
    return;
  }
  document.exitPointerLock();
  const newValue = props.value + slidingX.value;
  emit("update", new ObjectUpdate([], () => newValue));
  slidingX.value = 0;
}
</script>
<template>
  <n-input-group>
    <n-input-number
      type="number"
      :value="showValue"
      @update:value="
        (newValue) => emit('update', new ObjectUpdate([], () => newValue ?? 0))
      "
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
