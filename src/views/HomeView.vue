<script setup lang="ts">
import {
  EngineFactory,
  Effect,
  WebGPUEngine,
  ShaderStore,
  Tools,
} from "@babylonjs/core";
import { ModelDisplayVirtualScene } from "@/scenes/ModelDisplayVirtualScene";
import { BaseScene } from "@/scenes/BaseScene";
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
const editor = ref();
const engine = shallowRef<WebGPUEngine | null>(null);
const baseScene = shallowRef<BaseScene | null>(null);
const scene = shallowRef<ModelDisplayVirtualScene | null>(null);
const sceneFiles = await SceneFilesWithFilesystem.create("some-key").then(
  (fs) => ReactiveSceneFiles.create(fs)
);

//startCode.value; = sceneFiles.readFile("customVertexShader") ?? "";

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
      reloadBaseScene();
      reloadScene();

      let count = 0;
      canvas.addEventListener("keydown", (e: any) => {
        if (e.code === "ControlLeft") {
          scene.value?.createNewMeshFromScratch("mesh"+count);
          reloadScene();
          count++;
        }
      });
      //startCode.value = sceneFiles.readFile("customVertexShader"+scene.value?.selectedObject) ?? "";
      alert("REGISTERED LISTENEER");

      e.runRenderLoop(() => {
        if (baseScene.value === null) return;
        baseScene.value.update();
        baseScene.value.render();
      });
    });
  },
  { immediate: true }
);


function reloadBaseScene() {
  if (!engine.value) return;

  baseScene.value?.dispose();
  baseScene.value = new BaseScene(engine.value);
}

function reloadScene() {
  if (!engine.value) return;

  // engine.value.releaseEffects();
  scene.value?.dispose();
  if (baseScene.value) {
    scene.value = new ModelDisplayVirtualScene(baseScene.value, sceneFiles);
  } else {
    scene.value = null;
  }
  scene?.value?.onSelectedObject.add(
      () =>
      {
        if(scene.value?.selectedObject) {
          console.log(editor.value);
          $refs
          startCode.value = sceneFiles.readFile("customVertexShader" + scene.value?.selectedObject?.value) ?? "";
        }
      });
}

const setNewCode = useDebounceFn((newCode: () => string) => {
  const value = newCode();
  if (!scene.value) {
    console.error("No scene, but want to update code");
    return;
  }
  if(scene.value?.selectedObject)
    sceneFiles.writeFile("customVertexShader"+scene.value?.selectedObject?.value, value);

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
      <CodeEditor
        ref="editor"
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
@/scenes/BaseScene
