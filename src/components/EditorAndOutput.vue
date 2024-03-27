<script setup lang="ts">
import {
  EngineFactory,
  Effect,
  WebGPUEngine,
  ShaderStore,
  Tools,
} from "@babylonjs/core";
import { MyFirstScene } from "@/scenes/MyFirstScene";
import { BaseScene } from "@/scenes/BaseScene";
import CodeEditor from "@/components/CodeEditor.vue";

import { h, ref, shallowRef, watch, watchEffect } from "vue";
import { useDebounceFn, useElementSize } from "@vueuse/core";
import { useStore } from "@/stores/store";
import { assert } from "@stefnotch/typestef/assert";
import {
  ReactiveSceneFiles,
  SceneFilesWithFilesystem,
} from "@/filesystem/scene-files";

const props = defineProps<{
  files: ReactiveSceneFiles;
  canvas: HTMLCanvasElement;
  engine: WebGPUEngine;
}>();

const store = useStore();

const canvasContainer = ref<HTMLDivElement | null>(null);
const startCode = ref(props.files.readFile("customVertexShader") ?? "");
const baseScene = shallowRef<BaseScene>(new BaseScene(props.engine));
watch(
  () => props.engine,
  (engine) => {
    baseScene.value.dispose();
    baseScene.value = new BaseScene(engine);
  }
);
const scene = shallowRef<MyFirstScene | null>(null);

const { width, height } = useElementSize(() => props.canvas);
watch(
  [width, height],
  useDebounceFn(() => {
    props.engine.resize();
  }, 100)
);

watchEffect(() => {
  canvasContainer.value?.appendChild(props.canvas);
});

// GDPR compliance https://forum.babylonjs.com/t/offer-alternative-to-babylon-js-cdn/48982
Tools.ScriptBaseUrl = "/babylon";

reloadScene();
startCode.value = props.files.readFile("customVertexShader") ?? "";

props.engine.runRenderLoop(renderLoop);
watch(
  () => props.engine,
  (engine, oldEngine) => {
    oldEngine.stopRenderLoop();
    engine.runRenderLoop(renderLoop);
  }
);

function renderLoop() {
  baseScene.value.update();
  baseScene.value.render();
}

function reloadScene() {
  // engine.value.releaseEffects();
  scene.value?.dispose();
  if (baseScene.value) {
    scene.value = new MyFirstScene(baseScene.value, props.files);
  } else {
    scene.value = null;
  }
}

const setNewCode = useDebounceFn((newCode: () => string) => {
  const value = newCode();
  if (!scene.value) {
    console.error("No scene, but want to update code");
    return;
  }

  props.files.writeFile("customVertexShader", value);

  reloadScene();
}, 500);
</script>

<template>
  <main class="min-h-full">
    <div class="flex" style="height: 90vh">
      <div
        ref="canvasContainer"
        class="self-stretch flex-1 overflow-hidden"
      ></div>
      <CodeEditor
        class="self-stretch flex-1 overflow-hidden"
        :start-code="startCode"
        :is-dark="store.isDark"
        @update="setNewCode($event)"
      >
      </CodeEditor>
    </div>
  </main>
</template>

<style scoped></style>
