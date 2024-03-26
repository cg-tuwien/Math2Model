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

import { h, ref, shallowRef, watch } from "vue";
import { useDebounceFn, useElementSize } from "@vueuse/core";
import { useStore } from "@/stores/store";
import { assert } from "@stefnotch/typestef/assert";
import {
  ReactiveSceneFiles,
  SceneFilesWithFilesystem,
} from "@/filesystem/scene-files";

const props = defineProps<{
  files: ReactiveSceneFiles;
}>();

const store = useStore();

const canvasContainer = ref<HTMLDivElement | null>(null);
const canvasElement = document.createElement("canvas");
canvasElement.style.width = "100%";
canvasElement.style.height = "100%";
const startCode = ref(``);
const engine = shallowRef<WebGPUEngine | null>(null);
const baseScene = shallowRef<BaseScene | null>(null);
const scene = shallowRef<MyFirstScene | null>(null);

startCode.value = props.files.readFile("customVertexShader") ?? "";

const { width, height } = useElementSize(canvasElement);
watch(
  [width, height],
  useDebounceFn(() => {
    engine.value?.resize();
  }, 100)
);

watch(canvasContainer, (container) => {
  if (!container) return;
  container.appendChild(canvasElement);
});

// GDPR compliance https://forum.babylonjs.com/t/offer-alternative-to-babylon-js-cdn/48982
Tools.ScriptBaseUrl = "/babylon";

WebGPUEngine.IsSupportedAsync.then((supported) => {
  if (!supported) {
    alert("WebGPU not supported");
  }
});

const e = new WebGPUEngine(canvasElement, {});
e.compatibilityMode = false;
e.initAsync().then(() => {
  engine.value = e;
  e.getCaps().canUseGLInstanceID = false;
  baseScene.value = new BaseScene(e);
  reloadScene();
  startCode.value = props.files.readFile("customVertexShader") ?? "";

  e.runRenderLoop(() => {
    if (baseScene.value === null) return;
    baseScene.value.update();
    baseScene.value.render();
  });
});

function reloadScene() {
  if (!engine.value) return;

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
