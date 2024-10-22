<script setup lang="ts">
import type { ReactiveFilesystem } from "@/filesystem/reactive-files";
import type { SelectMixedOption } from "naive-ui/es/select/src/interface";
import type { DeepReadonly } from "vue";
import type { UINode } from "@/vpnodes/ui/uinode";
import SingleNodeDisplay from "@/components/visual-programming/SingleNodeDisplay.vue";
import { NodeEditor } from "rete";
import MultipleNodesCollapsable from "@/components/visual-programming/MultipleNodesCollapsable.vue";

const props = defineProps<{
  displayNodes: Map<string, Map<string, UINode>>;
  editor: NodeEditor;
  header: string;
}>();
</script>

<template>
  <n-infinite-scroll>
    <n-list show-divide>
      <template #header>
        <div class="m-1">{{ props.header }}</div>
      </template>
      <n-list-item class="m-1" v-for="name of displayNodes.keys()">
        <MultipleNodesCollapsable
          :display-nodes="displayNodes.get(name)"
          :editor="props.editor"
          :header="name"
        ></MultipleNodesCollapsable>
      </n-list-item>
    </n-list>
  </n-infinite-scroll>
</template>

<style scoped></style>
