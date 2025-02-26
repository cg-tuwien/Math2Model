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

const props = defineProps<{
  gpuDevice: GPUDevice;
  engine: WgpuEngine;
  fs: ReactiveFilesystem;
}>();

const triggerDownload = ref(false);
const minSize = ref(30.0);
const maxCurvature = ref(20.0);
const acceptablePlanarity = ref(1.0);
const includeUVs = ref(false);
const fileFormat = ref("obj");
const models: Ref<any[]> = ref([
  {
    path: "",
    name: "all",
  },
]);
const downloadTarget = ref("");
const mergeModels = ref(false);

let meshBuffer: any[] = [];
let bufferedNames: Ref<string[]> = ref([]);
let lastMeshBufferUpdate = 0;

let frameCount = 0;
function onFrame() {
  frameCount++;
  if (meshBuffer.length > 0) {
    //  debugger;
    if (lastMeshBufferUpdate < frameCount - 5) {
      exportMeshBuffer(models.value.at(-1).name);
      triggerDownload.value = false;
    }
  }
}

async function exportMeshBuffer(name: string) {
  let modelExporter = new MeshExporter3DFormats(meshBuffer, true);
  modelExporter.useUvs = includeUVs.value;
  let format = fileFormat.value;
  modelExporter.name = name;
  let fileContent = await modelExporter.exportModel(format);

  let error = fileContent == "error";
  if (error) {
    console.log("Error during exporting, format did not match any known");
    return;
  }
  //  alert(name+format);
  let filename = name + "." + format;
  let binary = fileContent.binary;
  let data = fileContent.data;
  if (!binary) saveFile(data, filename);
  else saveFileBinary(data, filename);
  let extraFile = fileContent.extraFile;
  if (extraFile) {
    saveFile(extraFile.data, name + "." + extraFile.fileExtension);
  }
  meshBuffer = [];
  bufferedNames.value = [];
  lastMeshBufferUpdate = frameCount;
}

async function readSceneFile(): Promise<string> {
  let f: File | null = await props.fs.readFile(SceneFileName);
  let text: string | undefined = await f?.text();
  assert(text !== undefined);
  return text as string;
}
async function exportMeshEarClipping(
  arrayVertices: any,
  includeUVs: boolean,
  name: string,
  buffer: boolean
) {
  console.log("ExportMeshEarClipping called");
  let looped = false;
  if (bufferedNames.value.includes(name)) {
    buffer = false;
    looped = true;
    triggerDownload.value = false;
  }

  let sceneFile = readSceneFile();
  if (buffer) {
    bufferedNames.value.push(name);
    console.log(bufferedNames);
  }
  lastMeshBufferUpdate = frameCount;
  if (!looped) {
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
  }
  if (buffer) return;
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
  exportMeshEarClipping(patches, includeUVs, name, buffer);
}

// Main function
async function main() {
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
    },
    onFrame
  );
}

main();
</script>
<template>
  <div class="absolute bg-red-100 p-2">
    <label
      >Min Size:
      <input type="range" v-model="minSize" min="0" max="30" step="0.01"
    /></label>
    <div></div>
    <label
      >Max Curvature:
      <input type="range" v-model="maxCurvature" min="0" max="5" step="0.01"
    /></label>

    <div></div>
    <label
      >Planarity Criterium:
      <input
        type="range"
        v-model="acceptablePlanarity"
        min="0.85"
        max="1"
        step="0.001"
    /></label>

    <div></div>
    <label
      >File Format:
      <select v-model="fileFormat">
        <option value="obj">obj</option>
        <option value="glb">glb</option>
      </select></label
    >

    <label
      >Target Model:
      <select v-model="downloadTarget">
        <option
          v-for="model in models"
          :value="model.path"
          v-bind:key="model.path"
        >
          {{ model.name }}
        </option>
      </select></label
    >
    <label v-if="downloadTarget == ''"
      >Merge Models<input type="checkbox" v-model="mergeModels"
    /></label>

    <div></div>
    <label>Include UVs: <input type="checkbox" v-model="includeUVs" /> </label>
    <div></div>
    <button @click="triggerDownload = true">Download</button>
  </div>
</template>
