<script setup lang="ts">
import type { UINode } from "@/vpnodes/ui/uinode";
import { NodeEditor } from "rete";
import MultipleNodesCollapsable from "@/components/visual-programming/MultipleNodesCollapsable.vue";
import type { Schemes } from "@/vpnodes/nodes-list";

const props = defineProps<{
  displayNodes: Map<string, Map<string, UINode>>;
  editor: NodeEditor<Schemes>;
  header: string;
  filter: string;
}>();

const colors = [
  "#A44A46",
  "#63E092",
  "#E1CF69",
  "#4E46A3",
  "#E069AF",
  "#4CA346",
];

function matchesFilter(name: string): boolean {
  return name.toLowerCase().includes(props.filter.toLowerCase());
}
</script>

<template>
  <n-infinite-scroll>
    <n-list show-divide>
      <template #header>
        <div class="m-1 select-none">{{ props.header }}</div>
      </template>
      <template v-for="(name, index) of displayNodes.keys()" :key="name">
        <n-list-item class="m-1" v-if="matchesFilter(name)">
          <MultipleNodesCollapsable
            :display-nodes="displayNodes.get(name) ?? new Map<string, UINode>()"
            :editor="props.editor"
            :header="name"
            :color="colors[index]"
          ></MultipleNodesCollapsable>
        </n-list-item>
      </template>
    </n-list>
  </n-infinite-scroll>
</template>

<style scoped></style>
