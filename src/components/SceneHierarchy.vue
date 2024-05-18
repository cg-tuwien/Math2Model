<script setup lang="ts">
import {
  ReadonlyQuaternion,
  ReadonlyVector3,
  type VirtualModelState,
  type VirtualModelUpdate,
} from "@/scenes/VirtualScene";
import { computed, h, ref, watch, watchEffect, type DeepReadonly } from "vue";
import { NButton, NInput, type TreeOption } from "naive-ui";
import { showError } from "@/notification";
import {
  commonWriteableModelState,
  toWriteableModelState,
  type WriteableModelState,
} from "@/sceneview/writeablemodelstate";
import { ShaderFiles } from "@/filesystem/shader-files";
import { type FilePath } from "@/filesystem/reactive-files";

function setCurrentModel(model: VirtualModelState): void {
  console.log("Set current Model (" + JSON.stringify(model) + ");");
  if (!currentModel.value)
    currentModel = computed(() => toWriteableModelState(model));
  else {
    // currentModel.value.id = model.id.valueOf();
    currentModel.value.name = model.name.valueOf();
    currentModel.value.code = model.code;
    currentModel.value.posX = model.position.x.valueOf();
    currentModel.value.posY = model.position.y.valueOf();
    currentModel.value.posZ = model.position.z.valueOf();
    currentModel.value.rotX = model.rotation.x.valueOf();
    currentModel.value.rotY = model.rotation.y.valueOf();
    currentModel.value.rotZ = model.rotation.z.valueOf();
    currentModel.value.scale = model.scale.valueOf();
  }
}
import { assertUnreachable } from "@stefnotch/typestef/assert";
import { Angle, Quaternion, Tools } from "@babylonjs/core";

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
let toAddModel = ref<[string, string] | null>(null);

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
      rotation: new ReadonlyVector3(
        Tools.ToRadians(model.rotX ?? 0),
        Tools.ToRadians(model.rotY ?? 0),
        Tools.ToRadians(model.rotZ ?? 0),
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
  toAddModel.value = ["New Model", "new-shader"];
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

  emit("addModel", toAddModel.value[0], toAddModel.value[1]);
  toAddModel.value = null;
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
          <n-input-number
            v-model:value="currentModel.posX"
            clearable
            v-on:input="change('posX')"
            :show-button="false"
          ></n-input-number>
          <n-text>Position y</n-text>
          <n-input-number
            v-model:value="currentModel.posY"
            clearable
            v-on:input="change('posY')"
            :show-button="false"
          ></n-input-number>
          <n-text>Position z</n-text>
          <n-input-number
            v-model:value="currentModel.posZ"
            clearable
            v-on:input="change('posZ')"
            :show-button="false"
          ></n-input-number>
        </div>
        <div>
          <n-text>Rotation x</n-text>
          <n-input-number
            v-model:value="currentModel.rotX"
            clearable
            v-on:input="change('rotX')"
            :show-button="false"
          ></n-input-number>
          <n-text>Rotation y</n-text>
          <n-input-number
            v-model:value="currentModel.rotY"
            clearable
            v-on:input="change('rotY')"
            :show-button="false"
          ></n-input-number>
        </div>
        <div>
          <n-text>Rotation z</n-text>
          <n-input-number
            v-model:value="currentModel.rotZ"
            clearable
            v-on:input="change('rotZ')"
            :show-button="false"
          ></n-input-number>
        </div>
        <div>
          <n-text>Scale</n-text>
          <n-input-number
            v-model:value="currentModel.scale"
            clearable
            v-on:input="change('scale')"
            :show-button="false"
          ></n-input-number>
        </div>
      </n-flex>
    </div>
    <div v-if="toAddModel" class="mr-1 ml-1">
      <n-flex>
        <n-text>Name</n-text>
        <n-input v-model:value="toAddModel[0]" type="text" clearable></n-input>
        <n-text>Shader Name</n-text>
        <n-input v-model:value="toAddModel[1]" type="text" clearable></n-input>
      </n-flex>
      <br />
      <n-flex justify="space-between">
        <n-button @click="stopAddModel()">Cancel</n-button>
        <n-button @click="addModel()">Confirm</n-button>
      </n-flex>
    </div>
  </n-flex>
</template>
