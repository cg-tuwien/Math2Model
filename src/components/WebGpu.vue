<script setup lang="ts">
// Taken from https://webgpu.github.io/webgpu-samples/?sample=rotatingCube#main.ts
// TODO: Either add attribution, or remove this file.
import {
  makeFilePath,
  type FilePath,
  type ReactiveFilesystem,
} from "@/filesystem/reactive-files";
import { ref } from "vue";
import { ExporterInstance } from "./exporter/Exporter";
import { Vector2, Vector3 } from "./exporter/VectorTypes";
import {mainExport} from "./exporter/GPUInteractionExport";
import type { WgpuEngine } from "@/engine/wgpu-engine";

const props = defineProps<{
    gpuDevice: GPUDevice;
    engine: WgpuEngine;
    fs: ReactiveFilesystem;
  }>();
const a = new Vector3(0, 1, 0);


const triggerDownload = ref(false);

const compiledShaders = ref<
  Map<
    FilePath,
    {
      lodStage: {
        shader: GPUShaderModule;
        pipeline: GPUComputePipeline;
      };
      verticesStage: {
        shader: GPUShaderModule;
        pipeline: GPUComputePipeline;
      };
    }
  >
>(new Map());



function save(blob: Blob, filename: string) {
  const blobUrl = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = blobUrl;
  a.download = filename;
  a.click();
  URL.revokeObjectURL(blobUrl);
}

function exportMeshEarClipping(arrayVertices: any) {
  let mexpstring = "o\n";
  // Prebake
  let edgeInformation = analyzeEdges(arrayVertices);
  let exporterInstance: ExporterInstance = new ExporterInstance(
    arrayVertices,
    edgeInformation
  );
  exporterInstance.Run();

  //  console.log(JSON.stringify(arrayVertices));
//  console.log(JSON.stringify(edgeInformation));

for (let vert of exporterInstance.vertPositions) {
    mexpstring += "v " + vert.x + " " + vert.y + " " + vert.z + "\n";
  }

  let t = exporterInstance.tris;
  for (let i = 0; i < t.length; i += 3) {
    mexpstring += "f " + (t[i] + 1) + " " + (t[i + 1] + 1) + " " + (t[i + 2] + 1) + "\n";
  }
  saveFile(mexpstring, "wowcool3dmodel_earclip.obj");
}
function saveFile(text: string, filename: string) {
    save(new Blob([text], { type: "text/plain" }), filename);
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
  mainExport(triggerDownload, exportMeshFromPatches, props);
}

// Define a type for a single vertex
interface Vertex {
  vert: { x: number; y: number; z: number };
  uv: { x: number; y: number };
  corner: boolean;
  globalIndex: number;
  tempIdx: number;
  side: number;
}

// Define a type for a single range
interface VertexRange {
  start: number;
  end: number;
  startVert: number;
  endVert: number;
  ipi: number;
}

// Helper function to insert ranges in sorted order
function insertSorted(rangeList: VertexRange[], range: VertexRange): void {
  const index = rangeList.findIndex((r) => r.start >= range.start);
  if (index === -1) {
    rangeList.push(range);
  } else {
    rangeList.splice(index, 0, range);
  }
}

// Helper function to create a VertexRange object
function createVertexRange(
  start: number,
  end: number,
  startVert: number,
  endVert: number,
  ipi: number
): VertexRange {
  return { start, end, startVert, endVert, ipi };
}

// Helper function to check if a range contains another range
function checkContains(range: VertexRange, other: VertexRange): boolean {
  return other.start >= range.start && other.end <= range.end;
}

// Helper function to calculate interpolation value
function getInterpolationValue(range: VertexRange, value: number): number {
  return (value - range.start) / (range.end - range.start);
}

// Helper function to interpolate between vertices
function interpolate(
  range: VertexRange,
  value: number,
  patches: Vertex[][]
): Vector3 {
  const ip = getInterpolationValue(range, value);

  const startVec = new Vector3(
    patches[range.ipi][range.startVert].vert.x,
    patches[range.ipi][range.startVert].vert.y,
    patches[range.ipi][range.startVert].vert.z
  );

  const endVec = new Vector3(
    patches[range.ipi][range.endVert].vert.x,
    patches[range.ipi][range.endVert].vert.y,
    patches[range.ipi][range.endVert].vert.z
  );

  return startVec.multiplyScalar(1 - ip).add(endVec.multiplyScalar(ip));
}

function analyzeEdges(patches: Vertex[][]): any {
  const rangesHorizontal: {
    top: Record<number, VertexRange[]>;
    bottom: Record<number, VertexRange[]>;
  } = {
    top: {},
    bottom: {},
  };

  const rangesVertical: {
    left: Record<number, VertexRange[]>;
    right: Record<number, VertexRange[]>;
  } = {
    left: {},
    right: {},
  };

  const ranges = { horizontal: rangesHorizontal, vertical: rangesVertical };
  const topRanges = rangesHorizontal["top"];
  const botRanges = rangesHorizontal["bottom"];
  const leftRanges = rangesVertical["left"];
  const rightRanges = rangesVertical["right"];

  let inpInd = 0;

  for (const patch of patches) {
    const v1 = patch[0],
      v2 = patch[1],
      v3 = patch[2],
      v4 = patch[3];
    const upper = v2.uv.y;
    const lower = v1.uv.y;
    const left = v1.uv.x;
    const right = v3.uv.x;

    const hedgeTop = createVertexRange(left, right, 1, 2, inpInd);
    const hedgeBottom = createVertexRange(left, right, 0, 3, inpInd);
    const vedgeLeft = createVertexRange(lower, upper, 0, 1, inpInd);
    const vedgeRight = createVertexRange(lower, upper, 3, 2, inpInd);

    inpInd++;

    if (!topRanges[upper]) topRanges[upper] = [];
    insertSorted(topRanges[upper], hedgeTop);

    if (!botRanges[lower]) botRanges[lower] = [];
    insertSorted(botRanges[lower], hedgeBottom);

    if (!leftRanges[left]) leftRanges[left] = [];
    insertSorted(leftRanges[left], vedgeLeft);

    if (!rightRanges[right]) rightRanges[right] = [];
    insertSorted(rightRanges[right], vedgeRight);
  }
  return ranges;
}

main();
</script>
<template>
  <div></div>
  <div class="absolute bg-red-100">
    <button @click="triggerDownload = true">Download</button>
  </div>
</template>
