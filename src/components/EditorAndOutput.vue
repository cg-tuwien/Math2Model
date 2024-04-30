<script setup lang="ts">
import {
  HemisphericLight,
  Observable,
  UniformBuffer,
  Vector3,
  type WebGPUEngine,
} from "@babylonjs/core";
import { BaseScene } from "@/scenes/BaseScene";
import CodeEditor, { type KeyedCode } from "@/components/CodeEditor.vue";

import {
  ref,
  shallowRef,
  watch,
  watchEffect,
  onUnmounted,
  readonly,
} from "vue";
import { useDebounceFn, useElementSize } from "@vueuse/core";
import { useStore } from "@/stores/store";
import {
  ReactiveFiles,
  makeFilePath,
  type FilePath,
} from "@/filesystem/reactive-files";
import { showError } from "@/notification";
import { useVirtualScene } from "@/scenes/VirtualScene";
import { getOrCreateScene } from "@/filesystem/start-files";
import { ShaderFiles } from "@/filesystem/shader-files";
import VirtualModel from "@/components/VirtualModel.vue";

// Unchanging props! No need to watch them.
const props = defineProps<{
  files: ReactiveFiles;
  canvas: HTMLCanvasElement;
  engine: WebGPUEngine;
}>();

const store = useStore();

// The underlying data
const scenePath = makeFilePath("scene.json");
const { sceneFile, startFile } = getOrCreateScene(props.files, scenePath);
const scene = useVirtualScene();
watch(
    () => props.files.fileNames.value.get(scenePath),
    () => {
        try {
            const {sceneFile, startFile} = getOrCreateScene(props.files, scenePath);

            if (sceneFile !== null) {
                scene.api.value.fromSerialized(sceneFile);
            }
        } catch (e) {
            console.log("Could not deserialize scene file.");
        }
    }
);
if (sceneFile !== null) {
  scene.api.value.fromSerialized(sceneFile);
}
const openFile = useOpenFile(startFile, props.files);

const shaderFiles = new ShaderFiles(props.files);
// The BabylonJS scene
const canvasContainer = ref<HTMLDivElement | null>(null);
const baseScene = shallowRef(new BaseScene(props.engine));
onUnmounted(() => {
  baseScene.value.dispose();
});
const updateObservable: Observable<void> = new Observable();
const globalUBO = shallowRef(new UniformBuffer(props.engine));
globalUBO.value.addUniform("iTime", 1);
globalUBO.value.addUniform("iTimeDelta", 1);
globalUBO.value.addUniform("iFrame", 1);
globalUBO.value.update();
updateObservable.add(() => {
  globalUBO.value.updateFloat("iTime", baseScene.value.time / 1000);
  globalUBO.value.updateFloat("iTimeDelta", baseScene.value.deltaTime / 1000);
  globalUBO.value.updateFloat("iFrame", baseScene.value.frame);
  globalUBO.value.update();
});
onUnmounted(() => {
  globalUBO.value.dispose();
});
let light = new HemisphericLight(
  "light1",
  new Vector3(0, 1, 0),
  baseScene.value
);
light.intensity = 0.7;
onUnmounted(() => {
  light.dispose();
});

// TODO: Gizmo
// Including
// - GPU picking
// - Dragging and then updating the filesystem (aka serialize the scene)
// Excluding
// - Switching to the shader. Instead we should have a scene tree/property editor, where the user can click on the referenced shaders to edit them.

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

props.engine.runRenderLoop(renderLoop);
function renderLoop() {
  baseScene.value.update();
  updateObservable.notifyObservers();
  baseScene.value.render();
}
onUnmounted(() => {
  props.engine.stopRenderLoop(renderLoop);
});

function useOpenFile(startFile: FilePath | null, fs: ReactiveFiles) {
  const keyedCode = ref<KeyedCode | null>(
    startFile !== null ? readFile(startFile) : null
  );

  function readFile(name: FilePath): KeyedCode | null {
    let code = fs.readFile(name);
    if (code === null) {
      return null;
    }
    return {
      id: crypto.randomUUID(),
      code,
      name,
    };
  }

  function openFiles(v: FilePath[]) {
    if (v.length > 0) {
      keyedCode.value = readFile(v[0]);
    }
  }
  function addFiles(files: FilePath[]) {
    files.forEach((file) => {
      if (fs.hasFile(file)) return;
      fs.writeFile(file, "");
    });
  }
  function renameFile(oldName: FilePath, newName: FilePath) {
    if (oldName === newName) return;
    const fileData = fs.readFile(oldName);
    if (fileData === null) return;
    fs.deleteFile(oldName);
    fs.writeFile(newName, fileData);

    if (oldName === keyedCode.value?.name) {
      keyedCode.value = readFile(newName);
    }
  }
  function deleteFiles(files: FilePath[]) {
    files.forEach((file) => {
      fs.deleteFile(file);
      if (file === keyedCode.value?.name) {
        keyedCode.value = null;
      }
    });
  }

  const setNewCode = useDebounceFn((newCode: () => string) => {
    const value = newCode();
    if (keyedCode.value === null) {
      showError("No file selected", new Error("No file selected"));
      return;
    }
    // Purposefully doesn't update the `keyedCode` value
    // This is to avoid triggering a reactivity loop when using this with the CodeEditor component
    fs.writeFile(keyedCode.value.name, value);
  }, 500);

  return {
    code: readonly(keyedCode),
    openFiles,
    addFiles,
    renameFile,
    deleteFiles,
    setNewCode,
  };
}
</script>

<template>
  <main class="min-h-full">
    <div class="flex" style="height: 70vh">
      <div
        ref="canvasContainer"
        class="self-stretch flex-1 overflow-hidden"
      ></div>
      <div>
        <VirtualModel
          v-for="model in scene.state.value.models"
          :key="model.id"
          :scene="baseScene"
          :files="props.files"
          :globalUBO="globalUBO"
          :model="model"
        ></VirtualModel>
      </div>
      <CodeEditor
        class="self-stretch flex-1 overflow-hidden"
        :keyed-code="openFile.code.value"
        :is-dark="store.isDark"
        @update="openFile.setNewCode($event)"
      >
      </CodeEditor>
    </div>
    <div class="flex" style="height: auto">
      <n-tabs type="line" animated class="ml-2">
        <n-tab-pane name="filebrowser" tab="File Browser">
          <FileBrowser
              :files="props.files"
              :open-files="
          openFile.code.value !== null ? [openFile.code.value.name] : []
        "
              @update:open-files="openFile.openFiles($event)"
              @add-files="openFile.addFiles($event)"
              @rename-file="(oldName, newName) => openFile.renameFile(oldName, newName)"
              @delete-files="openFile.deleteFiles($event)"
          ></FileBrowser>
        </n-tab-pane>
        <n-tab-pane name="sceneview" tab="Hierarchy">
          <SceneHierarchy
            :models="scene.state.value.models"
          ></SceneHierarchy>
        </n-tab-pane>
        <n-tab-pane name="addtab" tab="+">
          To be developed...
        </n-tab-pane>
      </n-tabs>
    </div>
  </main>
</template>

<style scoped></style>
