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

import { ref, shallowRef, watch, watchEffect, onUnmounted } from "vue";
import { useDebounceFn, useElementSize } from "@vueuse/core";
import { useStore } from "@/stores/store";
import { assert } from "@stefnotch/typestef/assert";
import {
  ReactiveSceneFiles,
  SceneFilesWithFilesystem,
} from "@/filesystem/scene-files";

// Unchanging props! No need to watch them.
const props = defineProps<{
  files: ReactiveSceneFiles;
  canvas: HTMLCanvasElement;
  engine: WebGPUEngine;
}>();

console.log("created");

const store = useStore();

const canvasContainer = ref<HTMLDivElement | null>(null);
const startCode = ref(props.files.readFile("customVertexShader") ?? "");
const baseScene = shallowRef(new BaseScene(props.engine));
const scene = shallowRef(new MyFirstScene(baseScene.value, props.files));

// Re-read the code after loading the user's scene
startCode.value = props.files.readFile("customVertexShader") ?? "";

// Attach the canvas to the DOM
watchEffect(() => {
  canvasContainer.value?.appendChild(props.canvas);
});

// Resize the engine when the canvas size changes
const { width, height } = useElementSize(() => props.canvas);
watch(
  [width, height],
  useDebounceFn(() => {
    props.engine.resize();
  }, 100)
);

// GDPR compliance https://forum.babylonjs.com/t/offer-alternative-to-babylon-js-cdn/48982
Tools.ScriptBaseUrl = "/babylon";

props.engine.runRenderLoop(renderLoop);
function renderLoop() {
  baseScene.value.update();
  baseScene.value.render();
}

function reloadScene() {
  scene.value.dispose();
  scene.value = new MyFirstScene(baseScene.value, props.files);
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

onUnmounted(() => {
  scene.value.dispose();
  baseScene.value.dispose();
});
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
