<script setup lang="ts">
import type { WgpuEngine } from "@/engine/wgpu-engine";
import type { VirtualModelState } from "@/scenes/scene-state";
import type { SelectOption } from "naive-ui";
import { computed, type DeepReadonly } from "vue";

const props = defineProps<{
  engine: WgpuEngine;
  models: DeepReadonly<VirtualModelState>[];
}>();

const resetOption: SelectOption = {
  label: "Reset",
  value: "",
};

const options = computed<SelectOption[]>(() => {
  return [
    resetOption,
    ...props.models.map(
      (v): SelectOption => ({
        label: v.name,
        value: v.id,
      })
    ),
  ];
});

function selectModel(id: string) {
  const origin = [0, 0, 0] as [number, number, number];
  const position =
    id === ""
      ? origin
      : (props.models.find((v) => v.id === id)?.position.serialize() ?? origin);

  props.engine.focusOn(position);
}
</script>
<template>
  <n-select
    filterable
    placeholder="Focus on"
    @update:value="selectModel"
    :options="options"
  />
</template>
