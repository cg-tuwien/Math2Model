<script setup lang="ts">

import {ReadonlyQuaternion, ReadonlyVector3, type ShaderCodeRef, type VirtualModelState} from "@/scenes/VirtualScene";
import {h, ref, watch} from "vue";
import {NInput, type TreeOption} from "naive-ui";
import {showInfo} from "@/notification";
import type {Quaternion, Vector3} from "@babylonjs/core";

export interface WriteableModelState {
  id: string;
  name: string;
  position: Vector3;
  rotation: Quaternion;
  scale: number;
}

function toWriteableModelState(model: VirtualModelState | undefined): WriteableModelState | null {
  if (model === undefined)
    return null;
  return {id: model.id,
    name: model.name,
    position: model.position.toVector3(),
    rotation: model.rotation.toQuaternion(),
    scale: model.scale
  }
}

const props = defineProps<{
  models: Array<VirtualModelState>;
}>();

const selectedModels = ref<Array<string>>([ props.models[0].id ]);
const pattern = ref("");
const selectedKeys = ref<Array<string>>([]);
const checkedKeys = ref<Array<string>>([]);
const data = ref<TreeOption[]>([]);

let currentModel = toWriteableModelState(props.models.at(0));

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

</script>
<template>
  <div class="flex">
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

    <div v-if="currentModel">
      Position
      x <n-input-number
        v-model:value="currentModel.position.x"
      ></n-input-number>
      y <n-input-number
        v-model:value="currentModel.position.y"
        ></n-input-number>
      z <n-input-number
        v-model:value="currentModel.position.z"
        ></n-input-number>
    </div>
  </div>
</template>
<style scoped></style>
