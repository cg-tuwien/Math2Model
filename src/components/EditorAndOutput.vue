<script setup lang="ts">
import CodeEditor from "@/components/CodeEditor.vue";
import IconFolderMultipleOutline from "~icons/mdi/folder-multiple-outline";
import IconFileTreeOutline from "~icons/mdi/file-tree-outline";
import { ref, watchEffect, h, onUnmounted } from "vue";
import { useLocalStorage, watchImmediate } from "@vueuse/core";
import { useStore } from "@/stores/store";
import {
  ReactiveFilesystem,
  makeFilePath,
  type FilePath,
} from "@/filesystem/reactive-files";
import { showError, showFileError } from "@/notification";
import {
  ReadonlyEulerAngles,
  ReadonlyVector3,
  useVirtualScene,
  type VirtualModelState,
} from "@/scenes/scene-state";
import { assertUnreachable } from "@stefnotch/typestef/assert";
import {
  deserializeScene,
  SceneFileName,
  serializeScene,
} from "@/filesystem/scene-file";
import type { WgpuEngine } from "@/engine/wgpu-engine";
import HeartSphereCode from "@/../parametric-renderer-core/shaders/HeartSphere.wgsl?raw";
import BasicGraph from "@/../parametric-renderer-core/graphs/BasicGraph.graph?raw";
import BasicGraphShader from "@/../parametric-renderer-core/graphs/BasicGraphShader.graph.wgsl?raw";
import type { ObjectUpdate } from "./input/object-update";
import CodeGraph from "@/components/visual-programming/CodeGraph.vue";
import type {
  WasmFrameTime,
  WasmModelInfo,
} from "parametric-renderer-core/pkg/web";
import { useErrorStore } from "@/stores/error-store";
import { syncFilesystem } from "@/engine/sync-filesystem";
import { useExportStore } from "@/stores/export-store";
import { createFirstTimeVisitorProject } from "@/scenes/example-scenes";

import WebGpu from "@/components/WebGpu.vue";
import { useOpenFile } from "./use-open-file";
import { useFsStore } from "@/stores/fs-store";

// Unchanging props! No need to watch them.
const props = defineProps<{
  fs: ReactiveFilesystem;
  canvas: HTMLCanvasElement;
  engine: WgpuEngine;
  gpuDevice: GPUDevice;
}>();

syncFilesystem(props.fs, props.engine);

const store = useStore();
const errorsStore = useErrorStore();
const fsStore = useFsStore();
const fpsCounter = ref<WasmFrameTime>({ avg_delta_time: 0, avg_gpu_time: 0 });
{
  const timer = setInterval(() => {
    props.engine.getFrameTime().then((v) => {
      fpsCounter.value = v;
    });
  }, 300);
  onUnmounted(() => {
    clearInterval(timer);
  });
}

const isFirstTimeVisitor = useLocalStorage("is-first-time-visitor", true);

if (isFirstTimeVisitor.value && !props.fs.hasFile(SceneFileName)) {
  await createFirstTimeVisitorProject().then((v) => fsStore.addFiles(v));
}

props.engine.setOnShaderCompiled((shader, messages) => {
  errorsStore.setErrors(makeFilePath(shader), messages);
});

// The underlying data
const sceneFile = ref<string | null>(null);
const exportModel = ref(false);
props.fs.watch((change) => {
  if (change.key === SceneFileName) {
    if (change.type === "insert" || change.type === "update") {
      // Thread safety: Ordered reads are guaranteed by readTextFile.
      props.fs.readTextFile(change.key)?.then((v) => {
        sceneFile.value = v;
      });
    } else {
      sceneFile.value = null;
    }
  }
});
sceneFile.value = await props.fs.readTextFile(SceneFileName);

const scene = useVirtualScene();
watchEffect(() => {
  if (sceneFile.value === null) {
    scene.api.value.clear();
    return;
  }
  try {
    const sceneData = deserializeScene(sceneFile.value);
    scene.api.value.fromSerialized(sceneData);
  } catch (error) {
    showFileError("Could not load scene file", SceneFileName, { error });
  }
});

const openFile = useOpenFile(
  // Open the first .wgsl file if it exists
  props.fs.listFiles().find((v) => v.endsWith(".wgsl")) ?? null,
  props.fs,
  errorsStore.errors
);

