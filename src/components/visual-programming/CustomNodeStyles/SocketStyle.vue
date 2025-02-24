<template>
  <n-icon
    class="socket"
    :title="data.name"
    :component="SquareDot"
    size="32"
  ></n-icon>
</template>

<script setup lang="ts">
import SquareDot from "~icons/mdi/circle-box-outline";

const props = defineProps<{
  data: {
    name: string;
  };
}>();
</script>

<style lang="scss" scoped>
@use "sass:math";

.socket {
  display: inline-block;
  cursor: pointer;
  //border: 1px solid grey;
  //height: 16px * 2;
  vertical-align: middle;
  background: inherit;
  z-index: 2;
  // box-sizing: border-box;
  &:hover {
    color: #fff;
  }
}
</style>
