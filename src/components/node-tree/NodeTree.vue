<script setup lang="ts">
import NodeTreeRender from "./NodeTreeRender.vue";
import { computed, ref, watch, watchEffect } from "vue";
import { changeColor } from "seemly";
import { useThemeVars } from "naive-ui";
import {
  NodeTreeHelper,
  makeSelectionGeneration,
  type NodePath,
  type SelectionGeneration,
  type TreeNode,
  type TreeSelection,
} from "./NodeTreeHelper";

const props = defineProps<{
  root: TreeNode;
}>();
const selection = defineModel<TreeSelection>("selection", {
  required: true,
});
const emit = defineEmits<{
  setExpanded: [path: NodePath, value: boolean];
  setIsSelected: [path: NodePath, value: [SelectionGeneration, boolean]];
}>();

const themeVars = useThemeVars();

/**
 * This imitates the VSCode behavior.
 */
function onNodeClick(event: MouseEvent, path: NodePath) {
  const numberOfClicks = event.detail;
  const isDoubleClick = numberOfClicks > 0 && numberOfClicks % 2 === 0;
  if (isDoubleClick) {
    onNodeExpand(path);
  }
  const isCtrlClick = event.ctrlKey;
  const isShiftClick = event.shiftKey;
  if (isCtrlClick && !isShiftClick) {
    const node = NodeTreeHelper.getNode(props.root, path);
    if (node === null) {
      return;
    }
    const isSelected = NodeTreeHelper.isSelected(
      node,
      selection.value.generation
    );
    emit("setIsSelected", path, [selection.value.generation, !isSelected]);
  } else if (isShiftClick) {
    const unselectedPaths: NodePath[] = [];
    const selectedPaths: NodePath[] = [];
    const visibleNodesIter = NodeTreeHelper.visibleNodesIter(props.root);

    const selectStart = () => {
      let continuousSelection: NodePath[] = [];
      while (true) {
        const v = visibleNodesIter.next();
        if (v.done) {
          break;
        }
        const [node, nodePath] = v.value;

        if (NodeTreeHelper.pathEquals(nodePath, selection.value.base)) {
          selectedPaths.push(nodePath);
          if (NodeTreeHelper.pathEquals(nodePath, path)) {
            // Start of selection == end of selection
          } else {
            selectUntilEnd(path);
          }
          break;
        }
        if (NodeTreeHelper.pathEquals(nodePath, path)) {
          selectedPaths.push(nodePath);
          selectUntilEnd(selection.value.base);
          break;
        }
        if (NodeTreeHelper.isSelected(node, selection.value.generation)) {
          continuousSelection.push(nodePath);
        } else {
          continuousSelection = [];
        }
      }

      unselectedPaths.push(...continuousSelection);
    };

    const selectUntilEnd = (endPath: NodePath) => {
      while (true) {
        const v = visibleNodesIter.next();
        if (v.done) {
          break;
        }
        const [_, nodePath] = v.value;
        selectedPaths.push(nodePath);
        if (NodeTreeHelper.pathEquals(nodePath, endPath)) {
          break;
        }
      }
    };

    const unselectAfterEnd = () => {
      while (true) {
        const v = visibleNodesIter.next();
        if (v.done) {
          break;
        }
        const [node, nodePath] = v.value;
        if (NodeTreeHelper.isSelected(node, selection.value.generation)) {
          unselectedPaths.push(nodePath);
        } else {
          break;
        }
      }
    };

    selectStart();
    unselectAfterEnd();
    const generation = selection.value.generation;
    for (const selectedPath of selectedPaths) {
      emit("setIsSelected", selectedPath, [generation, true]);
    }
    for (const unselectedPath of unselectedPaths) {
      emit("setIsSelected", unselectedPath, [generation, false]);
    }
  } else {
    const generation = makeSelectionGeneration(selection.value.generation + 1);
    emit("setIsSelected", path, [generation, true]);
    selection.value = {
      base: path,
      generation: makeSelectionGeneration(generation),
    };
  }
}

function onNodeExpand(path: NodePath) {
  const node = NodeTreeHelper.getNode(props.root, path);
  const isExpanded = node?.isExpanded ?? false;
  emit("setExpanded", path, !isExpanded);
}

const slots = defineSlots<{
  node(node: TreeNode): any;
}>();

const selectedColor = computed(() => {
  return changeColor(themeVars.value.successColor, {
    alpha: +themeVars.value.opacity2,
  });
});
const selectedColorHover = computed(() => {
  return changeColor(themeVars.value.successColor, {
    alpha: +themeVars.value.opacity1,
  });
});
</script>
<template>
  <div>
    <NodeTreeRender
      :data="props.root.children ?? []"
      :selectionGeneration="selection.generation"
      :path="[]"
      :selectedColor="selectedColor"
      :selectedColorHover="selectedColorHover"
      @node-click="(ev, path) => onNodeClick(ev, path)"
      @node-expand="(path) => onNodeExpand(path)"
      ><template #node="n"><slot name="node" v-bind="n" /></template
    ></NodeTreeRender>
  </div>
</template>
