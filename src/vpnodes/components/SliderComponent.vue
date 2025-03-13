<script setup lang="ts">
const props = defineProps<{
  data: {
    value: number;
    step: number;
    max: number;
    min: number;
    label: string;
    updateValue: (v: number) => void;
    change: (v?: number | null) => void;
    noDebounceChange: (v?: number | null) => void;
    showInput: boolean;
    width?: number;
    height?: number;
  };
  seed?: number;
}>();

// console.log(props.data.options);
</script>

<template>
  <n-text class="text-amber-50">{{ props.data.label }}</n-text>
  <n-slider
    v-model:value="props.data.value"
    :step="props.data.step"
    :max="props.data.max"
    :min="props.data.min"
    v-on:update-value="(v: number) => props.data.updateValue(v)"
    v-on:dragend="() => props.data.noDebounceChange()"
    v-on:pointerdown.stop=""
    tooltip
  ></n-slider>
  <n-input-number
    v-if="props.data.showInput"
    v-model:value="props.data.value"
    :step="props.data.step"
    :min="props.data.min"
    :max="props.data.max"
    v-on:update-value="(v: number | null) => props.data.change(v)"
    v-on:click="(v: number | null) => props.data.updateValue(v ?? 0.0)"
    v-on:pointerdown.stop=""
  >
  </n-input-number>
</template>

<style scoped></style>
