<script setup lang="ts">
import CodeEditor, { type KeyedCode } from "@/components/CodeEditor.vue";
import IconFolderMultipleOutline from "~icons/mdi/folder-multiple-outline";
import IconFileTreeOutline from "~icons/mdi/file-tree-outline";
import {
  ref,
  shallowRef,
  watch,
  watchEffect,
  onUnmounted,
  computed,
  h,
} from "vue";
import { useDebounceFn } from "@vueuse/core";
import { useStore } from "@/stores/store";
import {
  ReactiveFiles,
  makeFilePath,
  type FilePath,
  readOrCreateFile,
} from "@/filesystem/reactive-files";
import { showError } from "@/notification";
import {
  ReadonlyQuaternion,
  ReadonlyVector3,
  useVirtualScene,
  type VirtualModelUpdate,
} from "@/scenes/VirtualScene";
import { getOrCreateScene } from "@/filesystem/start-files";
import VirtualModel from "@/components/VirtualModel.vue";
import { assertUnreachable } from "@stefnotch/typestef/assert";
import { serializeScene } from "@/filesystem/scene-file";
import type { Engine } from "@/engine/engine";

// Unchanging props! No need to watch them.
const props = defineProps<{
  files: ReactiveFiles;
  canvas: HTMLCanvasElement;
  engine: Engine;
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
      const { sceneFile, startFile } = getOrCreateScene(props.files, scenePath);

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

const canvasContainer = ref<HTMLDivElement | null>(null);
const baseScene = shallowRef(props.engine.createBaseScene());
onUnmounted(() => {
  baseScene.value[Symbol.dispose]();
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

let stopRenderLoop = props.engine.startRenderLoop(renderLoop);
function renderLoop() {
  baseScene.value.update();
  baseScene.value.render();
}
onUnmounted(() => {
  stopRenderLoop.stop();
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

  function openFile(v: FilePath) {
    keyedCode.value = readFile(v);
  }
  function addFiles(files: Set<FilePath>) {
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
  function deleteFiles(files: Set<FilePath>) {
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
    code: computed(() => keyedCode.value),
    openFile,
    addFiles,
    renameFile,
    deleteFiles,
    setNewCode,
  };
}

type TabName = "filebrowser" | "sceneview";
function useTabs() {
  const splitSize = ref(0.2);
  const selectedTab = ref<TabName>("filebrowser");
  function renderTabIcon(name: TabName) {
    if (name === "filebrowser") {
      return h(IconFolderMultipleOutline);
    } else if (name === "sceneview") {
      return h(IconFileTreeOutline);
    } else {
      assertUnreachable(name);
    }
  }

  const lastSelectedTab = ref<TabName | null>(null);
  function toggleTabSize() {
    const isTabBig = splitSize.value > 0.01;
    if (!isTabBig) {
      splitSize.value = 0.2;
    } else if (isTabBig && lastSelectedTab.value === selectedTab.value) {
      splitSize.value = 0.0;
    }

    lastSelectedTab.value = selectedTab.value;
  }
  return {
    splitSize,
    selectedTab,
    renderTabIcon,
    toggleTabSize,
  };
}
const tabs = useTabs();

function updateScene() {
  const sceneContent = serializeScene(scene.api.value.serialize(), true);
  if (sceneContent === null) {
    showError(
      "Could not serialize scene",
      new Error("Could not serialize scene")
    );
  } else {
    props.files.writeFile(scenePath, sceneContent);
  }
}

function updateModels(ids: string[], update: VirtualModelUpdate) {
  scene.api.value.updateModels(ids, update);
  updateScene();
}

function addModel(name: string, shaderName: string | undefined) {
  if (shaderName) {
    const vertexSource = makeFilePath(shaderName + ".vert.wgsl");
    const fragmentSource = makeFilePath(shaderName + ".frag.wgsl");

    props.files.writeFile(vertexSource, HeartSphere);
    props.files.writeFile(
      fragmentSource,
      `
    varying vNormal : vec3<f32>;
    varying vUV : vec2<f32>;
    @fragment
    fn main(input : FragmentInputs) -> FragmentOutputs {
        fragmentOutputs.color = vec4<f32>(input.vUV,1.0, 1.0);
    }
`
    );

    const newModel = {
      id: crypto.randomUUID(),
      name: name,
      code: {
        vertexFile: vertexSource,
        fragmentFile: fragmentSource,
      },
      position: ReadonlyVector3.fromVector3(new Vector3(0, 0, 0)),
      rotation: ReadonlyQuaternion.identity,
      scale: 1,
    };

    scene.api.value.addModel(newModel);
    updateScene();
  }
}

function removeModel(ids: string[]) {
  for (let id of ids) {
    if (!scene.api.value.removeModel(id)) {
      showError("Could not delete model of id: " + id, null);
    }
  }

  updateScene();
}
</script>

<template>
  <main class="flex">
    <n-tabs
      type="line"
      animated
      placement="left"
      size="small"
      class="flex-1"
      v-model:value="tabs.selectedTab.value"
    >
      <n-tab
        name="filebrowser"
        :tab="tabs.renderTabIcon('filebrowser')"
        @click="tabs.toggleTabSize()"
      ></n-tab>
      <n-tab
        name="sceneview"
        :tab="tabs.renderTabIcon('sceneview')"
        @click="tabs.toggleTabSize()"
      ></n-tab>
    </n-tabs>
    <n-split
      direction="horizontal"
      style="height: 80vh"
      :max="0.75"
      :min="0"
      :default-size="0.2"
      v-model:size="tabs.splitSize.value"
    >
      <template #1>
        <div v-if="tabs.selectedTab.value === 'filebrowser'">
          <FileBrowser
            :files="props.files"
            @open-file="openFile.openFile($event)"
            @add-files="openFile.addFiles($event)"
            @rename-file="
              (oldName, newName) => openFile.renameFile(oldName, newName)
            "
            @delete-files="openFile.deleteFiles($event)"
          ></FileBrowser>
        </div>
        <div v-else-if="tabs.selectedTab.value === 'sceneview'">
          <SceneHierarchy
            :models="scene.state.value.models"
            :scene="scene.api.value"
            :files="props.files"
            :scene-path="scenePath"
            @update="(keys, update) => updateModels(keys, update)"
            @addModel="
              (modelName, shaderName) => addModel(modelName, shaderName)
            "
            @select="(vertex) => openFile.openFile(vertex)"
            @removeModel="(ids) => removeModel(ids)"
          ></SceneHierarchy>
        </div>
        <div v-else>
          <p>Unknown tab</p>
        </div>
      </template>
      <template #2>
        <div class="flex" style="height: 80vh; width: 100%">
          <div
            ref="canvasContainer"
            class="self-stretch overflow-hidden flex-1"
          ></div>
          <div v-if="baseScene.asBabylon() !== null">
            <VirtualModel
              v-for="model in scene.state.value.models"
              :key="model.id"
              :scene="baseScene.asBabylon()"
              :files="props.files"
              :model="model"
            ></VirtualModel>
          </div>
          <CodeEditor
            class="self-stretch overflow-hidden flex-1"
            :keyed-code="openFile.code.value"
            :is-dark="store.isDark"
            @update="openFile.setNewCode($event)"
          >
          </CodeEditor>
        </div>
      </template>
    </n-split>
  </main>
</template>

<style scoped></style>
