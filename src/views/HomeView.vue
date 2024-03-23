<script setup lang="ts">
import {
  EngineFactory,
  Effect,
  WebGPUEngine,
  ShaderStore,
  Tools,
} from "@babylonjs/core";
import { MyFirstScene } from "@/scenes/MyFirstScene";
import "@babylonjs/core/Engines/WebGPU/Extensions/";
import CodeEditor from "@/components/CodeEditor.vue";

import { ref, shallowRef, watch } from "vue";
import { useDebounceFn, useElementSize } from "@vueuse/core";
import { useStore } from "@/stores/store";
import { assert } from "@stefnotch/typestef/assert";
import {
  ReactiveSceneFiles,
  SceneFilesWithFilesystem,
} from "@/filesystem/scene-files";

const store = useStore();

const canvasElement = ref<HTMLCanvasElement | null>(null);
const startCode = ref(``);
const engine = shallowRef<WebGPUEngine | null>(null);
const scene = shallowRef<MyFirstScene | null>(null);
const sceneFiles = await SceneFilesWithFilesystem.create("some-key").then(
  (fs) => ReactiveSceneFiles.create(fs)
);

startCode.value = sceneFiles.readFile("customVertexShader") ?? "";

const { width, height } = useElementSize(canvasElement);
watch(
  [width, height],
  useDebounceFn(() => {
    engine.value?.resize();
  }, 100)
);

// GDPR compliance https://forum.babylonjs.com/t/offer-alternative-to-babylon-js-cdn/48982
Tools.ScriptBaseUrl = "/babylon";

WebGPUEngine.IsSupportedAsync.then((supported) => {
  if (!supported) {
    alert("WebGPU not supported");
  }
});

watch(
  canvasElement,
  (canvas) => {
    if (!canvas) return;
    engine.value?.dispose();
    const e = new WebGPUEngine(canvas, {});
    e.compatibilityMode = false;
    e.initAsync().then(() => {
      engine.value = e;
      e.getCaps().canUseGLInstanceID = false;
      reloadScene();
      startCode.value = sceneFiles.readFile("customVertexShader") ?? "";

      e.runRenderLoop(() => {
        if (scene.value === null) return;
        scene.value.frame++;
        scene.value.time += e.getDeltaTime();
        scene.value.render();
      });
    });
  },
  { immediate: true }
);

function reloadScene() {
  if (!engine.value) return;

  engine.value.releaseEffects();
  scene.value?.dispose();
  scene.value = new MyFirstScene(engine.value, sceneFiles);
}

const setNewCode = useDebounceFn((newCode: () => string) => {
  const value = newCode();
  if (!scene.value) {
    console.error("No scene, but want to update code");
    return;
  }

  sceneFiles.writeFile("customVertexShader", value);

  reloadScene();
}, 500);
</script>

<template>
  <main class="min-h-full">
    <div class="flex" style="height: 90vh">
      <canvas
        ref="canvasElement"
        class="touch-non self-stretch flex-1 overflow-hidden"
      ></canvas>
      <!-- TODO: That's a glsl shader -->
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
