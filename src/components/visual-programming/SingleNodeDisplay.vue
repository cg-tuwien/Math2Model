<script setup lang="ts">
import type { UINode } from "@/vpnodes/ui/uinode";

const props = defineProps<{
  uiNode: UINode;
  color: string;
}>();
</script>

<template>
  <div>
    <n-icon :component="uiNode.image" size="48px" :color="props.color"></n-icon>
    <div>
      {{ uiNode.type }} <br />
      {{ uiNode.prefix }} {{ uiNode.name }}
    </div>
  </div>
</template>

<style scoped></style>
