<script setup lang="ts">
import type { UINode } from "@/vpnodes/ui/uinode";
import { NodeEditor } from "rete";
import MultipleNodesCollapsable from "@/components/visual-programming/MultipleNodesCollapsable.vue";
import type { Schemes } from "@/vpnodes/nodes-list";
import { ref } from "vue";

const props = defineProps<{
  displayNodes: Map<string, Map<string, UINode>>;
  editor: NodeEditor<Schemes>;
  header: string;
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
  <n-scrollbar class="h-full">
    <n-list show-divide class="h-full">
      <template #header>
        <div class="mx-2">
          <div class="m-1 select-none">{{ props.header }}</div>
          <n-input
            v-model:value="filter"
            placeholder="Search"
            size="small"
            clearable
          >
            <template #prefix> <mdi-search /> </template>
          </n-input>
        </div>
      </template>
      <n-list-item
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
      </n-list-item>
    </n-list>
  </n-scrollbar>
</template>
