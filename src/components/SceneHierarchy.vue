<script setup lang="ts">
import {
  ReadonlyEulerAngles,
  ReadonlyVector3,
  type VirtualModelState,
  type VirtualModelUpdate,
} from "@/scenes/VirtualScene";
import { computed, h, ref, watchEffect, type DeepReadonly } from "vue";
import { NButton, NInput, type TreeOption } from "naive-ui";
import AngleInput from "@/components/input/AngleInput.vue";
import NumberInput from "@/components/input/NumberInput.vue";
import { showError } from "@/notification";
import {
  commonWriteableModelState,
  toWriteableModelState,
  type WriteableModelState,
} from "@/sceneview/writeablemodelstate";
import { type FilePath, makeFilePath } from "@/filesystem/reactive-files";
import { assertUnreachable } from "@stefnotch/typestef/assert";
import type { SelectMixedOption } from "naive-ui/es/select/src/interface";

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
  shaders: DeepReadonly<SelectMixedOption>[];
}>();

const pattern = ref("");
const selectedKeys = ref<string[]>(
  props.models.length > 0 ? [props.models[0].id] : [],
);
const checkedKeys = ref<string[]>([]);
const data = computed(() =>
  props.models.map(
    (model): TreeOption => ({
      label: model.name,
      key: model.id,
    }),
  ),
);

let currentModel = ref<WriteableModelState | null>(null);
let toAddModel = ref<[string, FilePath] | null>(null);
let customShader = ref<string | null>(null);

watchEffect(() => {
  const keys = selectedKeys.value;
  if (keys.length == 0) {
    currentModel.value = null;
  } else if (keys.length == 1) {
    const model = props.models.find((model) => model.id === keys[0]);
    if (model) {
      currentModel.value = toWriteableModelState(model);
      emit("select", model.code.vertexFile);
    }
  } else if (keys.length > 1) {
    const models: VirtualModelState[] = [];
    for (let key of keys) {
      const model = props.models.find((model) => model.id == key);
      if (model) models.push(model);
    }
    currentModel.value = commonWriteableModelState(models);
  }
});

function renderLabel({ option }: { option: TreeOption }) {
  return h("span", option.label);
}

function change(key: keyof WriteableModelState) {
  const model = currentModel.value;
  if (!model) return;
  let keys = selectedKeys.value;
  if (selectedKeys.value.length === 0) {
    console.warn("No model selected");
    return;
  }

  //const degX = new Angle(euler.x).degrees();
  //const degY = new Angle(euler.y).degrees();
  //const degZ = new Angle(euler.z).degrees();
  //console.log(degX, degY, degZ);

  if (key === "name") {
    emit("update", keys, {
      name: model.name ?? "",
    });
  } else if (key === "code") {
    emit("update", keys, {
      code: model.code ?? "",
    });
  } else if (key === "posX" || key === "posY" || key === "posZ") {
    emit("update", keys, {
      position: new ReadonlyVector3(
        model.posX ?? 0,
        model.posY ?? 0,
        model.posZ ?? 0,
      ),
    });
  } else if (key === "rotX" || key === "rotY" || key === "rotZ") {
    emit("update", keys, {
      rotation: new ReadonlyEulerAngles(
        model.rotX ?? 0,
        model.rotY ?? 0,
        model.rotZ ?? 0,
      ),
    });
  } else if (key === "scale") {
    emit("update", keys, {
      scale: model.scale ?? 0,
    });
  } else if (key === "id") {
    showError("Cannot change id", new Error("Cannot change id"));
  } else {
    assertUnreachable(key);
  }
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

  if (toAddModel.value[1] == "NEW...") {
    if (!customShader) {
      showError("Invalid State!", null);
      return;
    }
    if (customShader.value === null) {
      showError(
        "Please enter a name for the new shader or select an existing shader.",
        null,
      );
      return;
    }
    toAddModel.value[1] = makeFilePath(customShader.value + ".vert.wgsl");
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
  selectedKeys.value = [];
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
      <n-tree
        block-line
        cascade
        expand-on-click
        show-line
        multiple
        :show-irrelevant-nodes="false"
        v-model:selected-keys="selectedKeys"
        :pattern="pattern"
        :data="data"
        :checked-keys="checkedKeys"
        :render-label="renderLabel"
      />
    </n-flex>
    <n-divider></n-divider>
    <div v-if="currentModel && !toAddModel" class="mr-1 ml-1">
      <n-h3 class="underline">Inspector</n-h3>
      <n-text>Name</n-text>
      <n-input
        v-model:value="currentModel.name"
        type="text"
        clearable
        v-on:input="change('name')"
      ></n-input>
      <br /><br />
      <n-flex justify="space-between">
        <div>
          <n-text>Position x</n-text>
          <NumberInput
            v-model="currentModel.posX"
            @update:modelValue="change('posX')"
          ></NumberInput>
          <n-text>Position y</n-text>
          <NumberInput
            v-model="currentModel.posY"
            @update:modelValue="change('posY')"
          ></NumberInput>
          <n-text>Position z</n-text>
          <NumberInput
            v-model="currentModel.posZ"
            @update:modelValue="change('posZ')"
            :show-button="false"
          ></NumberInput>
        </div>
        <div>
          <n-text>Rotation x</n-text>
          <AngleInput
            v-model="currentModel.rotX"
            @update:modelValue="change('rotX')"
          ></AngleInput>
          <n-text>Rotation y</n-text>
          <AngleInput
            v-model="currentModel.rotY"
            @update:modelValue="change('rotY')"
          ></AngleInput>
          <n-text>Rotation z</n-text>
          <AngleInput
            v-model="currentModel.rotZ"
            @update:modelValue="change('rotZ')"
          ></AngleInput>
        </div>
        <div>
          <n-text>Scale</n-text>
          <NumberInput
            v-model="currentModel.scale"
            @update:modelValue="change('scale')"
          ></NumberInput>
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