const canvasContainer = ref<HTMLDivElement | null>(null);

// Attach the canvas to the DOM
watchEffect(() => {
  canvasContainer.value?.appendChild(props.canvas);
});

watchEffect(() => {
  let models = scene.state.value.models.map((v) => {
    let model: WasmModelInfo = {
      id: v.id,
      transform: {
        position: [v.position.x, v.position.y, v.position.z],
        rotation: [v.rotation.x, v.rotation.y, v.rotation.z],
        scale: v.scale,
      },
      material_info: {
        color: [v.material.color.x, v.material.color.y, v.material.color.z],
        emissive: [
          v.material.emissive.x,
          v.material.emissive.y,
          v.material.emissive.z,
        ],
        roughness: v.material.roughness,
        metallic: v.material.metallic,
        diffuse_texture: v.material.diffuseTexture,
      },
      shader_id: v.code,
      instance_count: v.instanceCount,
    };
    return model;
  });
  props.engine.updateModels(models);
});

type TabName = "filebrowser" | "sceneview";
function useTabs() {
  const defaultSplitSize = 0.1;
  const splitSize = ref(defaultSplitSize);
  const selectedTab = ref<TabName>("sceneview");
  function renderTabIcon(name: TabName) {
    if (name === "sceneview") {
      return h(IconFileTreeOutline);
    } else if (name === "filebrowser") {
      return h(IconFolderMultipleOutline);
    } else {
      assertUnreachable(name);
    }
  }

  const lastSelectedTab = ref<TabName | null>(null);
  function toggleTabSize() {
    const isTabBig = splitSize.value > 0.01;
    if (!isTabBig) {
      splitSize.value = defaultSplitSize;
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

function saveScene() {
  const sceneContent = serializeScene(scene.api.value.serialize(), true);
  if (sceneContent === null) {
    showError("Could not serialize scene");
  } else {
    props.fs.writeTextFile(SceneFileName, sceneContent);
  }
}

function updateModels(ids: string[], update: ObjectUpdate<any>) {
  scene.api.value.updateModels(ids, update);
  if (!update.isSliding) {
    saveScene();
  }
}

function addModel(name: string, shaderName: string) {
  if (shaderName) {
    let vertexSource = makeFilePath(shaderName);

    if (!props.fs.hasFile(vertexSource)) {
      if (vertexSource.endsWith(".wgsl"))
        props.fs.writeTextFile(vertexSource, HeartSphereCode);
      else if (vertexSource.endsWith(".graph")) {
        props.fs.writeTextFile(vertexSource, BasicGraph);
        vertexSource = makeFilePath(
          vertexSource.replace(".graph", ".graph.wgsl")
        );
        props.fs.writeTextFile(vertexSource, BasicGraphShader);
      }
    }

    const newModel: VirtualModelState = {
      id: crypto.randomUUID(),
      name: name,
      code: vertexSource,
      position: ReadonlyVector3.zero,
      rotation: ReadonlyEulerAngles.identity,
      scale: 1,
      material: {
        // Random material, ugly colors but anyways
        color: new ReadonlyVector3(Math.random(), Math.random(), Math.random()),
        roughness: Math.random(),
        metallic: Math.random(),
        emissive: new ReadonlyVector3(0, 0, 0),
        diffuseTexture: null,
      },
      instanceCount: 1,
    };

    scene.api.value.addModel(newModel);
    saveScene();
  }
}

function removeModel(ids: string[]) {
  for (let id of ids) {
    if (!scene.api.value.removeModel(id)) {
      showError("Could not delete model of id: " + id);
    }
  }

  saveScene();
}

function saveGraphWgsl(filePath: FilePath, content: string) {
  // Extension should be ".graph.wgsl"
  filePath = makeFilePath(filePath.replace(".graph", ".graph.wgsl"));
  props.fs.writeTextFile(filePath, content);
}
const exportStore = useExportStore();

watchImmediate(
  () => exportStore.isExportMode,
  (isExport) => {
    exportModel.value = exportStore.isExportMode;
    if (exportModel.value == false) props.engine.setLodStage(null);
  }
);
</script>

<template>
  <main class="flex flex-1 min-h-0">
    <n-tabs
      type="line"
      animated
      placement="left"
      size="small"
      class="flex-1"
      v-model:value="tabs.selectedTab.value"
    >
      <n-tab
        name="sceneview"
        :tab="tabs.renderTabIcon('sceneview')"
        @click="tabs.toggleTabSize()"
      ></n-tab>
      <n-tab
        name="filebrowser"
        :tab="tabs.renderTabIcon('filebrowser')"
        @click="tabs.toggleTabSize()"
      ></n-tab>
    </n-tabs>
    <n-split
      direction="horizontal"
      :max="0.75"
      :min="0"
      v-model:size="tabs.splitSize.value"
    >
      <template #1>
        <div class="pt-2 h-full w-full overflow-y-auto">
          <div v-if="tabs.selectedTab.value === 'sceneview'">
            <SceneHierarchy
              :models="scene.state.value.models"
              :fs="props.fs"
              @update="
                (keys: string[], update: ObjectUpdate<any>) =>
                  updateModels(keys, update)
              "
              @addModel="
                (modelName: string, shaderName: string) =>
                  addModel(modelName, shaderName)
              "
              @select="(vertex: FilePath) => openFile.openFile(vertex)"
              @removeModel="(ids: string[]) => removeModel(ids)"
            ></SceneHierarchy>
          </div>
          <div v-else-if="tabs.selectedTab.value === 'filebrowser'">
            <FileBrowser
              :fs="props.fs"
              @open-file="openFile.openFile($event)"
              @add-files="openFile.addFiles($event)"
              @rename-file="
                (oldName: FilePath, newName: FilePath) =>
                  openFile.renameFile(oldName, newName)
              "
              @delete-files="openFile.deleteFiles($event)"
            ></FileBrowser>
          </div>
          <div v-else>
            <p>Unknown tab</p>
          </div>
        </div>
      </template>
      <template #2>
        <n-split
          direction="horizontal"
          :max="0.75"
          :min="0.15"
          :default-size="0.5"
        >
          <template #1>
            <div class="flex h-full w-full">
              <div
                ref="canvasContainer"
                class="self-stretch overflow-hidden flex-1"
                v-show="sceneFile !== null"
              ></div>
              <n-card title="Missing scene file" v-if="sceneFile === null">
                <n-button type="primary" @click="saveScene()">
                  Create empty scene
                </n-button>
              </n-card>
              <div class="absolute bottom-0 text-gray-900">
                CPU {{ (fpsCounter.avg_delta_time * 1000.0).toFixed(1) }} ms /
                GPU {{ (fpsCounter.avg_gpu_time * 1000.0).toFixed(1) }} ms
              </div>
            </div>
          </template>
          <template #2>
            <div class="flex h-full w-full">
              <WebGpu
                v-if="exportModel"
                :gpuDevice="props.gpuDevice"
                :engine="props.engine"
                :fs="props.fs"
              ></WebGpu>
              <CodeEditor
                v-if="openFile.editorType.value === 'shader' && !exportModel"
                class="self-stretch overflow-hidden flex-1"
                :keyed-code="openFile.code.value"
                :is-readonly="openFile.isReadonly.value"
                :is-dark="store.isDark"
                :markers="openFile.markers.value"
                @update="openFile.setNewCode($event)"
                @graph="
                  openFile.openFile(
                    makeFilePath(
                      openFile.path.value?.replace('.wgsl', '') ?? ''
                    )
                  )
                "
              >
              </CodeEditor>
              <CodeGraph
                v-if="openFile.editorType.value === 'graph' && !exportModel"
                :fs="props.fs"
                :keyedGraph="openFile.code.value"
                @update="
                  (content) => {
                    if (openFile.path.value !== null) {
                      saveGraphWgsl(openFile.path.value, content);
                    } else {
                      console.error('Invalid state!');
                    }
                  }
                "
                @save="
                  (content) => {
                    if (openFile.path.value !== null) {
                      props.fs.writeTextFile(openFile.path.value, content);
                    } else {
                      console.error('Invalid state!');
                    }
                  }
                "
                @code="
                  () => {
                    openFile.openFile(
                      makeFilePath(openFile.path.value + '.wgsl')
                    );
                  }
                "
                ref="graphRef"
              ></CodeGraph>
            </div>
          </template>
        </n-split>
      </template>
    </n-split>
  </main>
</template>

<style scoped></style>
