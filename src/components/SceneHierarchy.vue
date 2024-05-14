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
  toWriteableModelState,
  type WriteableModelState,
} from "@/sceneview/writeablemodelstate";
import { ShaderFiles } from "@/filesystem/shader-files";

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
    currentModel.value.rotW = model.rotation.w.valueOf();
    currentModel.value.scale = model.scale.valueOf();
  }
}
import { assertUnreachable } from "@stefnotch/typestef/assert";

const emit = defineEmits({
  update(ids: string[], update: VirtualModelUpdate) {
    return true;
  },
  addModel(modelName: string, shaderName: string) {
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
watchEffect(() => {
  const keys = selectedKeys.value;
  if (keys.length == 1) {
    const model = props.models.find((model) => model.id === keys[0]);
    if (model) {
      currentModel.value = toWriteableModelState(model);
    }
  } else if (keys.length > 1) {
    // TODO: Multi model editing
    currentModel.value = null;
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
  } else if (
    key === "rotX" ||
    key === "rotY" ||
    key === "rotZ" ||
    key === "rotW"
  ) {
    emit("update", keys, {
      rotation: new ReadonlyQuaternion(
        model.rotX ?? 0,
        model.rotY ?? 0,
        model.rotZ ?? 0,
        model.rotW ?? 0,
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

function addModel() {
  emit("addModel", "Model 1", "model-1-shader");
}
</script>
<template>
  <n-flex justify="">
    <n-flex>
      <n-button @click="addModel()"> Add </n-button>
    </n-flex>
    <div>
      <h2 class="underline">Scene</h2>
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
    </div>
    <n-divider></n-divider>
    <div v-if="currentModel" class="mr-1 ml-1">
      <h2 class="underline">Inspector</h2>
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
          <n-text>Rotation w</n-text>
          <n-input-number
            v-model:value="currentModel.rotW"
            clearable
            v-on:input="change('rotW')"
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
  </n-flex>
</template>
