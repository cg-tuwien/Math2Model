<script setup lang="ts">
import type { UINode } from "@/vpnodes/ui/uinode";
import { NodeEditor } from "rete";
import MultipleNodesCollapsable from "@/components/visual-programming/MultipleNodesCollapsable.vue";
import type { Schemes } from "@/vpnodes/nodes-list";
import { ref } from "vue";

const props = defineProps<{
  displayNodes: Map<string, Map<string, UINode>>;
  editor: NodeEditor<Schemes>;
}>();

const colors = [
  "#A44A46",
  "#63E092",
  "#E1CF69",
  "#4E46A3",
  "#E069AF",
  "#4CA346",
];
const filter = ref("");
</script>

<template>
  <div class="h-full bg-neutral-50 dark:bg-neutral-900 p-1">
    <div class="p-2">
      <span class="select-none dark:text-white">Nodes</span>
      <n-input
        class="mt-2"
        v-model:value="filter"
        placeholder="Search"
        size="small"
        clearable
      >
      </n-input>
    </div>
    <n-scrollbar class="h-full">
      <ul class="h-full">
        <li
          class="m-1"
          v-for="(name, index) of displayNodes.keys()"
          :key="name"
        >
          <MultipleNodesCollapsable
            :display-nodes="displayNodes.get(name) ?? new Map<string, UINode>()"
            :editor="props.editor"
            :header="name"
            :color="colors[index]"
            :filter="filter"
          ></MultipleNodesCollapsable>
        </li>
      </ul>
    </n-scrollbar>
  </div>
</template>
