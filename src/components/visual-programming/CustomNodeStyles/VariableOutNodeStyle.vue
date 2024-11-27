<template>
  <div
    class="node"
    :class="{ selected: data.selected }"
    :style="nodeStyles()"
    data-testid="node"
  >
    <div class="title" data-testid="title">{{ data.label }}</div>
    <!-- Outputs-->
    <div
      class="output"
      v-for="[key, output] in outputs()"
      :key="key + seed"
      :data-testid="'output-' + key"
    >
      <div class="output-title" data-testid="output-title">
        {{ output.label }}
      </div>
      <Ref
        class="output-socket"
        :emit="emit"
        :data="{
          type: 'socket',
          side: 'output',
          key: key,
          nodeId: data.id,
          payload: output.socket,
        }"
        data-testid="output-socket"
      />
    </div>
    <!-- Controls-->
    <Ref
      class="control"
      v-for="[key, control] in controls()"
      :key="key + seed"
      :emit="emit"
      :data="{ type: 'control', payload: control }"
      :data-testid="'control-' + key"
    />
    <!-- Inputs-->
    <div
      class="input"
      v-for="[key, input] in inputs()"
      :key="key + seed"
      :data-testid="'input-' + key"
    >
      <Ref
        class="input-socket"
        :emit="emit"
        :data="{
          type: 'socket',
          side: 'input',
          key: key,
          nodeId: data.id,
          payload: input.socket,
        }"
        data-testid="input-socket"
      />
      <div
        class="input-title"
        v-show="!input.control || !input.showControl"
        data-testid="input-title"
      >
        {{ input.label }}
      </div>
      <Ref
        class="input-control"
        v-show="input.control && input.showControl"
        :emit="props.emit"
        :data="{ type: 'control', payload: input.control }"
        data-testid="input-control"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, defineComponent, type DeepReadonly } from "vue";
import { Ref } from "rete-vue-plugin";

function sortByIndex(entries: any) {
  entries.sort((a: any, b: any) => {
    const ai = (a[1] && a[1].index) || 0;
    const bi = (b[1] && b[1].index) || 0;

    return ai - bi;
  });
  return entries;
}

const props = defineProps<{
  data: {
    id: string;
    label: string;
    selected: boolean;
    width: number;
    height: number;
    inputs: any;
    controls: any;
    outputs: any;
  };
  emit: any;
  seed: string | number;
}>();

function nodeStyles() {
  return {
    width: Number.isFinite(props.data.width) ? `${props.data.width}px` : "",
    height: Number.isFinite(props.data.height) ? `${props.data.height}px` : "",
  };
}

function inputs() {
  return sortByIndex(Object.entries(props.data.inputs));
}
function controls() {
  return sortByIndex(Object.entries(props.data.controls));
}
function outputs() {
  return sortByIndex(Object.entries(props.data.outputs));
}
</script>

<style lang="scss" scoped>
@use "sass:math";

.node {
  background: #4e20b3;
  border: 2px solid black;
  border-radius: 10px;
  cursor: pointer;
  box-sizing: border-box;
  width: 200px;
  height: auto;
  padding-bottom: 6px;
  position: relative;
  user-select: none;

  &:hover {
    background: #2d07b2;
  }

  &.selected {
    border-color: lightblue;
  }

  .title {
    color: white;
    font-family: sans-serif;
    font-size: 18px;
    padding: 8px;
  }

  .output {
    text-align: right;
  }

  .input {
    text-align: left;
  }

  .output-socket {
    text-align: right;
    margin-right: -1px;
    display: inline-block;
  }

  .input-socket {
    text-align: left;
    margin-left: -1px;
    display: inline-block;
  }

  .input-title,
  .output-title {
    vertical-align: middle;
    color: white;
    display: inline-block;
    font-family: sans-serif;
    font-size: 14px;
    margin: 6px;
    line-height: 16px;
  }

  .input-control {
    z-index: 1;
    width: calc(100% - #{16px + 2 * 6px});
    vertical-align: middle;
    display: inline-block;
  }

  .control {
    padding: 6px math.div(16px, 2) + 6px;
  }
}
</style>
