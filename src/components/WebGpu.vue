<script setup lang="ts">
// Taken from https://webgpu.github.io/webgpu-samples/?sample=rotatingCube#main.ts
// TODO: Either add attribution, or remove this file.
import type { WgpuEngine } from "@/engine/wgpu-engine";
import {
  makeFilePath,
  type FilePath,
  type ReactiveFilesystem,
} from "@/filesystem/reactive-files";
import type { LodStageBuffers } from "@/webgpu-hook";
import { onUnmounted, ref } from "vue";
import { mat4, vec3, vec2 } from "webgpu-matrix";
import LodStageWgsl from "./lod_stage.wgsl?raw";
import PrepVerticesStageWgsl from "./prep_vertices_stage.wgsl?raw";
import OutputVerticesWgsl from "./vertices_stage.wgsl?raw";
import { OBJExporter } from 'three/addons/exporters/OBJExporter.js';
import { GLTFExporter } from 'three/addons/exporters/GLTFExporter.js';
import * as THREE from 'three';
import { createHelpers } from "./webgpu-helpers";
import { array, number, string } from "zod";
import { flatten } from "naive-ui/es/_utils";

const a = new THREE.Vector3( 0, 1, 0 );

// Unchanging props! No need to watch them.
const props = defineProps<{
  gpuDevice: GPUDevice;
  engine: WgpuEngine;
  fs: ReactiveFilesystem;
}>();

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

function concatArrayBuffers(views: ArrayBufferView[]): Uint8Array {
  let length = 0;
  for (const v of views) length += v.byteLength;

  let buf = new Uint8Array(length);
  let offset = 0;
  for (const v of views) {
    const uint8view = new Uint8Array(v.buffer, v.byteOffset, v.byteLength);
    buf.set(uint8view, offset);
    offset += uint8view.byteLength;
  }
  const { concatArrayBuffers, createBufferWith } = createHelpers(props.gpuDevice);

  return buf;
}

function createBufferWith(
  contents: Uint8Array,
  usage: GPUBufferUsageFlags,
  size?: number
) {
  const buffer = props.gpuDevice.createBuffer({
    size: size ?? contents.byteLength,
    usage,
    mappedAtCreation: true,
  });
  new Uint8Array(buffer.getMappedRange()).set(contents);
  buffer.unmap();
  return buffer;
}
function saveFile( text: string, filename: string ) {
  save( new Blob( [ text ], { type: 'text/plain' } ), filename );
}

function save( blob: Blob, filename: string ) {
  const blobUrl = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = blobUrl;
  a.download = filename;
  a.click();
  URL.revokeObjectURL(blobUrl);
}




