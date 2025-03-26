<template>
  <n-icon
    class="socket"
    :title="props.data.name"
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

<style scoped>
.socket {
  display: inline-block;
  cursor: pointer;
  vertical-align: middle;
  background: inherit;
  z-index: 2;
  &:hover {
    color: #fff;
  }
}
</style>
