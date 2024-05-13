<script setup lang="ts">
import { h, ref, watch } from "vue";
import { type TreeOption, NButton, NInput } from "naive-ui";
import {
  makeFilePath,
  type FilePath,
  type ReactiveFiles,
} from "@/filesystem/reactive-files";

const props = defineProps<{
  files: ReactiveFiles;
}>();

const openFiles = defineModel<FilePath[]>("openFiles", { required: true });

const emit = defineEmits<{
  "add-files": [files: FilePath[]];
  "delete-files": [files: FilePath[]];
  "rename-file": [oldName: FilePath, newName: FilePath];
}>();

const pattern = ref("");
const checkedKeys = ref<FilePath[]>([]);
const renamingKey = ref<{ oldName: FilePath; newName: FilePath } | null>(null);

const data = ref<TreeOption[]>([]);
watch(
  props.files.fileNames,
  (fileNames) => {
    data.value = [...fileNames.keys()].toSorted().map(
      (file): TreeOption => ({
        label: file,
        key: file,
      })
    );
    checkedKeys.value = checkedKeys.value.filter((key) => fileNames.has(key));
  },
  {
    deep: true,
    immediate: true,
  }
);

function toFilePaths(v: string[]) {
  return v.map((key) => makeFilePath(key));
}

function startAddFile() {
  let newFile = makeFilePath("untitled");
  let i = 0;
  while (props.files.hasFile(newFile)) {
    newFile = makeFilePath(`untitled${i}`);
    i++;
  }

  emit("add-files", [newFile]);
  // TODO: Autofocus the new file
  renamingKey.value = { oldName: newFile, newName: newFile };
}

function renderLabel({ option }: { option: TreeOption }) {
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
          emit(
            "rename-file",
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
  } else {
    return h("span", option.label);
  }
}
</script>
<template>
  <n-flex vertical>
    <n-flex justify="space-between">
      <n-flex>
        <n-button @click="startAddFile()" :disabled="checkedKeys.length > 0">
          Add
        </n-button>
        <n-button
          :disabled="checkedKeys.length !== 1"
          @click="
            renamingKey = { oldName: checkedKeys[0], newName: checkedKeys[0] }
          "
        >
          Rename
        </n-button>
        <n-popconfirm
          @positive-click="emit('delete-files', checkedKeys)"
          @negative-click="() => {}"
        >
          <template #trigger>
            <n-button :disabled="checkedKeys.length < 1"> Delete </n-button>
          </template>
          Are you sure you want to delete {{ checkedKeys.join(", ") }}?
        </n-popconfirm>
      </n-flex>
      <n-flex>
        <n-input v-model:value="pattern" placeholder="Search" clearable
      /></n-flex>
    </n-flex>
    <n-tree
      block-line
      checkable
      cascade
      expand-on-click
      show-line
      :show-irrelevant-nodes="false"
      :default-selected-keys="openFiles"
      :pattern="pattern"
      :data="data"
      :checked-keys="checkedKeys"
      :render-label="renderLabel"
      @update:checked-keys="(v: string[]) => (checkedKeys = toFilePaths(v))"
      @update:selected-keys="(v: string[]) => (openFiles = toFilePaths(v))"
    />
  </n-flex>
</template>
<style scoped></style>
