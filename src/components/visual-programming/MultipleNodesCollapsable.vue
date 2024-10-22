<script setup lang="ts">
import type { UINode } from "@/vpnodes/ui/uinode";
import SingleNodeDisplay from "@/components/visual-programming/SingleNodeDisplay.vue";
import { NodeEditor } from "rete";

const props = defineProps<{
  displayNodes: Map<string, UINode>;
  editor: NodeEditor;
  header: string;
}>();
</script>

<template>
  <n-collapse :default-expanded-names="props.header">
    <n-collapse-item :title="props.header" :name="props.header">
      <n-list clickable hoverable>
        <n-list-item
          v-for="node of displayNodes.values()"
          :onclick="() => node.create(node)"
          :draggable="node.draggable"
          v-on:dragstart="
            (ev) => {
              ev.dataTransfer.setData('text/plain', JSON.stringify(node));
              ev.dataTransfer.effectAllowed = 'copy';
            }
          "
        >
          <SingleNodeDisplay :ui-node="node"></SingleNodeDisplay>
        </n-list-item>
      </n-list>
    </n-collapse-item>
  </n-collapse>
</template>

<style scoped></style>
