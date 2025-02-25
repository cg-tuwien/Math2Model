<script setup lang="ts">
// Taken from https://webgpu.github.io/webgpu-samples/?sample=rotatingCube#main.ts
// TODO: Either add attribution, or remove this file.
import {
  type ReactiveFilesystem,
} from "@/filesystem/reactive-files";
import { ref } from "vue";
import { ExporterInstance } from "./exporter/Exporter";
import { Vector2, Vector3 } from "./exporter/VectorTypes";
import {mainExport} from "./exporter/GPUInteractionExport";
import type { WgpuEngine } from "@/engine/wgpu-engine";
import type {Vertex, VertexRange} from "./exporter/VertexType";
import { analyzeEdges } from "./exporter/EdgeAnalysis"; 
import {save, saveFile} from "./exporter/FileDownload"; 

const props = defineProps<{
    gpuDevice: GPUDevice;
    engine: WgpuEngine;
    fs: ReactiveFilesystem;
  }>();

const triggerDownload = ref(false);
const minSize = ref(30.0);
const maxCurvature = ref(20.0);

function exportMeshEarClipping(arrayVertices: any) {
  let mexpstring = "o\n";
  // Prebake
  let edgeInformation = analyzeEdges(arrayVertices);
  let exporterInstance: ExporterInstance = new ExporterInstance(
    arrayVertices,
    edgeInformation
  );
  exporterInstance.Run();

  for (let vert of exporterInstance.vertPositions) {
    mexpstring += "v " + vert.x + " " + vert.y + " " + vert.z + "\n";
  }

  let t = exporterInstance.tris;
  for (let i = 0; i < t.length; i += 3) {
    mexpstring += "f " + (t[i] + 1) + " " + (t[i + 1] + 1) + " " + (t[i + 2] + 1) + "\n";
  }
  saveFile(mexpstring, "wowcool3dmodel_earclip.obj");
}

function exportMeshFromPatches(vertexStream: Float32Array) {
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
  exportMeshEarClipping(patches);
}

// Main function
async function main() {
  mainExport(triggerDownload, exportMeshFromPatches, props,{minSize:minSize ,maxCurvature:maxCurvature});
}

main();
</script>
<template>
  <div></div>
  <div class="absolute bg-red-100 p-2">
  <label>Min Size: <input type="range" v-model="minSize" min="0" max="30" step="0.1"></label>
  <label>Max Curvature: <input type="range" v-model="maxCurvature" min="0" max="10" step="0.1"></label>
  <button @click="triggerDownload = true">Download</button></div>
</template>