function exportMeshEarClipping(arrayVertices: any)
{
  let vertices = new Float32Array(arrayVertices.length*3);
  let inds = new Uint32Array(arrayVertices.length*6);
  let index = 0;
  let mexpstring = "o\n";
  let vertcount = 0;
  let zeros = 8;
  let vertices_remapping = new Array(arrayVertices.length);
  let vertex_index = 0;
  let output_uv_toggle: boolean = false;
  // Remap vertices into tris
  let arrayVertices2 = new Array();
  let map_verts = new Map();
  
  // Prebake 
  
  
  let edgeInformation = analyzeEdges(arrayVertices); 
  // console.log("EDGE INFO: ");
  // console.log(edgeInformation);
  let outvertices = [];
  let outtriangles = [];
  let vertoffset = 0;
  console.log("AVLENGTH: " + arrayVertices.length);

  console.log("Unity stuff: ");
  console.log(JSON.stringify(arrayVertices));
  console.log(JSON.stringify(edgeInformation));
  for (const patch of arrayVertices) {
    let v1 = patch[0], v2 = patch[1], v3 = patch[2], v4 = patch[3];
    const upper: number = v2.uv.y;
    const lower: number = v1.uv.y;
    const left : number= v1.uv.x;
    const right: number = v3.uv.x;
    
    let leftEdge = new Array();
    let rightEdge = new Array();
    let bottomEdge = new Array();
    let topEdge = new Array();

    v1 = {side: 3, vert: v1};
    v2 = {side: 0, vert: v2};
    v3 = {side: 1, vert: v3};
    v4 = {side: 2, vert: v4};
    
    leftEdge.push(v1);
    topEdge.push(v2);
    rightEdge.push(v3);
    bottomEdge.push(v4);
    //debugger;
    
    
    let leftEdgeInfo = edgeInformation.vertical.right[left];
    let rightEdgeInfo = edgeInformation.vertical.left[right];
    let upperEdgeInfo = edgeInformation.horizontal.bottom[upper];
    let lowerEdgeInfo = edgeInformation.horizontal.top[lower];
    
    
    processEdges(leftEdgeInfo,lower,upper,leftEdge,arrayVertices,0);
    
    processEdges2(rightEdgeInfo,lower,upper,rightEdge,arrayVertices,1);
    
    processEdges(upperEdgeInfo,left,right,topEdge,arrayVertices,2);
    
    //debugger;
    processEdges2(lowerEdgeInfo,left,right,bottomEdge,arrayVertices,3);
    bottomEdge = bottomEdge.reverse();
    rightEdge = rightEdge.reverse();
    let vertices : THREE.Vector3[]= [];
    let square_edges = leftEdge.concat(topEdge).concat(rightEdge).concat(bottomEdge);
    let square_edge_indices : number[] = [];
    let index = 0;

    
    if(square_edges.length !== 4)
      continue;
    for(let vert of square_edges)
    {
//      console.log(vert);
      mexpstring += "v " + vert.vert.uv.x + " " + vert.vert.uv.y + " " + 0 + "\n";
      vertices.push(vert.vertex);
      square_edge_indices.push(index++);
    }
    let triangles: Array<number> = [];
    
    while(square_edges.length > 2)
    {
      for(let i = 0; i < square_edges.length; i++)
      {
        let sideC: number = square_edges[i].side;
        let sideB: number = square_edges[(i+1)%square_edges.length].side;
        let sideA: number = square_edges[(i+2)%square_edges.length].side;
        if (!(sideC === sideB && sideB === sideA)) {
          let v1 = square_edge_indices[i]+vertoffset;
          let v2 = square_edge_indices[(i+1)%square_edges.length]+vertoffset;
          let v3 = square_edge_indices[(i+2)%square_edges.length]+vertoffset;
          let line = "f " + (v1+1) + " " + (v2+1) + " " + (v3+1) + "\n";
          mexpstring+=line;
          triangles.push(v1,v2,v3);
          square_edges.splice(i+1,1);
          square_edge_indices.splice(i+1,1);
          break;
        }
        else
        {
          continue;
        }
      }
    }
    vertoffset += square_edge_indices.length;

  }
  saveFile( mexpstring,"wowcool3dmodel_earclip.obj" );
}

/**
 * Processes edge information and populates the targetEdges array within a specified range.
 * @param {Array} edgeInfoList - Array of edge information objects (VertexRange).
 * @param {number} lowerBound - The lower threshold value for filtering edges.
 * @param {number} upperBound - The upper threshold value for filtering edges.
 * @param {Array} targetEdges - The array to be populated with vertices.
 * @param {Array} vertexArray - Array of vertex data indexed by ipi and endVert.
 */
 function processEdges(edgeInfoList: Array<VertexRange>, lowerBound: number, upperBound: number, targetEdges: any, patches:any,side:number) {
  if(!edgeInfoList)
  {
    return;
  }
  for (let i = 0; i < edgeInfoList.length; i++) {
      let edgeInfo = edgeInfoList[i]; // VertexRange object
      if (edgeInfo.end > lowerBound && edgeInfo.end < upperBound) {
          // Push the appropriate vertex to targetEdges
          targetEdges.push({vert: patches[edgeInfo.ipi][edgeInfo.endVert],side:side});
      }
  }
}

function processEdges2(edgeInfoList: Array<VertexRange>, lowerBound: number, upperBound: number, targetEdges: any, patches:any,side:number) {
  if(!edgeInfoList)
  {
    return;
  }
  for (let i = 0; i < edgeInfoList.length; i++) {
      let edgeInfo = edgeInfoList[i]; // VertexRange object
      if (edgeInfo.start > lowerBound && edgeInfo.start < upperBound) {
          // Push the appropriate vertex to targetEdges
          targetEdges.push({vert: patches[edgeInfo.ipi][edgeInfo.startVert],side:side});
      }
  }
}

