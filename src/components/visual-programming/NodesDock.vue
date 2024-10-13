<script setup lang="ts">
import type { ReactiveFilesystem } from "@/filesystem/reactive-files";
import type { SelectMixedOption } from "naive-ui/es/select/src/interface";
import type { DeepReadonly } from "vue";
import type { UINode } from "@/vpnodes/ui/uinode";
import SingleNodeDisplay from "@/components/visual-programming/SingleNodeDisplay.vue";
import { NodeEditor } from "rete";

const props = defineProps<{
  displayNodes: UINode[];
  editor: NodeEditor;
  header: string;
}>();
</script>

<template>
  <n-infinite-scroll>
    <n-list bordered hoverable clickable show-divider>
      <template #header> {{ props.header }} </template>
      <n-list-item
        v-for="node of displayNodes"
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
  </n-infinite-scroll>
</template>

<style scoped></style>
