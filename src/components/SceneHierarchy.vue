<script setup lang="ts">
import {
  ReadonlyQuaternion,
  ReadonlyVector3,
  type ShaderCodeRef,
  type VirtualModelState,
  VirtualScene,
  type VirtualSceneState,
} from "@/scenes/VirtualScene";
import {
  computed,
  type ComputedRef,
  h,
  ref,
  watch,
  type DeepReadonly,
} from "vue";
import { NInput, type TreeOption } from "naive-ui";
import { showInfo } from "@/notification";
import { Quaternion, Vector3 } from "@babylonjs/core";
import type { FilePath, ReactiveFiles } from "@/filesystem/reactive-files";
import { serializeScene } from "@/filesystem/scene-file";
import {
  toWriteableModelState,
  type WriteableModelState,
} from "@/sceneview/writeablemodelstate";

function setCurrentModel(model: VirtualModelState): void {
  console.log("Set current Model (" + JSON.stringify(model) + ");");
  if (!currentModel.value) currentModel = toWriteableModelState(model);
  else {
    currentModel.value.id = model.id.valueOf();
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

const emit = defineEmits({
  update<T extends keyof WriteableModelState>(
    key: T,
    value: WriteableModelState[T],
    ids: string[],
  ) {
    return true;
  },
});

const props = defineProps<{
  models: DeepReadonly<VirtualModelState>[];
  scene: VirtualScene;
  files: ReactiveFiles;
  scenePath: FilePath;
}>();

const selectedModels = ref<Array<string>>([props.models[0].id]);
const pattern = ref("");
const selectedKeys = ref<Array<string>>([]);
const checkedKeys = ref<Array<string>>([]);
const data = ref<TreeOption[]>([]);

let currentModel = toWriteableModelState(props.models[0] ?? undefined);

data.value = [...props.models.values()].map(
  (model): TreeOption => ({
    label: model.name,
    key: model.id,
  }),
);

function renderLabel({ option }: { option: TreeOption }) {
  return h("span", option.label);
}

watch(
  () => selectedKeys.value,
  () => {
    if (selectedKeys.value.length == 1) {
      const model = props.models.find(
        (model) => model.id === selectedKeys.value[0],
      );
      if (model) setCurrentModel(model);
    } else if (selectedKeys.value.length > 1) {
      const model = props.models.find(
        (model) => model.id === selectedKeys.value[0],
      );
      if (model)
        setCurrentModel({
          id: "",
          name: "...",
          code: model.code,
          position: ReadonlyVector3.fromVector3(new Vector3(0, 0, 0)),
          rotation: ReadonlyQuaternion.fromQuaternion(
            new Quaternion(0, 0, 0, 0),
          ),
          scale: 0,
        });
    }
  },
);

function change<T extends keyof WriteableModelState>(key: T) {
  if (!currentModel.value) return;
  if (Object.values(currentModel.value).includes(null)) return;
  emit("update", key, currentModel.value[key], selectedKeys.value);
  data.value = [...props.models.values()].map(
    (model): TreeOption => ({
      label: model.name,
      key: model.id,
    }),
  );
}
</script>
<template>
  <n-flex justify="">
    <div>
      <n-tree
        block-line
        cascade
        expand-on-click
        show-line
        multiple
        :show-irrelevant-nodes="false"
        :default-selected-keys="selectedModels"
        :selected-keys="selectedKeys"
        :pattern="pattern"
        :data="data"
        :checked-keys="checkedKeys"
        :render-label="renderLabel"
        @update:selected-keys="(v: string[]) => (selectedKeys = v)"
      />
    </div>
    <n-divider></n-divider>
    <div v-if="currentModel" class="mr-1 ml-1">
      <n-text>Name</n-text>
      <n-input
        v-model:value="currentModel.name"
        type="text"
        clearable
        v-on:input="change('name')"
        @change="
          (value) => {
            if (currentModel) currentModel.name = value;
          }
        "
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
<style scoped></style>
