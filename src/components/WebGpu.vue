<script setup lang="ts">
// Taken from https://webgpu.github.io/webgpu-samples/?sample=rotatingCube#main.ts
// TODO: Either add attribution, or remove this file.
import { type ReactiveFilesystem } from "@/filesystem/reactive-files";
import { ref, type Ref } from "vue";
import { ExporterInstance } from "./exporter/Exporter";
import { Vector2, Vector3 } from "./exporter/VectorTypes";
import { mainExport } from "./exporter/GPUInteractionExport";
import type { WgpuEngine } from "@/engine/wgpu-engine";
import { analyzeEdges } from "./exporter/EdgeAnalysis";
import { save, saveFile, saveFileBinary } from "./exporter/FileDownload";
import { MeshExporter3DFormats } from "./exporter/MeshExporter3DFormats";
import { SceneFileName, deserializeScene } from "@/filesystem/scene-file";
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
  if(fileContent.errors)
  {
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
  arrayVertices: any,
  includeUVs: boolean,
  name: string,
  buffer: boolean
) {
  let looped = false;
  if (toDownload.value.length == 0) {
    buffer = false;
    looped = true;
  }

  let sceneFile = readSceneFile();
  lastMeshBufferUpdate = frameCount;

  // Prebake
  let edgeInformation = analyzeEdges(arrayVertices);
  let exporterInstance: ExporterInstance = new ExporterInstance(
    arrayVertices,
    edgeInformation
  );
  exporterInstance.useUvs = includeUVs;
  exporterInstance.Run();

  let scene = deserializeScene(await sceneFile);
  let sceneModel: any = null;
  models.value.forEach((localModel) => {
    if (localModel.name == name) {
      scene.models.forEach((model) => {
        if (model.id == localModel.uuid) {
          // console.log("Found info on model!")
          sceneModel = model;
          //console.log(sceneModel);
          // console.log("Equal to");
          // console.log(localModel);
        }
      });
    }
  });
  assert(sceneModel != null);
  meshBuffer.push({
    name: name,
    verts: exporterInstance.vertPositions,
    tris: exporterInstance.tris,
    uvs: exporterInstance.uvs,
    material: sceneModel.material,
    position: sceneModel.position,
    rotation: sceneModel.rotation,
    scale: sceneModel.scale,
    instanceCount: sceneModel.instance_count,
  });
  console.log("Mesh has " + exporterInstance.vertPositions.length+ " vertices")

  if (toDownload.value.length != 0) return;
  exportMeshBuffer(name);
}

function exportMeshFromPatches(
  vertexStream: Float32Array,
  includeUVs: boolean,
  name: string,
  buffer: boolean
) {
  let slicedStream = vertexStream.slice(4);
  let index = 0;
  let patch_verts = [];
  let patches: any = [];
  while (index < slicedStream.length) {
    let section = slicedStream.slice(index, index + 3);
    let section2 = slicedStream.slice(index + 4, index + 6);
    if (section[0] == 0 && section[1] == 0 && section[2] == 0) {
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
      vert: new Vector3(section[0], section[1], section[2]),
      uv: new Vector2(section2[0], section2[1]),
    });

    index += 8;
    if (patch_verts.length != 4) {
      continue;
    }

    patches.push(patch_verts);

    patch_verts = [];
  }

  for (let i = 0; i < patches.length; i++) {
    for (let j = 0; j < 4; j++) {
      let p = patches[i][j];
      patches[i][j].vert = new Vector3(p.vert.x, p.vert.y, p.vert.z);

      patches[i][j].uv = new Vector2(p.uv.x, p.uv.y);
      patches[i][j].globalIndex = -1;
    }
  }
  accumulateMeshForExport(patches, includeUVs, name, buffer);
}

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
    toDownload.value.push(model.id);
  });
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
        <option v-for="model in models" :value="model.path" :key="model.path">
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
      @click="exportStore.isExportMode = false;"
      :disabled="exportInProgress"
      class="w-full bg-red-500 text-white py-2 rounded-lg hover:bg-blue-600 disabled:bg-gray-400"
    >
      Cancel
    </button>
  </div>
</template>