function exportMeshFromPatches(vertexStream: Float32Array) {
  const uintArray = new Uint32Array(vertexStream.buffer);

  let capacity = uintArray[1];

  let slicedStream = vertexStream.slice(4);
  let index = 0;
  let arrayVertices = [];
  let patch_verts = [];
  let patches: any = [];
  while (index < slicedStream.length)
  {
    let section = slicedStream.slice(index,index+3);
    let section2 = slicedStream.slice(index+4,index+6);
    if(section[0] == 0 && section[1] == 0 && section[2] == 0)
    {
      let endofdata = true;
      for(let x = 0; x < 10; x++)
      {
        if(slicedStream[index+x] != 0)
        {
          endofdata = false;
        }
      }
      if(endofdata)
      {
        break;
      }
    }
    patch_verts.push({
      vert: new THREE.Vector3(section[0],section[1],section[2]),
      uv: new THREE.Vector2(section2[0],section2[1])
    });

    index+=8;
    if(patch_verts.length != 4)
    {
      continue;
    }
    
    patches.push(
      patch_verts
    );
    //console.log(patches);
    patch_verts = [];
  }
  
  //console.log(JSON.stringify(patches) );
  //patches = fixPatchSeams(patches);
  //debugger;
  for(let i = 0; i < patches.length; i++)
  {
    for(let j = 0; j < 4; j++)
    {
      let p = patches[i][j];
      patches[i][j].vert = new THREE.Vector3(p.vert.x,p.vert.y,p.vert.z);
      
      patches[i][j].uv = new THREE.Vector2(p.uv.x,p.uv.y);
    }
  }
  exportMeshEarClipping(patches);
}

// QuadTree implementation for storing patches in TypeScript
class QuadTree {
  private boundary: Rectangle;
  private capacity: number;
  private patches: Patch[]; // Stores patches with UVs and world coordinates
  private divided: boolean;
  private northwest?: QuadTree;
  private northeast?: QuadTree;
  private southwest?: QuadTree;
  private southeast?: QuadTree;

  constructor(boundary: Rectangle, capacity: number) {
    this.boundary = boundary; // Rectangle representing the boundary of this quad
    this.capacity = capacity; // Maximum number of patches before splitting
    this.patches = []; // Patches stored in this node
    this.divided = false; // Whether the node has been subdivided
  }

  // Subdivide this quad tree into four children
  private subdivide(): void {
    const { x, y, w, h } = this.boundary;

    const nw = new Rectangle(x, y, w / 2, h / 2);
    const ne = new Rectangle(x + w / 2, y, w / 2, h / 2);
    const sw = new Rectangle(x, y + h / 2, w / 2, h / 2);
    const se = new Rectangle(x + w / 2, y + h / 2, w / 2, h / 2);

    this.northwest = new QuadTree(nw, this.capacity);
    this.northeast = new QuadTree(ne, this.capacity);
    this.southwest = new QuadTree(sw, this.capacity);
    this.southeast = new QuadTree(se, this.capacity);

    this.divided = true;
  }

  // Insert a patch into the quad tree
  public insert(patch: Patch): boolean {
    for (let uv of patch.uvs) {
      if (!this.boundary.contains(uv)) {
        return false; // Patch is outside the boundary of this quad
      }
    }

    // If the patch fits entirely within this node and this node is not subdivided
    if (!this.divided) {
      if (this.patches.length < this.capacity) {
        this.patches.push(patch);
        return true;
      } else {
        this.subdivide();
      }
    }

    // Try to insert the patch into a child node
    if (this.northwest?.insert(patch)) return true;
    if (this.northeast?.insert(patch)) return true;
    if (this.southwest?.insert(patch)) return true;
    if (this.southeast?.insert(patch)) return true;

    // If no child node can take the patch, it stays here
    this.patches.push(patch);
    return true;
  }

  // Query for patches within a specific range
  public query(range: Rectangle, found: Patch[] = []): Patch[] {
    if (!this.boundary.intersects(range)) {
      return found; // No intersection, return empty list
    }

    for (let patch of this.patches) {
      if (patch.uvs.some(uv => range.contains(uv))) {
        found.push(patch);
      }
    }

    if (this.divided) {
      this.northwest?.query(range, found);
      this.northeast?.query(range, found);
      this.southwest?.query(range, found);
      this.southeast?.query(range, found);
    }

    return found;
  }
}

// Helper class to represent a rectangle boundary
class Rectangle {
  public x: number;
  public y: number;
  public w: number;
  public h: number;

  constructor(x: number, y: number, w: number, h: number) {
    this.x = x; // Top-left x
    this.y = y; // Top-left y
    this.w = w; // Width
    this.h = h; // Height
  }

