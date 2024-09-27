<script setup lang="ts">
import type { UINode } from "@/vpnodes/ui/uinode";

const props = defineProps<{
  uiNode: UINode;
}>();
</script>

<template>
  <n-flex
    ><n-image :alt="uiNode.name" width="50px" :src="uiNode.image"></n-image>
    {{ uiNode.prefix }} <br />
    parametric {{ uiNode.name }}</n-flex
  >
</template>

<style scoped></style>
