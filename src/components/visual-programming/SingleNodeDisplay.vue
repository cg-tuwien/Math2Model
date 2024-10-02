<script setup lang="ts">
import type { UINode } from "@/vpnodes/ui/uinode";

const props = defineProps<{
  uiNode: UINode;
}>();
</script>

<template>
  <n-flex
    ><n-image
      :alt="uiNode.name"
      width="50px"
      :src="uiNode.image"
      preview-disabled
    ></n-image>
    {{ uiNode.type }} <br />
    {{ uiNode.prefix }} {{ uiNode.name }}</n-flex
  >
</template>

<style scoped></style>
