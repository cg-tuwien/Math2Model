<script setup lang="ts">
import {
  ReadonlyEulerAngles,
  ReadonlyVector3,
  type VirtualModelState,
} from "@/scenes/scene-state";
import { computed, h, ref, watchEffect, type DeepReadonly } from "vue";
import { NButton, NInput, NText } from "naive-ui";
import NumberInput from "@/components/input/NumberInput.vue";
import VectorInput from "@/components/input/VectorInput.vue";
import EulerInput from "@/components/input/EulerInput.vue";
import { showError } from "@/notification";
import { commonModelState } from "@/scenes/aggregrate-model-state";
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
import { ObjectUpdate, type ObjectPathPart } from "./input/object-update";

const emit = defineEmits({
  update(ids: string[], update: ObjectUpdate) {
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
  return h(NText, null, () => [h("span", null, option.label)]);
}

type VirtualModelPath =
  | keyof VirtualModelState
  | [keyof VirtualModelState, ...ObjectPathPart[]];
function change(path: VirtualModelPath, value: ObjectUpdate) {
  let keys = selectedKeys.value;
  if (selectedKeys.value.length === 0) {
    console.warn("No model selected");
    return;
  }
  emit("update", keys, value.addPath(path));
}
function vector3Update(
  value: ObjectUpdate<number>
): ObjectUpdate<ReadonlyVector3> {
  return new ObjectUpdate(
    [],
    (old: ReadonlyVector3) => {
      // This simply assumes that our value updates an array of 3 numbers
      let oldObj = [old.x, old.y, old.z];
      let newObj = value.applyTo(oldObj);
      return new ReadonlyVector3(newObj[0], newObj[1], newObj[2]);
    },
    value.isSliding
  );
}
function eulerAnglesUpdate(value: ObjectUpdate<number>) {
  return new ObjectUpdate(
    [],
    (old: ReadonlyVector3) => {
      // This simply assumes that our value updates an array of 3 numbers
      let oldObj = [old.x, old.y, old.z];
      let newObj = value.applyTo(oldObj);
      return new ReadonlyEulerAngles(newObj[0], newObj[1], newObj[2]);
    },
    value.isSliding
  );
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
    showError("Illegal State!");
    return;
  }
  if (!toAddModel.value[1]) {
    showError("Please add a filename for the generated shaderfiles.");
    return;
  }

  if (toAddModel.value[1] == undefined) {
    if (!customShader) {
      showError("Invalid State!");
      return;
    }
    if (customShader.value === null) {
      showError(
        "Please enter a name for the new shader or select an existing shader."
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
    showError("Select at least one Model to remove.");
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
        @input="(v) => change('name', new ObjectUpdate([], () => v + ''))"
      ></n-input>
      <br /><br />
      <n-flex justify="space-between">
        <div>
          <n-text>Position</n-text>
          <VectorInput
            :value="[
              currentModel.position.x,
              currentModel.position.y,
              currentModel.position.z,
            ]"
            @update="(v) => change('position', vector3Update(v))"
          ></VectorInput>
        </div>
        <div>
          <n-text>Rotation</n-text>
          <EulerInput
            :value="[
              currentModel.rotation.x,
              currentModel.rotation.y,
              currentModel.rotation.z,
            ]"
            @update="(v) => change('rotation', eulerAnglesUpdate(v))"
          ></EulerInput>
        </div>
        <div>
          <n-text>Scale</n-text>
          <NumberInput
            :value="currentModel.scale"
            :step="0.1"
            @update="(v) => change('scale', v)"
          ></NumberInput>
        </div>
        <div>
          <n-text>Parametric Function</n-text>
          <n-select
            placeholder="Select a shader for the model"
            :options="props.shaders"
            :value="currentModel.code"
            @update="(v) => change('code', new ObjectUpdate([], () => v + ''))"
          ></n-select>
          <n-text>Material</n-text>
          <n-text>Color</n-text>
          <VectorInput
            :value="[
              currentModel.material.color.x,
              currentModel.material.color.y,
              currentModel.material.color.z,
            ]"
            :step="0.1"
            @update="(v) => change(['material', 'color'], vector3Update(v))"
          ></VectorInput>
          <n-text>Roughness</n-text>
          <NumberInput
            :value="currentModel.material.roughness"
            :step="0.1"
            @update="(v) => change(['material', 'roughness'], v)"
          ></NumberInput>
          <n-text>Metallic</n-text>
          <NumberInput
            :value="currentModel.material.metallic"
            :step="0.1"
            @update="(v) => change(['material', 'metallic'], v)"
          ></NumberInput>
          <n-text>Emissive</n-text>
          <VectorInput
            :value="[
              currentModel.material.emissive.x,
              currentModel.material.emissive.y,
              currentModel.material.emissive.z,
            ]"
            :step="0.1"
            @update="(v) => change(['material', 'emissive'], vector3Update(v))"
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
