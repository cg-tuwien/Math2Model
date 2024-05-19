<script setup lang="ts">
import { computed } from "vue";
import {
  type NodeKey,
  type NodePath,
  type SelectionGeneration,
  NodeTreeHelper,
  type TreeNode,
  type TreeSelection,
} from "./NodeTreeHelper";
import { useThemeVars } from "naive-ui";

const themeVars = useThemeVars();

// A recursive component used to render a tree of nodes

const props = defineProps<{
  data: TreeNode[];
  selection: TreeSelection;
  path: NodePath;
  selectedColor: string;
}>();

const emit = defineEmits<{
  "node-click": [event: MouseEvent, path: NodePath];
  "node-expand": [path: NodePath];
  // TODO: Arrow keys to navigate the tree & to select nodes
}>();

const slots = defineSlots<{
  node(node: TreeNode): any;
}>();

const nodePaths = computed(() => {
  return props.data.map((node) => props.path.concat([node.key]));
});
</script>
<template>
  <!-- https://vuejs.org/guide/components/slots.html#fancy-list-example -->

  <ul>
    <li
      v-for="(node, index) in props.data"
      :key="node.key"
      @click.stop.prevent="emit('node-click', $event, nodePaths[index])"
    >
      <div
        class="flex tree-node"
        :class="{
          'is-selected': NodeTreeHelper.isSelected(
            node,
            props.selection.generation
          ),
          'is-selection-base': NodeTreeHelper.pathEquals(
            props.selection.base,
            nodePaths[index]
          ),
        }"
      >
        <span
          v-for="i in path.length"
          :key="i"
          class="depth-spacer select-none"
        ></span>
        <span
          @click="emit('node-expand', nodePaths[index])"
          class="flex items-center select-none w-4"
        >
          <span v-if="(node.children?.length ?? 0) <= 0">
            <mdi-circle-small></mdi-circle-small>
          </span>
          <span v-else-if="node.isExpanded">
            <mdi-chevron-down></mdi-chevron-down>
          </span>
          <span v-else>
            <mdi-chevron-right></mdi-chevron-right>
          </span>
        </span>
        <slot name="node" v-bind="node" />
      </div>
      <div class="tree-children">
        <NodeTreeRender
          v-if="node.children && node.isExpanded"
          :data="node.children"
          :selection="props.selection"
          :path="nodePaths[index]"
          :selectedColor="props.selectedColor"
          @node-click="(ev, path) => emit('node-click', ev, path)"
          @node-expand="(path) => emit('node-expand', path)"
        >
          <template #node="n"><slot name="node" v-bind="n" /></template>
        </NodeTreeRender>
      </div>
    </li>
  </ul>
</template>
<style scoped>
.tree-node {
  transition: background-color 0.1s;
}
.tree-node:hover {
  cursor: pointer;
  background-color: v-bind("themeVars.hoverColor");
}
.tree-node.is-selected {
  background-color: color-mix(
    in srgb,
    v-bind("themeVars.successColor") 20%,
    transparent
  );
}
.tree-node.is-selected:hover {
  background-color: color-mix(
    in srgb,
    v-bind("themeVars.successColor") 15%,
    transparent
  );
}
.tree-node.is-selection-base {
  outline: 1px solid v-bind("themeVars.successColor");
}
.depth-spacer {
  padding-left: 1rem;
  border-right: 1px solid transparent;
}
.tree-children:hover .depth-spacer {
  border-right-color: v-bind("themeVars.borderColor");
}
</style>
