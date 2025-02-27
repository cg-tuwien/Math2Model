<script setup lang="ts">
import { type ReactiveFilesystem } from "@/filesystem/reactive-files";
import { ref, type Ref } from "vue";
import { ExporterInstance } from "./exporter/Exporter";
import { Vector2, Vector3 } from "./exporter/VectorTypes";
import { mainExport } from "./exporter/GPUInteractionExport";
import type { WgpuEngine } from "@/engine/wgpu-engine";
import { analyzeEdges } from "./exporter/EdgeAnalysis";
import { save, saveFile, saveFileBinary } from "./exporter/FileDownload";
import { MeshExporter3DFormats } from "./exporter/MeshExporter3DFormats";
import {
  SceneFileName,
  deserializeScene,
  type SerializedScene,
} from "@/filesystem/scene-file";
import { assert } from "@stefnotch/typestef/assert";
import { useStore } from "../stores/store";
import { computed } from "vue";
import { darkTheme, lightTheme } from "naive-ui";
import { useExportStore } from "@/stores/export-store";

const store = useStore();
const theme = computed(() => (store.isDark ? darkTheme : lightTheme));
const exportStore = useExportStore();
const props = defineProps<{
  gpuDevice: GPUDevice;
  engine: WgpuEngine;
  fs: ReactiveFilesystem;
}>();

const triggerDownload = ref(false);
const minSize = ref(0.1);
const maxCurvature = ref(2.5);
const acceptablePlanarity = ref(0.999);
const includeUVs = ref(false);
const fileFormat = ref("obj");
const subdivisionSteps = ref(4);
const models: Ref<any[]> = ref([
  {
    path: "",
    name: "all",
    uuid: "",
  },
]);
const downloadTarget = ref("");
const mergeModels = ref(true);
const exportInProgress = ref(false);
const toDownload: Ref<string[]> = ref([]);

let meshBuffer: any[] = [];
let bufferedNames: Ref<string[]> = ref([]);
let lastMeshBufferUpdate = 0;

let frameCount = 0;
function onFrame() {
  frameCount++;
}

async function exportMeshBuffer(name: string) {
  if (mergeModels.value) {
    exportMeshList(meshBuffer, name);
  } else {
    meshBuffer.forEach((mesh) => {
      exportMeshList([mesh], mesh.name);
    });
  }

  meshBuffer = [];
  bufferedNames.value = [];
  lastMeshBufferUpdate = frameCount;
  exportInProgress.value = false;
  triggerDownload.value = false;
}

async function exportMeshList(meshes: any[], name: string) {
  let modelExporter = new MeshExporter3DFormats(meshes);
  modelExporter.useUvs = includeUVs.value;
  let format = fileFormat.value;
  modelExporter.name = name;
  let fileContent = await modelExporter.exportModel(format);

  let error = fileContent.error;
  if (error) {
    console.log("Error during exporting, format did not match any known");
    return;
  }

  let filename = name + "." + format;
  let binary = fileContent.binary;
  let data = fileContent.data;
  let successful = fileContent.error;
  if (fileContent.errors) {
    fileContent.errors.forEach((errorModel) => {
      alert("Did not successfully export " + errorModel);
    });
  }
  if (!binary) saveFile(data as string, filename);
  else saveFileBinary(data as Uint8Array, filename);
  let extraFile = fileContent.extraFile;
  if (extraFile) {
    saveFile(extraFile.data as string, name + "." + extraFile.fileExtension);
  }
}

