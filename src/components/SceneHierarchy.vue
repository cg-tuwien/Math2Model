<script setup lang="ts">
import {
  type VirtualModelState,
  type VirtualModelUpdate,
} from "@/scenes/VirtualScene";
import { computed, h, ref, watch, watchEffect, type DeepReadonly } from "vue";
import { NButton, NInput } from "naive-ui";
import NumberInput from "@/components/input/NumberInput.vue";
import VectorInput from "@/components/input/VectorInput.vue";
import EulerInput from "@/components/input/EulerInput.vue";
import { showError } from "@/notification";
import { commonModelState } from "@/sceneview/writeablemodelstate";
import { type FilePath, makeFilePath } from "@/filesystem/reactive-files";
import type { SelectMixedOption } from "naive-ui/es/select/src/interface";
import {
  NodeTreeHelper,
  makeSelectionGeneration,
  type NodePath,
  type SelectionGeneration,
  type TreeNode,
  type TreeSelection,
} from "./node-tree/NodeTreeHelper";

const emit = defineEmits({
  update(ids: string[], update: VirtualModelUpdate) {
    return true;
  },
  addModel(modelName: string, shaderName: string) {
    return true;
  },
  removeModel(ids: string[]) {
    return true;
  },
  select(vertex: FilePath) {
    return true;
  },
});

const props = defineProps<{
  models: DeepReadonly<VirtualModelState>[];
  shaders: SelectMixedOption[];
}>();

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
watchEffect(() => {
  const oldChildrenMap = new Map(
    data.value.children?.map((node) => [node.key, node])
  );
  data.value.children = props.models.map(
    (model): TreeNode => ({
      label: model.name,
      key: model.id,
      isExpanded: oldChildrenMap.get(model.id)?.isExpanded ?? undefined,
      isSelected: oldChildrenMap.get(model.id)?.isSelected ?? undefined,
    })
  );
});
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
const selectedKeys = computed(() => {
  const selected: string[] = [];
  const generation = treeSelection.value.generation;
  for (const [node, _] of NodeTreeHelper.visibleNodesIter(filteredData.value)) {
    if (NodeTreeHelper.isSelected(node, generation)) {
      selected.push(node.key);
    }
  }
  return selected;
});
let currentModel = ref<DeepReadonly<VirtualModelState> | null>(null);
let toAddModel = ref<[string, FilePath] | null>(null);
let customShader = ref<string | null>(null);

watchEffect(() => {
  const keys = selectedKeys.value;
  if (keys.length == 0) {
    currentModel.value = null;
  } else if (keys.length == 1) {
    const model = props.models.find((model) => model.id === keys[0]);
    if (model) {
      currentModel.value = model;
      emit("select", model.code);
    }
  } else if (keys.length > 1) {
    const models: VirtualModelState[] = [];
    for (let key of keys) {
      const model = props.models.find((model) => model.id == key);
      if (model) {
        models.push(model);
      }
    }
    currentModel.value = commonModelState(models);
  }
});

function renderLabel({ option }: { option: TreeNode }) {
  return h("span", option.label);
}

function change(value: VirtualModelUpdate) {
  let keys = selectedKeys.value;
  if (selectedKeys.value.length === 0) {
    console.warn("No model selected");
    return;
  }
  emit("update", keys, value);
}

function startAddModel() {
  if (props.shaders.length > 0) {
    const filePath = props.shaders[0].value as FilePath;
    toAddModel.value = ["New Model", filePath ?? makeFilePath("new-shader")];
  } else {
    toAddModel.value = ["New Model", makeFilePath("new-shader")];
  }
}

function stopAddModel() {
  toAddModel.value = null;
}

function addModel() {
  if (!toAddModel.value) {
    showError("Illegal State!", null);
    return;
  }
  if (!toAddModel.value[1]) {
    showError("Please add a filename for the generated shaderfiles.", null);
    return;
  }

  if (toAddModel.value[1] == undefined) {
    if (!customShader) {
      showError("Invalid State!", null);
      return;
    }
    if (customShader.value === null) {
      showError(
        "Please enter a name for the new shader or select an existing shader.",
        null
      );
      return;
    }
    toAddModel.value[1] = makeFilePath(customShader.value + ".wgsl");
  }

  emit("addModel", toAddModel.value[0], toAddModel.value[1]);
  toAddModel.value = null;
  customShader.value = null;
}

