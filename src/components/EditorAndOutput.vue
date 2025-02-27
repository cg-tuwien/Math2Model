<script setup lang="ts">
import CodeEditor, {
  type KeyedCode,
  type Marker,
} from "@/components/CodeEditor.vue";
import { MarkerSeverity } from "monaco-editor/esm/vs/editor/editor.api";
import IconFolderMultipleOutline from "~icons/mdi/folder-multiple-outline";
import IconFileTreeOutline from "~icons/mdi/file-tree-outline";
import { ref, watchEffect, computed, h } from "vue";
import { useDebounceFn, watchImmediate } from "@vueuse/core";
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
import type { WasmModelInfo } from "parametric-renderer-core/pkg/web";
import { useErrorStore } from "@/stores/error-store";
import { syncFilesystem } from "@/engine/sync-filesystem";
import { useExportStore } from "@/stores/export-store";

import WebGpu from "@/components/WebGpu.vue";

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
props.engine.setOnShaderCompiled((shader, messages) => {
  errorsStore.setErrors(makeFilePath(shader), messages);
});

// The underlying data
const sceneFile = ref<string | null>(null);
const exportModel = ref(false);

props.fs.watchFromStart((change) => {
  if (change.key === SceneFileName) {
    // Thread safety: Ordered reads are guaranteed by readTextFile.
    props.fs.readTextFile(change.key)?.then((v) => {
      sceneFile.value = v;
    });
  }
});

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
  props.fs
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

type EditorType = "shader" | "graph";

function useOpenFile(startFile: FilePath | null, fs: ReactiveFilesystem) {
  const openedFileName = ref<FilePath | null>(startFile);
  const keyedCode = ref<KeyedCode | null>(null);
  const editorType = ref<EditorType>("shader");
  const markers = computed<Marker[]>(() => {
    if (openedFileName.value === null) return [];
    const messages = errorsStore.errors.get(openedFileName.value) ?? [];

    return messages.map((message: any) => {
      const startColumn = message.location?.line_position ?? 1;
      const endColumn = startColumn + (message.location?.length ?? 1);
      // TODO: Translate to UTF-16 ^^^
      const lineNumber = message.location?.line_number ?? 1;
      return {
        message: message.message,
        startLineNumber: lineNumber,
        startColumn,
        endColumn,
        endLineNumber: lineNumber,
        severity:
          message.message_type === "Error"
            ? MarkerSeverity.Error
            : message.message_type === "Warning"
              ? MarkerSeverity.Warning
              : MarkerSeverity.Info,
      } satisfies Marker;
    });
  });

  watchImmediate(openedFileName, (fileName) => {
    if (fileName === null) {
      keyedCode.value = null;
      return;
    }

    const id = crypto.randomUUID();
    keyedCode.value = {
      id,
      code: "",
      name: fileName,
    };

    if (fileName.endsWith(".wgsl")) {
      editorType.value = "shader";
    } else if (fileName.endsWith(".graph")) {
      editorType.value = "graph";
    } else {
      editorType.value = "shader";
    }

    // And now asynchronously load the file
    let file = fs.readTextFile(fileName);
    if (file === null) {
      showFileError("Could not read file", fileName);
      return;
    }
    file.then((v) => {
      if (keyedCode.value?.id !== id) {
        // We already opened another file
        return;
      }
      keyedCode.value = {
        id: crypto.randomUUID(),
        code: v,
        name: fileName,
      };
    });
  });

  const isReadonly = computed(() => {
    if (keyedCode.value === null) {
      return true;
    }
    return keyedCode.value.name.endsWith(".graph.wgsl");
  });

  function openFile(v: FilePath) {
    openedFileName.value = v;
  }
  function addFiles(files: Set<FilePath>) {
    files.forEach((file) => {
      if (fs.hasFile(file)) return;
      fs.writeTextFile(file, "");
    });
  }
  function renameFile(oldName: FilePath, newName: FilePath) {
    if (oldName === newName) return;
    fs.renameFile(oldName, newName);
    if (oldName === openedFileName.value) {
      openFile(newName);
    }
  }

  function deleteFiles(files: Set<FilePath>) {
    files.forEach((file) => {
      fs.deleteFile(file);
      if (file === openedFileName.value) {
        openedFileName.value = null;
      }
    });
  }

  const setNewCode = useDebounceFn((newCode: () => string) => {
    const value = newCode();
    if (keyedCode.value === null) {
      showError("No file selected");
      return;
    }
    // Keeps the ID intact, but updates the code
    // This ID scheme is used to avoid triggering recursive updates (the CodeEditor has a copy of the code)
    keyedCode.value = {
      ...keyedCode.value,
      code: value,
    };
    fs.writeTextFile(keyedCode.value.name, value);
  }, 500);

  return {
    path: computed(() => openedFileName.value),
    editorType: computed(() => editorType.value),
    isReadonly,
    code: computed(() => keyedCode.value),
    markers,
    openFile,
    addFiles,
    renameFile,
    deleteFiles,
    setNewCode,
  };
}

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
function toggleExportUI() {
  exportModel.value = !exportModel.value;
  if (exportModel.value == false) props.engine.setLodStage(null);
}
const exportStore = useExportStore();

watchImmediate(
  () => exportStore.isExportMode,
  (isExport) => {
    toggleExportUI();
    if (isExport) {
    } else {
    }
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
            <WebGpu
              v-if="exportModel"
              :gpuDevice="props.gpuDevice"
              :engine="props.engine"
              :fs="props.fs"
            ></WebGpu>
            <div class="flex h-full w-full">
              <div
                ref="canvasContainer"
                class="self-stretch overflow-hidden flex-1"
              >
                <div v-if="sceneFile == null">Missing scene.json</div>
              </div>
            </div>
          </template>
          <template #2>
            <div class="flex h-full w-full">
              <CodeEditor
                v-if="openFile.editorType.value === 'shader'"
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
                v-if="openFile.editorType.value === 'graph'"
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
