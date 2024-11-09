<script setup lang="ts">
import type { UINode } from "@/vpnodes/ui/uinode";

const props = defineProps<{
  uiNode: UINode;
  color: string;
}>();
</script>

<template>
  <n-flex>
    <n-icon :component="uiNode.image" size="48px" :color="color"></n-icon>
    <div>
      {{ uiNode.type }} <br />
      {{ uiNode.prefix }} {{ uiNode.name }}
    </div>
  </n-flex>
</template>

<style scoped></style>