function removeModel() {
  if (selectedKeys.value.length < 1) {
    showError("Select at least one Model to remove.", null);
    return;
  }
  emit("removeModel", selectedKeys.value);
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
  <n-flex justify="">
    <n-flex vertical class="mr-1 ml-1">
      <n-h3 class="underline">Scene</n-h3>
      <n-flex>
        <n-button @click="startAddModel()"> Add </n-button>
        <n-button @click="removeModel()"> Delete </n-button>
      </n-flex>
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
    <n-divider></n-divider>
    <div v-if="currentModel && !toAddModel" class="mr-1 ml-1">
      <n-h3 class="underline">Inspector</n-h3>
      <n-text>Name</n-text>
      <n-input
        :value="currentModel.name"
        type="text"
        clearable
        @input="
          (name) =>
            change({
              name,
            })
        "
      ></n-input>
      <br /><br />
      <n-flex justify="space-between">
        <div>
          <n-text>Position</n-text>
          <VectorInput
            :modelValue="[
              currentModel.position.x,
              currentModel.position.y,
              currentModel.position.z,
            ]"
            @update:modelValue="
              (position) =>
                change({
                  position: { x: position[0], y: position[1], z: position[2] },
                })
            "
          ></VectorInput>
        </div>
        <div>
          <n-text>Rotation</n-text>
          <EulerInput
            :modelValue="[
              currentModel.rotation.x,
              currentModel.rotation.y,
              currentModel.rotation.z,
            ]"
            @update:modelValue="
              (rotation) =>
                change({
                  rotation: {
                    x: rotation[0],
                    y: rotation[1],
                    z: rotation[2],
                  },
                })
            "
          ></EulerInput>
        </div>
        <div>
          <n-text>Scale</n-text>
          <NumberInput
            :modelValue="currentModel.scale"
            @update:modelValue="(scale) => change({ scale })"
          ></NumberInput>
        </div>
        <div>
          <n-text>Material</n-text>
          <n-text>Color</n-text>
          <VectorInput
            :modelValue="[
              currentModel.material.color.x,
              currentModel.material.color.y,
              currentModel.material.color.z,
            ]"
            @update:modelValue="
              (color) =>
                change({
                  material: {
                    color: { x: color[0], y: color[1], z: color[2] },
                  },
                })
            "
          ></VectorInput>
          <n-text>Roughness</n-text>
          <NumberInput
            :modelValue="currentModel.material.roughness"
            @update:modelValue="
              (roughness) =>
                change({
                  material: { roughness },
                })
            "
          ></NumberInput>
          <n-text>Metallic</n-text>
          <NumberInput
            :modelValue="currentModel.material.metallic"
            @update:modelValue="
              (metallic) =>
                change({
                  material: { metallic },
                })
            "
          ></NumberInput>
          <n-text>Emissive</n-text>
          <VectorInput
            :modelValue="[
              currentModel.material.emissive.x,
              currentModel.material.emissive.y,
              currentModel.material.emissive.z,
            ]"
            @update:modelValue="
              (emissive) =>
                change({
                  material: {
                    emissive: {
                      x: emissive[0],
                      y: emissive[1],
                      z: emissive[2],
                    },
                  },
                })
            "
          ></VectorInput>
        </div>
      </n-flex>
    </div>
    <div v-if="toAddModel" class="mr-1 ml-1">
      <n-flex>
        <n-text>Name</n-text>
        <n-input v-model:value="toAddModel[0]" type="text" clearable></n-input>
        <n-text>Shader</n-text>
        <n-select
          placeholder="Select a shader for the model"
          :options="props.shaders"
          v-model:value="toAddModel[1]"
        ></n-select>
        <n-input
          v-if="toAddModel[1] === 'NEW...'"
          v-model:value="customShader"
          type="text"
          placeholder="Please input a name for the new Shader"
          clearable
        >
        </n-input>
      </n-flex>
      <br />
      <n-flex justify="space-between">
        <n-button @click="stopAddModel()">Cancel</n-button>
        <n-button @click="addModel()">Confirm</n-button>
      </n-flex>
    </div>
  </n-flex>
</template>
