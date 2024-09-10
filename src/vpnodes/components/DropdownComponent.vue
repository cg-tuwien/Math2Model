<script setup lang="ts">
const props = defineProps<{
  data: any;
}>();

// console.log(props.data.options);
</script>

<template>
  <n-text class="text-amber-50">{{ props.data.label }}</n-text>
  <n-select
    :options="props.data.options"
    v-model:value="props.data.selected"
    :placeholder="props.data.placeholder"
    v-on:update-value="props.data.change"
    v-on:pointerdown.stop=""
  ></n-select>
</template>

<style scoped></style>
