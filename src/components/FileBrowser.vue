<script setup lang="ts">
import { computed, h, ref, watch } from "vue";
import { NButton, NInput } from "naive-ui";
import NodeTree from "./node-tree/NodeTree.vue";
import SingleFile from "./node-tree/SingleFile.vue";
import {
  makeSelectionGeneration,
  NodeTreeHelper,
  type NodePath,
  type SelectionGeneration,
  type TreeNode,
  type TreeSelection,
} from "./node-tree/NodeTreeHelper";
import {
  makeFilePath,
  type FilePath,
  type ReactiveFilesystem,
} from "@/filesystem/reactive-files";

const props = defineProps<{
  fs: ReactiveFilesystem;
}>();

const emit = defineEmits<{
  openFile: [file: FilePath];
  addFiles: [files: Set<FilePath>];
  deleteFiles: [files: Set<FilePath>];
  renameFile: [oldName: FilePath, newName: FilePath];
}>();
function openFile(file: FilePath) {
  emit("openFile", file);
}
function addFiles(files: Set<FilePath>) {
  emit("addFiles", files);
}
function deleteFiles(files: Set<FilePath>) {
  emit("deleteFiles", files);
}
function renameFile(oldName: FilePath, newName: FilePath) {
  emit("renameFile", oldName, newName);
}

const renamingKey = ref<{ oldName: FilePath; newName: FilePath } | null>(null);
const pattern = ref("");
function matchesPattern(node: TreeNode): { start: number; end: number } | null {
  if (pattern.value === "") {
    return { start: 0, end: node.label.length };
  }
  const start = node.label.indexOf(pattern.value);
  if (start === -1) {
    return null;
  }
  return { start, end: start + pattern.value.length };
}
const data = ref<TreeNode>({
  key: "root",
  label: "Root",
  children: [],
});
const dumbFiles = ref(new Set<FilePath>());
props.fs.watchFromStart((change) => {
  if (change.type === "insert") {
    dumbFiles.value.add(change.key);
  } else if (change.type === "remove") {
    dumbFiles.value.delete(change.key);
  }
});

watch(
  dumbFiles,
  (fileNames) => {
    const oldDataMap = new Map(
      (data.value.children ?? []).map((node) => [node.key, node])
    );

    data.value.children = [...fileNames.keys()]
      .toSorted()
      .map((file): TreeNode => {
        const newNode = oldDataMap.get(file) ?? {
          label: file,
          key: file,
        };
        return newNode;
      });
  },
  {
    deep: true,
    immediate: true,
  }
);
const filteredData = computed(() => {
  if (pattern.value === "") {
    return data.value;
  }
  const filteredData = { ...data.value };
  filteredData.children = (data.value.children ?? []).flatMap((node) =>
    NodeTreeHelper.filterNodes(node, (v) => matchesPattern(v) !== null)
  );
  return filteredData;
});
const treeSelection = ref<TreeSelection>({
  base: [],
  generation: makeSelectionGeneration(0),
});
const selectedFiles = computed(() => {
  const selectedFiles: FilePath[] = [];
  const generation = treeSelection.value.generation;
  for (const [node, _] of NodeTreeHelper.visibleNodesIter(filteredData.value)) {
    if (NodeTreeHelper.isSelected(node, generation)) {
      selectedFiles.push(makeFilePath(node.key));
    }
  }
  return selectedFiles;
});
watch(
  () => treeSelection.value.generation,
  () => {
    const selectedFile = selectedFiles.value.at(0);
    if (selectedFile !== undefined) {
      openFile(selectedFile);
    }
  },
  {
    immediate: true,
  }
);

function startAddFile() {
  let newFile = makeFilePath("untitled");
  let i = 0;
  while (props.fs.hasFile(newFile)) {
    newFile = makeFilePath(`untitled${i}`);
    i++;
  }

  addFiles(new Set([newFile]));
  // TODO: Autofocus the new file
  renamingKey.value = { oldName: newFile, newName: newFile };
}

function renderLabel({ option }: { option: TreeNode }) {
  if (renamingKey.value !== null && option.key === renamingKey.value.oldName) {
    return h(NInput, {
      value: renamingKey.value.newName,
      size: "small",
      onUpdateValue: (v: string) => {
        if (renamingKey.value === null) return;
        renamingKey.value.newName = makeFilePath(v);
      },
      onBlur: () => {
        if (renamingKey.value === null) return;
        if (
          renamingKey.value.newName !== renamingKey.value.oldName &&
          renamingKey.value.newName !== ""
        ) {
          renameFile(
            makeFilePath(renamingKey.value.oldName),
            makeFilePath(renamingKey.value.newName)
          );
        }
        renamingKey.value = null;
      },
      onKeyup: (e: KeyboardEvent) => {
        if (e.key === "Enter") {
          e.preventDefault();
          e.stopPropagation();
          if (e.target instanceof HTMLElement) {
            e.target?.blur();
          }
        }
      },
    });
  } else if (pattern.value === "") {
    // No filtering
    return h(SingleFile, {
      node: option,
      highlight: null,
    });
  } else {
    const matches = matchesPattern(option);
    return h(SingleFile, {
      node: option,
      highlight: matches,
    });
  }
}

function onNodeExpand(path: NodePath, value: boolean) {
  let node = NodeTreeHelper.getNode(data.value, path);
  if (node !== null) {
    node.isExpanded = value;
  }
}
function onNodeSelect(path: NodePath, value: [SelectionGeneration, boolean]) {
  let node = NodeTreeHelper.getNode(data.value, path);
  if (node !== null) {
    node.isSelected = value;
  }
}
</script>
<template>
  <n-flex vertical>
    <n-flex class="mx-2">
      <n-button size="small" @click="startAddFile()"> Add </n-button>
      <n-button
        size="small"
        :disabled="selectedFiles.length !== 1"
        @click="
          {
            let oldName = makeFilePath(selectedFiles[0]);
            renamingKey = {
              oldName,
              newName: oldName,
            };
          }
        "
      >
        Rename
      </n-button>
      <n-popconfirm
        @positive-click="deleteFiles(new Set(selectedFiles))"
        @negative-click="() => {}"
      >
        <template #trigger>
          <n-button size="small" :disabled="selectedFiles.length < 1">
            Delete
          </n-button>
        </template>
        Are you sure you want to delete
        {{ selectedFiles.join(", ") }}?
      </n-popconfirm>
    </n-flex>
    <n-flex class="mx-2">
      <n-input
        v-model:value="pattern"
        placeholder="Search"
        clearable
        size="small"
    /></n-flex>
    <NodeTree
      :root="filteredData"
      v-model:selection="treeSelection"
      @setExpanded="onNodeExpand"
      @setIsSelected="onNodeSelect"
    >
      <template #node="node">
        <renderLabel :option="node" />
      </template>
    </NodeTree>
  </n-flex>
</template>
<style scoped></style>
