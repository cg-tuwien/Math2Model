<script setup lang="ts">
import type { UINode } from "@/vpnodes/ui/uinode";
import SingleNodeDisplay from "@/components/visual-programming/SingleNodeDisplay.vue";
import { NodeEditor } from "rete";
import type { Schemes } from "@/vpnodes/nodes-list";

const props = defineProps<{
  displayNodes: Map<string, UINode>;
  editor: NodeEditor<Schemes>;
  header: string;
  color: string;
  filter: string;
}>();

function matchesFilter(node: UINode): boolean {
  return node.name.toLowerCase().includes(props.filter.toLowerCase());
}
</script>

<template>
  <n-collapse :default-expanded-names="props.header">
    <n-collapse-item
      :title="props.header"
      :name="props.header"
      class="select-none"
    >
      <n-list clickable hoverable :show-divider="false">
        <template v-for="node of displayNodes.values()">
          <n-list-item
            class="border-2 border-black mb-1"
            style="border-radius: 10px; padding: 8px 12px"
            v-if="matchesFilter(node)"
            @click="() => node.create(node)"
            :draggable="node.draggable"
            @dragstart="
              (ev: DragEvent) => {
                if (ev.dataTransfer) {
                  ev.dataTransfer.setData('text/plain', JSON.stringify(node));
                  ev.dataTransfer.effectAllowed = 'copy';
                }
              }
            "
          >
            <SingleNodeDisplay
              :ui-node="node"
              :color="color"
            ></SingleNodeDisplay>
          </n-list-item>
        </template>
      </n-list>
    </n-collapse-item>
  </n-collapse>
</template>

<style scoped></style>
