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
import {
  type FilePath,
  makeFilePath,
  ReactiveFilesystem,
} from "@/filesystem/reactive-files";
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

const props = defineProps<{
  models: DeepReadonly<VirtualModelState>[];
  fs: ReactiveFilesystem;
}>();

const emit = defineEmits<{
  update: [ids: string[], update: ObjectUpdate];
  addModel: [modelName: string, shaderName: string];
  removeModel: [ids: string[]];
  select: [vertex: FilePath];
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
let toAddModel = ref<[string, FilePath]>([
  "New Model",
  makeFilePath("new-model"),
]);
let customShader = ref<string | null>(null);
let customGraph = ref<string | null>(null);

watchEffect(() => {
  const keys = selectedKeys.value;
  if (keys.length == 0) {
    currentModel.value = null;
  } else if (keys.length == 1) {
    const model = props.models.find((model) => model.id === keys[0]);
    if (model) {
      currentModel.value = model;
      let selectVal = model.code;
      if (model.code.endsWith(".wgsl") && model.code.includes(".graph")) {
        selectVal = makeFilePath(model.code.replace(".wgsl", ""));
      }
      emit("select", selectVal);
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

const isAdding = ref<boolean>(false);
const fileNames = ref(new Set<FilePath>());
props.fs.watchFromStart((change) => {
  if (!change.key.endsWith(".wgsl")) return;
  if (change.type === "insert") {
    fileNames.value.add(change.key);
  } else if (change.type === "remove") {
    fileNames.value.delete(change.key);
  }
});

const shadersDropdown = computed<SelectMixedOption[]>(() => {
  return [...fileNames.value]
    .toSorted()
    .map(
      (fileName): SelectMixedOption => ({
        label: fileName.substring(
          0,
          fileName.valueOf().length - ".wgsl".length
        ),
        value: fileName,
      })
    )
    .concat({
      label: "New Shader...",
      value: undefined,
    });
});

function startAddModel() {
  isAdding.value = true;
  if (shadersDropdown.value.length > 0) {
    const filePath = shadersDropdown.value[0].value as FilePath;
    toAddModel.value = ["New Model", filePath ?? makeFilePath("new-shader")];
  } else {
    toAddModel.value = ["New Model", makeFilePath("new-shader")];
  }
}

function stopAddModel() {
  isAdding.value = false;
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

  if (toAddModel.value[1] === "wgsl" || toAddModel.value[1] === "graph") {
    if (!customShader || !customGraph) {
      showError("Invalid State!");
      return;
    }
    if (customGraph.value === null && customShader.value === null) {
      showError(
        "Please enter a name for the new shader or select an existing shader."
      );
      return;
    }
    if (customGraph.value !== null) {
      toAddModel.value[1] = makeFilePath(customGraph.value + ".graph");
    } else {
      toAddModel.value[1] = makeFilePath(customShader.value + ".wgsl");
    }
  }

  emit("addModel", toAddModel.value[0], toAddModel.value[1]);
  customShader.value = null;
  customGraph.value = null;
}

function addModelNew(fileEnd: "wgsl" | "graph") {
  if (!toAddModel.value) {
    showError("Illegal State!");
    return;
  }

  const fileName = toAddModel.value[0] + new Date().toLocaleString();
  toAddModel.value[1] = makeFilePath(
    fileName
      .replaceAll(" ", "-")
      .replaceAll(":", "")
      .replaceAll(".", "")
      .replaceAll(",", "")
      .replaceAll("/", "")
      .replaceAll("\\", "")
      .replaceAll("-", "_") +
      "." +
      fileEnd
  );
  emit("addModel", toAddModel.value[0], toAddModel.value[1]);
  isAdding.value = false;
  //toAddModel.value = null;
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
  <n-modal :show="isAdding" closable class="w-full sm:w-1/2 lg:w-1/3">
    <n-card
      title="Create new Model"
      closable
      :v-if="isAdding"
      :on-close="() => (isAdding = false)"
    >
      <template #action>
        <n-flex vertical>
          <n-flex justify="space-between">
            <n-text
              >Create a code-based model, if you want full freedom. Pick this
              option if you have advanced knowledge in mathematics and
              programming.</n-text
            >
            <n-text
              >Create a graph-based model, if you want a more restricted option.
              Pick this option if you have basic knowledge in
              mathematics.</n-text
            >
          </n-flex>
          <n-text type="info">optional enter model name</n-text>
          <n-input v-model:value="toAddModel[0]"></n-input>
          <n-flex justify="end">
            <n-button type="info" :on-click="() => addModelNew('graph')"
              >New Graph</n-button
            >
            <n-button type="primary" :on-click="() => addModelNew('wgsl')"
              >New Code</n-button
            >
          </n-flex>
        </n-flex>
      </template>
    </n-card>
  </n-modal>
  <n-flex justify="">
    <n-flex vertical class="mr-1 ml-1">
      <n-h3 class="underline">Scene</n-h3>
      <n-flex>
        <n-button size="small" @click="startAddModel()"> Add </n-button>
        <n-button size="small" @click="removeModel()"> Delete </n-button>
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
    <div v-if="currentModel && !isAdding" class="mr-1 ml-1">
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
            :options="shadersDropdown"
            :value="currentModel.code"
            @update-value="
              (v: string) => change('code', new ObjectUpdate([], () => v + ''))
            "
          ></n-select>
          <n-text>Instance Count</n-text>
          <NumberInput
            :value="currentModel.instanceCount"
            :step="1"
            @update="
              (v) =>
                change(
                  'instanceCount',
                  new ObjectUpdate(
                    v.path,
                    (curr) => Math.max(Math.round(v.newValue(curr)), 1),
                    v.isSliding
                  )
                )
            "
          ></NumberInput>
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
    <!--<div v-if="toAddModel" class="mr-1 ml-1">
      <n-flex>
        <n-text>Name</n-text>
        <n-input v-model:value="toAddModel[0]" type="text" clearable></n-input>
        <n-text>Shader</n-text>
        <n-select
          placeholder="Select a shader for the model"
          :options="shadersDropdown"
          v-model:value="toAddModel[1]"
          v-on:update-value="(v) => console.log(v)"
        ></n-select>
        <n-input
          v-if="toAddModel[1] === 'wgsl'"
          v-model:value="customShader"
          type="text"
          placeholder="Please input a name for the new Shader"
          clearable
        >
        </n-input>
        <n-input
          v-if="toAddModel[1] === 'graph'"
          v-model:value="customGraph"
          type="text"
          placeholder="Please input a name for the new Graph"
          clearable
        >
        </n-input>
      </n-flex>
      <br />
      <n-flex justify="space-between">
        <n-button @click="stopAddModel()">Cancel</n-button>
        <n-button @click="addModel()">Confirm</n-button>
      </n-flex>
    </div>-->
  </n-flex>
</template>
