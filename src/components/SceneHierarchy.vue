<script setup lang="ts">
import {
  ReadonlyEulerAngles,
  ReadonlyVector3,
  type VirtualModelState,
} from "@/scenes/scene-state";
import { computed, h, ref, watch, watchEffect, type DeepReadonly } from "vue";
import { NButton, NInput, NText, type UploadFileInfo } from "naive-ui";
import type { SelectMixedOption } from "naive-ui/es/select/src/interface";
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
import {
  NodeTreeHelper,
  makeSelectionGeneration,
  type NodePath,
  type SelectionGeneration,
  type TreeNode,
  type TreeSelection,
} from "./node-tree/NodeTreeHelper";
import { ObjectUpdate, type ObjectPathPart } from "./input/object-update";
import { useExportStore } from "@/stores/export-store";
import { computedAsync } from "@vueuse/core";

const exportStore = useExportStore();
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

const currentModelTexture = computedAsync<UploadFileInfo[]>(async () => {
  const diffuseTexture = currentModel.value?.material.diffuseTexture ?? null;
  console.log("changed", diffuseTexture);
  if (diffuseTexture === null) return [];

  const filePromise = props.fs.readFile(makeFilePath(diffuseTexture));
  if (filePromise === null) return [];
  const file = await filePromise;
  return [
    {
      id: file.name,
      name: file.name,
      status: "finished",
      file: file,
      type: file.type,
    },
  ];
}, []);

let toAddModel = ref<[string, FilePath]>([
  "New Model",
  makeFilePath("new-model"),
]);

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
  return h(
    "div",
    h(NText, null, () => [h("span", null, option.label)])
  );
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
const shaderFiles = ref(new Set<FilePath>());
props.fs.watchFromStart((change) => {
  if (!change.key.endsWith(".wgsl")) return;
  if (change.type === "insert") {
    shaderFiles.value.add(change.key);
  } else if (change.type === "remove") {
    shaderFiles.value.delete(change.key);
  }
});

const textureFiles = ref(new Set<FilePath>());
props.fs.watchFromStart((change) => {
  if (
    !change.key
      .substring(change.key.length - 6)
      .match(
        "\.(xbm|tif|jfif|pjp|apng|jpeg|heif|ico|tiff|webp|svgz|jpg|heic|gif|svg|png|bmp|pjpeg|avif)"
      )
  )
    return;
  if (change.type === "insert") {
    textureFiles.value.add(change.key);
  } else if (change.type === "remove") {
    textureFiles.value.delete(change.key);
  }
});