async function readSceneFile(): Promise<string> {
  let f: File | null = await props.fs.readFile(SceneFileName);
  let text: string | undefined = await f?.text();
  assert(text !== undefined);
  return text as string;
}
async function accumulateMeshForExport(
  instanceMapPatches: Map<number, any>,
  includeUVs: boolean,
  uuid: string,
  buffer: boolean
) {
  let name = uuid;
  let looped = false;
  if (toDownload.value.length == 0) {
    buffer = false;
    looped = true;
  }

  let scenePromise = getScene();
  lastMeshBufferUpdate = frameCount;

  let scene = await scenePromise;
  for (const [instance_id, patches] of instanceMapPatches) {
    // Bake edge information
    let edgeInformation = analyzeEdges(patches);
    let exporterInstance: ExporterInstance = new ExporterInstance(
      patches,
      edgeInformation
    );
    exporterInstance.useUvs = includeUVs;
    exporterInstance.Run();

    let sceneModel: any = null;
    models.value.forEach((localModel) => {
      if (localModel.uuid == uuid) {
        scene.models.forEach((model) => {
          if (model.id == localModel.uuid) {
            sceneModel = model;
          }
        });
      }
    });

    assert(sceneModel != null);
    meshBuffer.push({
      name: sceneModel.name,
      verts: exporterInstance.vertPositions,
      tris: exporterInstance.tris,
      uvs: exporterInstance.uvs,
      material: sceneModel.material,
      position: sceneModel.position,
      rotation: sceneModel.rotation,
      scale: sceneModel.scale,
      instanceCount: sceneModel.instance_count,
      instance_id: instance_id,
    });
    name = sceneModel.name;
    console.log(
      "Mesh has " +
        exporterInstance.vertPositions.length +
        " vertices on instance#" +
        instance_id
    );
  }
  if (toDownload.value.length != 0) return;
  exportMeshBuffer(name);
}

async function getScene(): Promise<SerializedScene> {
  let sceneFile = readSceneFile();
  let scene = deserializeScene(await sceneFile);
  return scene;
}
function exportMeshFromPatches(
  vertexStream: Float32Array,
  includeUVs: boolean,
  name: string,
  buffer: boolean
) {
  let instance_id: Uint32Array = new Uint32Array(vertexStream.buffer);
  instance_id = instance_id.slice(4);
  let slicedStream = vertexStream.slice(4);
  let index = 0;
  let patch_verts: { vert: Vector3; uv: Vector2 }[] = [];
  let patches: Map<
    number,
    { vert: Vector3; uv: Vector2; globalIndex?: number }[][]
  > = new Map();

  while (index < slicedStream.length) {
    let vertex = slicedStream.slice(index, index + 3);
    let uv = slicedStream.slice(index + 4, index + 6);
    let instance = instance_id[index + 6];
    let latestPatch = 0;
    if (vertex[0] == 0 && vertex[1] == 0 && vertex[2] == 0) {
      let endofdata = true;
      for (let x = 0; x < 10; x++) {
        if (slicedStream[index + x] != 0) {
          endofdata = false;
        }
      }
      if (endofdata) {
        break;
      }
    }
    patch_verts.push({
      vert: new Vector3(vertex[0], vertex[1], vertex[2]),
      uv: new Vector2(uv[0], uv[1]),
    });

    index += 8;
    if (patch_verts.length != 4) {
      continue;
    }

    if (!patches.has(instance)) {
      patches.set(instance, []);
    }
    patches.get(instance)?.push(patch_verts);

    patch_verts = [];
  }

  for (const [instance_id, patchstructure] of patches) {
    for (let i = 0; i < patchstructure.length; i++) {
      for (let j = 0; j < 4; j++) {
        let p = patchstructure[i][j];
        p.globalIndex = -1;
      }
    }
  }

  accumulateMeshForExport(patches, includeUVs, name, buffer);
}

let scene = getScene();
scene.then((actualScene) => {
  actualScene.models.forEach((model) => {
    models.value.push({
      path: model.parametricShader,
      name: model.name,
      uuid: model.id,
    });
  });
});

// Main function
mainExport(
  triggerDownload,
  exportMeshFromPatches,
  props,
  {
    minSize: minSize,
    maxCurvature: maxCurvature,
    acceptablePlanarity: acceptablePlanarity,
    includeUVs: includeUVs,
    models: models,
    downloadTarget: downloadTarget,
    bufferedNames: bufferedNames,
    subdivisionSteps: subdivisionSteps,
    toDownload: toDownload,
  },
  onFrame
);

