<script setup lang="ts">
const props = defineProps<{
  data: any;
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
    v-on:dragend="() => props.data.change()"
    v-on:pointerdown.stop=""
    tooltip
  ></n-slider>
  <n-input-number
    v-if="props.data.showInput"
    v-model:value="props.data.value"
    :step="props.data.step"
    :min="props.data.min"
    :max="props.data.max"
    v-on:change="(v: number | null) => props.data.change(v)"
    v-on:pointerdown.stop=""
  >
  </n-input-number>
</template>

<style scoped></style>