const shadersDropdown = computed<SelectMixedOption[]>(() => {
  return [...shaderFiles.value]
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

const texturesDropdown = computed<SelectMixedOption[]>(() => {
  const options = [...textureFiles.value].toSorted().map(
    (fileName): SelectMixedOption => ({
      label: fileName,
      value: fileName,
    })
  );
  options.unshift({
    label: "no texture",
    value: undefined,
  });
  return options;
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

function toggleExportUI() {
  exportStore.isExportMode = !exportStore.isExportMode;
}

function uploadFile(data: {
  file: UploadFileInfo;
  fileList: UploadFileInfo[];
}): boolean {
  if (data.file.file) {
    props.fs.writeBinaryFile(makeFilePath(data.file.file.name), data.file.file);
    change(
      ["material", "diffuseTexture"],
      new ObjectUpdate([], () => data.file.file?.name ?? "")
    );
    return true;
  }
  return false;
}
</script>
<template>
  <n-modal v-model:show="isAdding" closable>
    <div class="w-full max-w-2xl">
      <n-card
        title="Create new Model"
        closable
        @close="() => (isAdding = false)"
      >
        <template #action>
          <n-flex vertical>
            <n-flex justify="space-between">
              <n-text>
                Create a code-based model, if you want full freedom. A code
                editor with syntax highlighting and auto completion will be
                provided. Pick this option if you have advanced knowledge in
                programming.
              </n-text>
              <n-text>
                Create a graph-based model, if you want a more restricted
                option. A visual scripting tool, where you connect predefined
                nodes with each other will be provided. Pick this option if you
                have little to no programming experience. Some complex
                functionality is available for more experienced programmers.
              </n-text>
            </n-flex>
            <n-text type="info">optional enter model name</n-text>
            <n-input v-model:value="toAddModel[0]"></n-input>
            <n-flex justify="end">
              <n-button type="info" @click="addModelNew('graph')"
                >New Graph</n-button
              >
              <n-button type="primary" @click="addModelNew('wgsl')"
                >New Code</n-button
              >
            </n-flex>
          </n-flex>
        </template>
      </n-card>
    </div>
  </n-modal>
  <n-flex vertical class="mr-2 ml-2">
    <n-flex vertical class="mb-2">
      <n-space justify="space-between" :wrap="false">
        <span class="text-gray-600 text-xs">SCENE</span>
        <span class="whitespace-nowrap">
          <TooltippedIconButton tooltip="Add" @click="startAddModel()">
            <mdi-add />
          </TooltippedIconButton>
          <span class="m-0.5"></span>
          <TooltippedIconButton
            tooltip="Remove"
            :disabled="selectedKeys.length === 0"
            @click="removeModel()"
          >
            <mdi-trash />
          </TooltippedIconButton>
          <n-divider vertical style="margin: -10px 12px 0px 12px"></n-divider>
          <TooltippedIconButton
            tooltip="Export scene"
            @click="toggleExportUI()"
          >
            <mdi-export-variant />
          </TooltippedIconButton>
        </span>
      </n-space>

      <NodeTree
        :root="filteredData"
        v-model:selection="treeSelection"
        @setExpanded="onNodeExpand"
        @setIsSelected="onNodeSelect"
      >
        <template #node="node">
          <n-text class="mr-2"><mdi-category-outline /></n-text>
          <n-text>{{ node.label }}</n-text>
        </template>
      </NodeTree>
      <n-button size="small" @click="startAddModel()"> <mdi-add /> </n-button>
    </n-flex>
    <n-divider :style="{ margin: '0px' }"></n-divider>
    <div v-if="currentModel && !isAdding" class="mr-1 ml-1">
      <n-space justify="space-between">
        <span class="text-gray-600 text-xs">INSPECTOR</span>
        <TooltippedIconButton
          tooltip="Close"
          @click="
            treeSelection.generation = (treeSelection.generation +
              1) as SelectionGeneration
          "
        >
          <mdi-close />
        </TooltippedIconButton>
      </n-space>
      <br />
      <n-text>Name</n-text>
      <n-input
        :value="currentModel.name"
        type="text"
        clearable
        @input="(v) => change('name', new ObjectUpdate([], () => v + ''))"
      ></n-input>
      <br /><br />
      <n-flex>
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
        <div class="w-full">
          <n-text>Scale</n-text>
          <NumberInput
            :value="currentModel.scale"
            :step="0.1"
            @update="(v) => change('scale', v)"
          ></NumberInput>
        </div>
        <div class="w-full">
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
          <ColorInput
            :value="currentModel.material.color"
            @update="(v: ObjectUpdate) => change(['material', 'color'], v)"
          ></ColorInput>
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
          <ColorInput
            :value="currentModel.material.emissive"
            @update="(v: ObjectUpdate) => change(['material', 'emissive'], v)"
          ></ColorInput>
          <n-text>Texture</n-text>
          <div class="flex mb-2">
            <NumberInput
              :value="currentModel.material.textureWidth"
              :step="0.1"
              @update="(v) => change(['material', 'textureWidth'], v)"
            ></NumberInput>
            <NumberInput
              :value="currentModel.material.textureHeight"
              :step="0.1"
              @update="(v) => change(['material', 'textureHeight'], v)"
            ></NumberInput>
          </div>
          <n-select
            placeholder="Select an existing texture or upload a new one for this model"
            :options="texturesDropdown"
            v-model:value="currentModel.material.diffuseTexture"
            v-on:update-value="
              (v: string) =>
                change(
                  ['material', 'diffuseTexture'],
                  new ObjectUpdate([], () => v)
                )
            "
          ></n-select>
          <n-upload
            directory-dnd
            accept="image/*"
            :max="2"
            v-on:before-upload="uploadFile"
            :file-list="currentModelTexture"
            list-type="image-card"
            style="margin-top: 0.5rem"
            :show-remove-button="false"
          >
            <n-upload-dragger>
              <div style="margin-bottom: 12px">
                <n-icon size="48" :depth="3">
                  <mdi-archive-arrow-down />
                </n-icon>
              </div>
            </n-upload-dragger>
          </n-upload>
          <n-text style="font-size: 16px">
            Click or drag an image file to the empty area above to upload a
            texture.
          </n-text>
        </div>
      </n-flex>
    </div>
  </n-flex>
</template>