  public contains(point: { x: number; y: number }): boolean {
    return (
      point.x >= this.x &&
      point.x < this.x + this.w &&
      point.y >= this.y &&
      point.y < this.y + this.h
    );
  }

  public intersects(range: Rectangle): boolean {
    return !(
      range.x > this.x + this.w ||
      range.x + range.w < this.x ||
      range.y > this.y + this.h ||
      range.y + range.h < this.y
    );
  }
}

// Patch structure
interface Patch {
  uvs: { x: number; y: number }[]; // Array of 4 UV coordinates
  worldCoords: { x: number; y: number; z: number }[]; // Corresponding world coordinates
}



async function main() {
  const _adapter = await navigator.gpu.requestAdapter(); // Wait for Rust backend to be in business
  const device = props.gpuDevice;

  const sceneUniformsLayout = device.createBindGroupLayout({
    entries: [
      {
        binding: 0,
        visibility: GPUShaderStage.COMPUTE,
        buffer: {
          type: "uniform",
        },
      },
      {
        binding: 1,
        visibility: GPUShaderStage.COMPUTE,
        buffer: {
          type: "uniform",
        },
      },
      {
        binding: 2,
        visibility: GPUShaderStage.COMPUTE,
        buffer: {
          type: "uniform",
        },
      },
    ],
  });
  const layout1 = device.createBindGroupLayout({
    entries: [
      {
        binding: 0,
        visibility: GPUShaderStage.COMPUTE,
        buffer: {
          type: "uniform",
        },
      },
      {
        binding: 1,
        visibility: GPUShaderStage.COMPUTE,
        buffer: {
          type: "storage",
        },
      },
      {
        binding: 2,
        visibility: GPUShaderStage.COMPUTE,
        buffer: {
          type: "storage",
        },
      },
      {
        binding: 3,
        visibility: GPUShaderStage.COMPUTE,
        buffer: {
          type: "storage",
        },
      },
      {
        binding: 4,
        visibility: GPUShaderStage.COMPUTE,
        buffer: {
          type: "storage",
        },
      },
      {
        binding: 5,
        visibility: GPUShaderStage.COMPUTE,
        buffer: {
          type: "storage",
        },
      },
    ],
  });
  const layout2 = device.createBindGroupLayout({
    entries: [
      {
        binding: 0,
        visibility: GPUShaderStage.COMPUTE,
        buffer: {
          type: "storage",
        },
      },
      {
        binding: 1,
        visibility: GPUShaderStage.COMPUTE,
        buffer: {
          type: "read-only-storage",
        },
      },
      {
        binding: 2,
        visibility: GPUShaderStage.COMPUTE,
        buffer: {
          type: "storage",
        },
      },
      {
        binding: 3,
        visibility: GPUShaderStage.COMPUTE,
        buffer: {
          type: "uniform",
        },
      },
    ],
  });

  const prepVerticesBinding0 = device.createBindGroupLayout({
    entries: [
      {
        binding: 0,
        visibility: GPUShaderStage.COMPUTE,
        buffer: {
          type: "read-only-storage",
        },
      },
      {
        binding: 1,
        visibility: GPUShaderStage.COMPUTE,
        buffer: {
          type: "storage",
        },
      },
    ],
  });

  const vertexOutputLayout = device.createBindGroupLayout({
    entries: [
      //{
      //  binding: 0,
      //  visibility: GPUShaderStage.COMPUTE,
      //  buffer: {
      //    type: "uniform",
      //  },
      //},
      {
        binding: 1,
        visibility: GPUShaderStage.COMPUTE,
        buffer: {
          type: "read-only-storage",
        },
      },
      {
        binding: 2,
        visibility: GPUShaderStage.COMPUTE,
        buffer: {
          type: "storage",
        },
      },
    ],
  });

  const pipelineLayout = device.createPipelineLayout({
    bindGroupLayouts: [sceneUniformsLayout, layout1, layout2],
  });

  const prepVerticesStageLayout = device.createPipelineLayout({
    bindGroupLayouts: [prepVerticesBinding0],
  });
  const outputVerticesLayout = device.createPipelineLayout({
    bindGroupLayouts: [sceneUniformsLayout, vertexOutputLayout],
  });

  function makeShaderFromCodeAndPipeline(
    code: string,
    shaderPipelineLayout: GPUPipelineLayout
  ): {
    pipeline: GPUComputePipeline;
    shader: GPUShaderModule;
  } {
    const shader = props.gpuDevice.createShaderModule({
      code,
    });
    shader.getCompilationInfo().then((info) => {
      console.log("Shader compilation resulted in ", info);
    });
    const pipeline = props.gpuDevice.createComputePipeline({
      layout: shaderPipelineLayout,
      compute: {
        module: shader,
      },
    });
    return { shader, pipeline };
  }

  async function getShaderCode(
    fs: ReactiveFilesystem,
    key: FilePath
  ): Promise<{
    lodStage: string;
    verticesStage: string;
  }> {
    // Race condition free, because the fs resolves read requests in order
    const code = await fs.readTextFile(key);

    const result = {
      lodStage: LodStageWgsl,
      verticesStage: OutputVerticesWgsl,
    };
    if (code !== null) {
      const replaceWithCode = (v: string) =>
        v.replace(
          /\/\/\/\/ START sampleObject([^]+?)\/\/\/\/ END sampleObject/,
          code
        );
      result.lodStage = replaceWithCode(result.lodStage);
      result.verticesStage = replaceWithCode(result.verticesStage);
      ///console.log("LOD STAGE: " + result.lodStage);
      //console.log("VERTICES STAGE: " + result.verticesStage);
    }
    return result;
  }

  let prepVerticesStageShaderAndPipeline = makeShaderFromCodeAndPipeline(
    PrepVerticesStageWgsl,
    prepVerticesStageLayout
  );

  let oneVertexEntry = 32;
  let startVertexOffset = 8;
  let padding = 8;
  const vertOutputBufferSize =
    startVertexOffset + padding + oneVertexEntry * 200_000;
  let vertOutputBuffer = device.createBuffer({
    label: "VertOutputBuffer",
    size: vertOutputBufferSize,
    usage:
      GPUBufferUsage.COPY_SRC |
      GPUBufferUsage.COPY_DST |
      GPUBufferUsage.STORAGE,
  });
  let vertReadableBuffer = device.createBuffer({
    label: "VertReadableBuffer",
    size: vertOutputBufferSize,
    usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ,
  });

  props.fs.watchFromStart((change) => {
    if (!change.key.endsWith(".wgsl")) return;
    if (change.type === "insert" || change.type === "update") {
      getShaderCode(props.fs, change.key).then(
        ({ lodStage, verticesStage }) => {
          console.log("LSTAGE 2: " + lodStage);
          console.log("VSTAGE 2: " + lodStage);
          compiledShaders.value.set(change.key, {
            lodStage: makeShaderFromCodeAndPipeline(lodStage, pipelineLayout),
            verticesStage: makeShaderFromCodeAndPipeline(
              verticesStage,
              outputVerticesLayout
            ),
          });
        }
      );
    } else {
      compiledShaders.value.delete(change.key);
    }
  });

  const patchesBufferDebug = createBufferWith(
    // Fill however many slots you want to debug print
    concatArrayBuffers([new Uint32Array([0, 0, 0, 0, 0, 0, 0, 0])]),
    GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ
  );
  let hasDebugInfo = false;
  const debugPrintBuffer = (
    buffer: GPUBuffer,
    commandEncoder: GPUCommandEncoder
  ) => {
    if (!hasDebugInfo) {
      hasDebugInfo = true;
      const size = Math.min(patchesBufferDebug.size, buffer.size);
      commandEncoder.copyBufferToBuffer(buffer, 0, patchesBufferDebug, 0, size);
      // Need to wait until *after* submit has happened to be able to call mapAsync
      setTimeout(() => {
        patchesBufferDebug.mapAsync(GPUMapMode.READ, 0, size).then(() => {
          const arrayBuffer = patchesBufferDebug
            .getMappedRange(0, size)
            .slice(0);
          const uint32Array = new Uint32Array(arrayBuffer);
          console.log("patchesBufferDebug", uint32Array);
          patchesBufferDebug.unmap();
          hasDebugInfo = false;
        });
      }, 0);
    }
  };

  const MAX_PATCHES = 100_000;
  const patchesBufferReset = createBufferWith(
    concatArrayBuffers([new Uint32Array([0, MAX_PATCHES])]),
    GPUBufferUsage.COPY_SRC
  );

  const vertOutputBufferReset = createBufferWith(
    concatArrayBuffers([new Uint32Array([0, vertOutputBufferSize])]),
    GPUBufferUsage.COPY_SRC
  );

  const indirectComputeBufferReset = createBufferWith(
    concatArrayBuffers([new Uint32Array([0, 1, 1])]),
    GPUBufferUsage.COPY_SRC
  );

  const forceRenderFalse = createBufferWith(
    concatArrayBuffers([new Uint32Array([0])]),
    GPUBufferUsage.COPY_SRC
  );

  const forceRenderTrue = createBufferWith(
    concatArrayBuffers([new Uint32Array([1])]),
    GPUBufferUsage.COPY_SRC
  );

  const dispatchVerticesStage = createBufferWith(
    concatArrayBuffers([new Uint32Array([1, 1, 1])]),
    GPUBufferUsage.COPY_SRC | GPUBufferUsage.STORAGE | GPUBufferUsage.INDIRECT
  );

  function lodStageCallback(
    shaderPath: string,
    buffers: LodStageBuffers,
    commandEncoder: GPUCommandEncoder
  ) {
    //console.log(compiledShaders);
    debugger;
    const compiledShader = compiledShaders.value.get(makeFilePath(shaderPath));
    if (!compiledShader) {
      console.error("Shader not found: ", shaderPath);
      return;
    }
    const pipeline = compiledShader.lodStage.pipeline;

    // We inefficiently recreate the bind groups every time.
    const userInputBindGroup = device.createBindGroup({
      layout: sceneUniformsLayout,
      entries: [
        {
          binding: 0,
          resource: { buffer: buffers.time },
        },
        {
          binding: 1,
          resource: { buffer: buffers.screen },
        },
        {
          binding: 2,
          resource: { buffer: buffers.mouse },
        },
      ],
    });

    const toRenderPatchesBindGroup = device.createBindGroup({
      layout: layout1,
      entries: [
        {
          binding: 0,
          resource: { buffer: buffers.computePatchesInput },
        },
        {
          binding: 1,
          resource: { buffer: buffers.finalPatches2 },
        },
        {
          binding: 2,
          resource: { buffer: buffers.finalPatches4 },
        },
        {
          binding: 3,
          resource: { buffer: buffers.finalPatches8 },
        },
        {
          binding: 4,
          resource: { buffer: buffers.finalPatches16 },
        },
        {
          binding: 5,
          resource: { buffer: buffers.finalPatches32 },
        },
      ],
    });

    const pingPongPatchesBindGroup = [
      device.createBindGroup({
        layout: layout2,
        entries: [
          {
            binding: 0,
            resource: { buffer: buffers.indirectDispatch1 },
          },
          {
            binding: 1,
            resource: { buffer: buffers.patches0 },
          },
          {
            binding: 2,
            resource: { buffer: buffers.patches1 },
          },
          {
            binding: 3,
            resource: { buffer: buffers.forceRender },
          },
        ],
      }),
      device.createBindGroup({
        layout: layout2,
        entries: [
          {
            binding: 0,
            resource: { buffer: buffers.indirectDispatch0 },
          },
          {
            binding: 1,
            resource: { buffer: buffers.patches1 },
          },
          {
            binding: 2,
            resource: { buffer: buffers.patches0 },
          },
          {
            binding: 3,
            resource: { buffer: buffers.forceRender },
          },
        ],
      }),
    ];

    let bindGroupVertPrep = device.createBindGroup({
      layout: prepVerticesBinding0,
      entries: [
        {
          binding: 0,
          resource: { buffer: buffers.finalPatches2 },
        },
        {
          binding: 1,
          resource: { buffer: dispatchVerticesStage },
        },
      ],
    });

    // @group(1) @binding(0) var<uniform> model: Model;
    // @group(1) @binding(1) var<storage, read> render_buffer: RenderBufferRead;
    // @group(1) @binding(2) var<storage, read_write> output_buffer: OutputBuffer;

    let outputVerticesBindGroup = device.createBindGroup({
      layout: vertexOutputLayout,
      entries: [
        //{
        //  binding: 0,
        //  resource: { buffer:  },
        //},
        {
          binding: 1,
          resource: { buffer: buffers.finalPatches2 },
        },
        {
          binding: 2,
          resource: { buffer: vertOutputBuffer },
        },
      ],
    });

    const doubleNumberOfRounds = 3;
    // loop entire process, duplicate entire commandEncoder procedure "doubleNumberOfRounds" times to get more subdivision levels
    for (let i = 0; i < doubleNumberOfRounds; i++) {
      const isLastRound = i === doubleNumberOfRounds - 1;
      // Ping
      commandEncoder.copyBufferToBuffer(
        patchesBufferReset,
        0,
        buffers.patches1,
        0,
        patchesBufferReset.size
      );
      commandEncoder.copyBufferToBuffer(
        indirectComputeBufferReset,
        0,
        buffers.indirectDispatch1,
        0,
        indirectComputeBufferReset.size
      );

      let computePassPing = commandEncoder.beginComputePass();
      computePassPing.setPipeline(pipeline);
      computePassPing.setBindGroup(0, userInputBindGroup);
      computePassPing.setBindGroup(1, toRenderPatchesBindGroup);
      computePassPing.setBindGroup(2, pingPongPatchesBindGroup[0]);
      computePassPing.dispatchWorkgroupsIndirect(buffers.indirectDispatch0, 0);
      computePassPing.end();

      // debugPrintBuffer(buffers.patches1, commandEncoder);

      // Pong
      if (isLastRound) {
        commandEncoder.copyBufferToBuffer(
          forceRenderTrue,
          0,
          buffers.forceRender,
          0,
          forceRenderTrue.size
        );
      }

      commandEncoder.copyBufferToBuffer(
        patchesBufferReset,
        0,
        buffers.patches0,
        0,
        patchesBufferReset.size
      );
      commandEncoder.copyBufferToBuffer(
        indirectComputeBufferReset,
        0,
        buffers.indirectDispatch0,
        0,
        indirectComputeBufferReset.size
      );

      let computePassPong = commandEncoder.beginComputePass({ label: "Pong" });
      computePassPong.setPipeline(pipeline);
      computePassPong.setBindGroup(0, userInputBindGroup);
      computePassPong.setBindGroup(1, toRenderPatchesBindGroup);
      computePassPong.setBindGroup(2, pingPongPatchesBindGroup[1]);
      computePassPong.dispatchWorkgroupsIndirect(buffers.indirectDispatch1, 0);
      computePassPong.end();

      if (isLastRound) {
        commandEncoder.copyBufferToBuffer(
          forceRenderFalse,
          0,
          buffers.forceRender,
          0,
          forceRenderFalse.size
        );
      }
    }

    // Prepare vertices output
    let computePassPrep = commandEncoder.beginComputePass({
      label: "prepareVertexOutput",
    });
    computePassPrep.setPipeline(prepVerticesStageShaderAndPipeline.pipeline);
    computePassPrep.setBindGroup(0, bindGroupVertPrep);
    computePassPrep.dispatchWorkgroups(1);
    computePassPrep.end();

    commandEncoder.copyBufferToBuffer(
      vertOutputBufferReset,
      0,
      vertOutputBuffer,
      0,
      vertOutputBufferReset.size
    );

    // Run vertex output shader
    let computePassVertices = commandEncoder.beginComputePass({
      label: "vertexOutputStage",
    });
    computePassVertices.setPipeline(compiledShader.verticesStage.pipeline);
    computePassVertices.setBindGroup(0, userInputBindGroup);
    computePassVertices.setBindGroup(1, outputVerticesBindGroup);
    computePassVertices.dispatchWorkgroupsIndirect(dispatchVerticesStage, 0);
    computePassVertices.end();

    if (triggerDownload.value) {
      triggerDownload.value = false;
      commandEncoder.copyBufferToBuffer(
        vertOutputBuffer,
        0,
        vertReadableBuffer,
        0,
        vertOutputBuffer.size
      );
      setTimeout(() => {
        vertReadableBuffer
          .mapAsync(GPUMapMode.READ, 0, vertReadableBuffer.size)
          .then(() => {
            const arrayBuffer = vertReadableBuffer
              .getMappedRange(0, vertReadableBuffer.size)
              .slice(0);
            const vertexStream = new Float32Array(arrayBuffer);
            //console.log("vertReadableBuffer Output:", vertexStream); // Only zeroes. Huh
            vertReadableBuffer.unmap();
            exportMeshFromPatches(vertexStream);
          });
      }, 10);
    }
  }

  props.engine.setLodStage(lodStageCallback);
}

