<script setup lang="ts">

import {
  ReadonlyQuaternion,
  ReadonlyVector3,
  type ShaderCodeRef,
  type VirtualModelState,
  VirtualScene, type VirtualSceneState
} from "@/scenes/VirtualScene";
import {h, ref, watch} from "vue";
import {NInput, type TreeOption} from "naive-ui";
import {showInfo} from "@/notification";
import {Quaternion, Vector3} from "@babylonjs/core";
import type {FilePath, ReactiveFiles} from "@/filesystem/reactive-files";
import {serializeScene} from "@/filesystem/scene-file";

export interface WriteableModelState {
  id: string;
  name: string;
  code: ShaderCodeRef
  posX: number;
  posY: number;
  posZ: number;
  rotX: number;
  rotY: number;
  rotZ: number;
  rotW: number;
  scale: number;
}

function toWriteableModelState(model: VirtualModelState | undefined): WriteableModelState | null {
  if (model === undefined)
    return null;
  const pos = model.position.toVector3();
  const rot = model.rotation.toQuaternion();
  return {
    id: model.id.valueOf(),
    name: model.name.valueOf(),
    code: model.code,
    posX: pos.x.valueOf(),
    posY: pos.y.valueOf(),
    posZ: pos.z.valueOf(),
    rotX: rot.x.valueOf(),
    rotY: rot.y.valueOf(),
    rotZ: rot.z.valueOf(),
    rotW: rot.w.valueOf(),
    scale: model.scale.valueOf()
  }
}

function fromWritableModelState(model: WriteableModelState): VirtualModelState {
  return {
    id: model.id,
    name: model.name,
    code: model.code,
    position: ReadonlyVector3.fromVector3(new Vector3(model.posX, model.posY, model.posZ)),
    rotation: ReadonlyQuaternion.fromQuaternion(new Quaternion(model.rotX, model.rotY, model.rotZ, model.rotW)),
    scale: model.scale
  }
}

const props = defineProps<{
  models: Array<VirtualModelState>;
  scene: VirtualScene;
  files: ReactiveFiles;
  scenePath: FilePath;
}>();

const selectedModels = ref<Array<string>>([ props.models[0].id ]);
const pattern = ref("");
const selectedKeys = ref<Array<string>>([]);
const checkedKeys = ref<Array<string>>([]);
const data = ref<TreeOption[]>([]);

let currentModel = toWriteableModelState(props.models.at(0));

const x = ref(0);

data.value = [...props.models.values()].toSorted().map(
    (model): TreeOption => ({
      label: model.name,
      key: model.id,
    })
);

function renderLabel({ option }: { option: TreeOption }) {
  return h("span", option.label);
}

watch(() => selectedKeys.value,
  () => {
      if (selectedKeys.value.length == 1) {
        currentModel = props.models.find((model) => model.id === selectedKeys.value[0]) === null ? currentModel :
            toWriteableModelState(props.models.find((model) => model.id === selectedKeys.value[0]));
      }
      return;
});

function change() {
  if (!currentModel)
    return;
  props.scene.updateModel(currentModel.id, fromWritableModelState(currentModel));
  const sceneContent = props.scene.serialize();
  props.files.writeFile(props.scenePath, serializeScene(sceneContent));
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

    <div v-if="currentModel">
      Name <n-input v-model:value="currentModel.name" clearable v-model:on-update:value="change"></n-input>
      <br /><br />
      <n-flex justify="space-between">
        <div>
          Position
          x <n-input-number
            v-model:value="currentModel.posX" v-on:input="change"
        ></n-input-number>
          Position y <n-input-number
            v-model:value="currentModel.posY" clearable v-on:input="change"
            ></n-input-number>
          Position z <n-input-number
            v-model:value="currentModel.posZ" clearable v-on:input="change"
            ></n-input-number>
        </div>
        <div>
          Rotation
          x <n-input-number
            v-model:value="currentModel.rotX" clearable v-on:input="change"
          ></n-input-number>
            Rotation y <n-input-number
              v-model:value="currentModel.rotY" clearable v-on:input="change"
          ></n-input-number>
            Rotation z <n-input-number
              v-model:value="currentModel.rotZ" clearable v-on:input="change"
          ></n-input-number>
            Rotation w <n-input-number
              v-model:value="currentModel.rotW" clearable v-on:input="change"
          ></n-input-number>
        </div>
      </n-flex>
    </div>
  </n-flex>
</template>
<style scoped></style>