async function startExport() {
  triggerDownload.value = true;
  exportInProgress.value = true;
  toDownload.value.length = 0;
  let sceneFile = readSceneFile();
  let scene = deserializeScene(await sceneFile);
  scene.models.forEach((model) => {
    if (downloadTarget.value !== "")
      console.log(
        "Searching for download target " +
          downloadTarget.value +
          " candidate: " +
          model.id
      );
    if (downloadTarget.value == "" || model.id == downloadTarget.value)
      toDownload.value.push(model.id);
  });
  console.log("To download: ", toDownload);
}
</script>
<template :theme="theme">
  <div class="absolute bg-red-100 p-4 rounded-lg shadow-md w-80 space-y-4">
    <div class="flex items-center justify-between">
      <label>Min Size:</label>
      <div class="flex items-center space-x-2">
        <input
          type="range"
          v-model="minSize"
          min="0"
          max="30"
          step="0.001"
          :disabled="exportInProgress"
          class="w-32"
        />
        <input
          type="number"
          v-model="minSize"
          :disabled="true"
          class="w-14 text-center"
        />
      </div>
    </div>

    <div class="flex items-center justify-between">
      <label>Max Curvature:</label>
      <div class="flex items-center space-x-2">
        <input
          type="range"
          v-model="maxCurvature"
          min="0"
          max="3"
          step="0.01"
          :disabled="exportInProgress"
          class="w-32"
        />
        <input
          type="number"
          v-model="maxCurvature"
          :disabled="true"
          class="w-14 text-center"
        />
      </div>
    </div>

    <div class="flex items-center justify-between">
      <label>Planarity Criterium:</label>
      <div class="flex items-center space-x-2">
        <input
          type="range"
          v-model="acceptablePlanarity"
          min="0.97"
          max="1"
          step="0.00001"
          :disabled="exportInProgress"
          class="w-32"
        />
        <input
          type="number"
          v-model="acceptablePlanarity"
          :disabled="true"
          class="w-14 text-center"
        />
      </div>
    </div>

    <div class="flex items-center justify-between">
      <label>File Format:</label>
      <select
        v-model="fileFormat"
        :disabled="exportInProgress"
        class="w-24 p-1"
      >
        <option value="obj">obj</option>
        <option value="glb">glb</option>
      </select>
    </div>

    <div class="flex items-center justify-between">
      <label>Target Model:</label>
      <select
        v-model="downloadTarget"
        :disabled="exportInProgress"
        class="w-24 p-1"
      >
        <option v-for="model in models" :value="model.uuid" :key="model.path">
          {{ model.name }}
        </option>
      </select>
    </div>

    <div v-if="downloadTarget == ''" class="flex items-center justify-between">
      <label>Merge Model Files</label>
      <input
        type="checkbox"
        v-model="mergeModels"
        :disabled="exportInProgress"
      />
    </div>

    <div class="flex items-center justify-between">
      <label>Include UVs:</label>
      <input
        type="checkbox"
        v-model="includeUVs"
        :disabled="exportInProgress"
      />
    </div>

    <div class="flex items-center justify-between">
      <label>Division Steps:</label>
      <input
        type="range"
        v-model="subdivisionSteps"
        min="1"
        max="5"
        step="1"
        :disabled="exportInProgress"
        class="w-32"
      />
    </div>

    <button
      @click="startExport"
      :disabled="exportInProgress"
      class="w-full bg-blue-500 text-white py-2 rounded-lg hover:bg-blue-600 disabled:bg-gray-400"
    >
      Download
    </button>
    <button
      @click="exportStore.isExportMode = false"
      :disabled="exportInProgress"
      class="w-full bg-red-500 text-white py-2 rounded-lg hover:bg-blue-600 disabled:bg-gray-400"
    >
      Cancel
    </button>
  </div>
</template>