// Define a type for a single vertex
interface Vertex {
    vert: { x: number; y: number; z: number };
    uv: { x: number; y: number };
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
    const index = rangeList.findIndex(r => r.start >= range.start);
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
function interpolate(range: VertexRange, value: number, patches: Vertex[][]): THREE.Vector3 {
    const ip = getInterpolationValue(range, value);

    const startVec = new THREE.Vector3(
        patches[range.ipi][range.startVert].vert.x,
        patches[range.ipi][range.startVert].vert.y,
        patches[range.ipi][range.startVert].vert.z
    );

    const endVec = new THREE.Vector3(
        patches[range.ipi][range.endVert].vert.x,
        patches[range.ipi][range.endVert].vert.y,
        patches[range.ipi][range.endVert].vert.z
    );

    return startVec.multiplyScalar(1 - ip).add(endVec.multiplyScalar(ip));
}


type VRL =  Record<number, VertexRange[]>;

// Main function
function analyzeEdges(patches: Vertex[][]): any {
    const rangesHorizontal: { top: Record<number, VertexRange[]>; bottom: Record<number, VertexRange[]> } = {
        top: {},
        bottom: {},
    };

    const rangesVertical: { left: Record<number, VertexRange[]>; right: Record<number, VertexRange[]> } = {
        left: {},
        right: {},
    };

    const ranges = {horizontal: rangesHorizontal, vertical: rangesVertical};
    const topRanges = rangesHorizontal["top"];
    const botRanges = rangesHorizontal["bottom"];
    const leftRanges = rangesVertical["left"];
    const rightRanges = rangesVertical["right"];

    let inpInd = 0;

    for (const patch of patches) {
        const v1 = patch[0], v2 = patch[1], v3 = patch[2], v4 = patch[3];
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

// Main function
function fixPatchSeams(patches: Vertex[][]): Vertex[][] {
    const rangesHorizontal: { top: Record<number, VertexRange[]>; bottom: Record<number, VertexRange[]> } = {
        top: {},
        bottom: {},
    };

    const rangesVertical: { left: Record<number, VertexRange[]>; right: Record<number, VertexRange[]> } = {
        left: {},
        right: {},
    };

    const topRanges = rangesHorizontal["top"];
    const botRanges = rangesHorizontal["bottom"];
    const leftRanges = rangesVertical["left"];
    const rightRanges = rangesVertical["right"];

    let inpInd = 0;

    for (const patch of patches) {
        const v1 = patch[0], v2 = patch[1], v3 = patch[2], v4 = patch[3];
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

    // Horizontal interpolation (top-to-bottom and bottom-to-top)
    for (const edge of Object.keys(topRanges).map(Number)) {
        if (botRanges[edge]) {
            const topRange = topRanges[edge];
            const botRange = botRanges[edge];
            interpolateRanges(topRange, botRange, patches);
        }
    }

    // Vertical interpolation (left-to-right and right-to-left)
    for (const edge of Object.keys(leftRanges).map(Number)) {
        if (rightRanges[edge]) {
            const leftRange = leftRanges[edge];
            const rightRange = rightRanges[edge];
            interpolateRanges(leftRange, rightRange, patches);
        }
    }

    console.log("PATCHED: ");
    console.log(patches);

    return patches;
}

function interpolateRanges(
    range1: VertexRange[],
    range2: VertexRange[],
    patches: Vertex[][]
) {
    for (const uvr of range1) {
        for (const uvrOther of range2) {
            if (checkContains(uvrOther, uvr)) {
                if (uvr.start !== uvrOther.start) {
                    const interpolatedStart = interpolate(uvrOther, uvr.start, patches);
                    //console.log("Interpolated start! Previously " + JSON.stringify(patches[uvr.ipi][uvr.startVert].vert) + " Now " + JSON.stringify(interpolatedStart));
                    patches[uvr.ipi][uvr.startVert].vert = {
                        x: interpolatedStart.x,
                        y: interpolatedStart.y,
                        z: interpolatedStart.z,
                    };
                }
                if (uvr.end !== uvrOther.end) {
                    const interpolatedEnd = interpolate(uvrOther, uvr.end, patches);
                    //console.log("Interpolated end");
                    patches[uvr.ipi][uvr.endVert].vert = {
                        x: interpolatedEnd.x,
                        y: interpolatedEnd.y,
                        z: interpolatedEnd.z,
                    };
                }
            }
        }
    }
}


main();

</script>
<template>
  <div></div>
  <div class="absolute bg-red-100">
    <button @click="triggerDownload = true">Download</button>
  </div>
</template>