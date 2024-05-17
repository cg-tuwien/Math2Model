<script setup lang="ts">
import {
  type NodeKey,
  type NodePath,
  type SelectionGeneration,
  NodeTreeHelper,
  type TreeNode,
} from "./NodeTreeHelper";
import { useThemeVars } from "naive-ui";

const themeVars = useThemeVars();

// A recursive component used to render a tree of nodes

const props = defineProps<{
  data: TreeNode[];
  selectionGeneration: SelectionGeneration;
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
</script>
<template>
  <!-- https://vuejs.org/guide/components/slots.html#fancy-list-example -->

  <ul>
    <li
      v-for="node in props.data"
      :key="node.key"
      @click.stop.prevent="emit('node-click', $event, path.concat([node.key]))"
    >
      <div
        class="flex tree-node"
        :class="{
          'is-selected': NodeTreeHelper.isSelected(
            node,
            props.selectionGeneration
          ),
        }"
      >
        <span
          v-for="i in path.length"
          :key="i"
          class="depth-spacer select-none"
        ></span>
        <span
          @click="emit('node-expand', path.concat([node.key]))"
          class="flex flex-col justify-center select-none w-4"
        >
          <span v-if="(node.children?.length ?? 0) <= 0">
            <mdi-circle-medium></mdi-circle-medium>
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
          :selectionGeneration="props.selectionGeneration"
          :path="path.concat([node.key])"
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
.depth-spacer {
  padding-left: 1rem;
  border-right: 1px solid transparent;
}
.tree-children:hover .depth-spacer {
  border-right-color: v-bind("themeVars.borderColor");
}
</style>
