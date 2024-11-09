<template>
  <svg data-testid="connection">
    <path :d="path"></path>
  </svg>
</template>

<script lang="ts" setup>
import { ArrowForwardIosFilled } from "@vicons/material";
const props = defineProps<{
  data: any;
  start?: string;
  end?: string;
  path: string;
}>();
</script>

<style lang="scss" scoped>
svg {
  overflow: visible !important;
  position: absolute;
  pointer-events: none;
  width: 9999px;
  height: 9999px;
  path {
    fill: none;
    stroke-width: 3.5px;
    stroke: #f4d120;
    pointer-events: auto;
    animation: opacity 1.5s linear 0.5s infinite alternate;
  }
}

@keyframes opacity {
  from {
    stroke-opacity: 100%;
  }
  to {
    stroke-opacity: 50%;
  }
}
</style>
