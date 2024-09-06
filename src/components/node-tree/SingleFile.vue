<script setup lang="ts">
import type { TreeNode } from "./NodeTreeHelper";
import { NText } from "naive-ui";

const props = defineProps<{
  node: TreeNode;
  highlight: { start: number; end: number } | null;
}>();
</script>
<template>
  <n-text class="select-none" v-if="props.highlight === null">{{
    node.label
  }}</n-text>
  <n-text class="select-none" v-else>
    <span>{{ node.label.slice(0, props.highlight.start) }}</span>
    <span class="bg-yellow-200">{{
      node.label.slice(props.highlight.start, props.highlight.end)
    }}</span>
    <span>{{ node.label.slice(props.highlight.end) }}</span>
  </n-text>
</template>
