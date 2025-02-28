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
import { NButton, NInput, NText, type UploadFileInfo } from "naive-ui";
import type { SelectMixedOption } from "naive-ui/es/select/src/interface";
import {
  SceneFileName,
  deserializeScene,
  type SerializedScene,
} from "@/filesystem/scene-file";
import { assert } from "@stefnotch/typestef/assert";
import { useStore } from "../stores/store";
import { computed } from "vue";
import { useExportStore } from "@/stores/export-store";

const store = useStore();
//const theme = computed(() => (store.isDark ? darkTheme : lightTheme));
const exportStore = useExportStore();
const props = defineProps<{
  gpuDevice: GPUDevice;
  engine: WgpuEngine;
  fs: ReactiveFilesystem;
}>();

const triggerDownload = ref(false);
const minSize = ref(0.1);
const maxCurvature = ref(2.5);
const acceptablePlanarity = ref(0.99999);
const includeUVs = ref(false);
const fileFormat = ref("glb");
const subdivisionSteps = ref(4);
const models: Ref<any[]> = ref([
  {
    path: "",
    name: "all",
    uuid: "",
  },
]);
const normalType: Ref<number> = ref(0);
const downloadTarget = ref("");
const mergeModels = ref(true);
const exportInProgress = ref(false);
const toDownload: Ref<string[]> = ref([]);
const exportProgress: Ref<number> = ref(0);

let exportSteps = 0;
let exportStepsDone = 0;
let stepProgress = 0;

let meshBuffer: any[] = [];
let bufferedNames: Ref<string[]> = ref([]);
let lastMeshBufferUpdate = 0;

let frameCount = 0;
let canceled = false;
function onFrame() {
  frameCount++;
}

async function exportMeshBuffer(name: string) {
  let exportProgressBase = 0;
  let exportProgressPerStep = 1 / meshBuffer.length;
  let i = 0;
  if (!canceled) {
    if (mergeModels.value) {
      exportMeshList(meshBuffer, name);
    } else {
      meshBuffer.forEach((mesh) => {
        exportProgress.value = i * exportProgressPerStep;
        exportMeshList([mesh], mesh.name, exportProgressPerStep);
        i++;
      });
    }
  }

  meshBuffer = [];
  bufferedNames.value = [];
  lastMeshBufferUpdate = frameCount;
  exportInProgress.value = false;
  triggerDownload.value = false;
  exportProgress.value = 0;
  exportStepsDone = 0;
  canceled = false;
}

async function exportMeshList(
  meshes: any[],
  name: string,
  progressToMake?: number
) {
  let exportProgressPerStep = 1 / meshBuffer.length;
  let modelExporter = new MeshExporter3DFormats(meshes);
  modelExporter.useUvs = includeUVs.value;
  let format = fileFormat.value;
  modelExporter.name = name;
  if (progressToMake) {
    modelExporter.progressToMake = progressToMake;
    modelExporter.exportProgress = exportProgress;
  }
  let fileContent = await modelExporter.exportModel(format);

  if (canceled) {
    return;
  }
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
  let progressPerStep = 1 / exportSteps;
  let name = uuid;
  let looped = false;
  if (toDownload.value.length == 0) {
    buffer = false;
    looped = true;
  }

  let scenePromise = getScene();
  lastMeshBufferUpdate = frameCount;

  exportProgress.value = progressPerStep * (exportStepsDone + 0.5);
  let scene = await scenePromise;
  for (const [instance_id, patches] of instanceMapPatches) {
    // Bake edge information
    let edgeInformation = analyzeEdges(patches);
    let exporterInstance: ExporterInstance = new ExporterInstance(
      patches,
      edgeInformation
    );
    exporterInstance.useUvs = includeUVs;
    exporterInstance.normalsType = normalType.value;
    exporterInstance.exportProgressVar = exportProgress;
    exporterInstance.exportProgressStart = exportProgress.value;
    exporterInstance.exportProgressToDo = progressPerStep * 0.5;
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
    console.log("Known models: ", models);
    console.log("Scene models: ", scene.models);

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
  exportStepsDone++;
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
  let progressPerStep = 1 / exportSteps;
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
    let progressForStep = (index / slicedStream.length) * progressPerStep;
    progressForStep /= 2;
    exportProgress.value =
      progressPerStep * (exportStepsDone + progressForStep);
  }

  console.log(
    "Got request to export mesh " + name + " patch verts: " + patch_verts.length
  );
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

props.fs.watchFromStart((change) => {
  if (change.key === SceneFileName) {
    // Thread safety: Ordered reads are guaranteed by readTextFile.
    props.fs.readTextFile(change.key)?.then((v) => {
      models.value = [
        {
          path: "",
          name: "all",
          uuid: "",
        },
      ];
      let actualScene = deserializeScene(v);
      actualScene.models.forEach((model) =>
        models.value.push({
          path: model.parametricShader,
          name: model.name,
          uuid: model.id,
        })
      );
    });
  }
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
  exportSteps = toDownload.value.length;
  console.log("To download: ", toDownload);
}

function cancelExport() {
  exportStore.isExportMode = false;
  meshBuffer = [];
  exportInProgress.value = false;
  exportProgress.value = 0;
  toDownload.value = [];
  canceled = true;
}
</script>

<template :theme="theme">
  <n-card title="Export Settings" size="small">
    <n-space vertical :size="8">
      <n-form-item
        label="Min Size:"
        label-placement="left"
        class="compact-form-item"
      >
        <n-slider
          v-model:value="minSize"
          :min="0"
          :max="30"
          :step="0.001"
          :disabled="exportInProgress"
        />
        <n-input-number v-model:value="minSize" />
      </n-form-item>

      <n-form-item
        label="Max Curvature:"
        label-placement="left"
        class="compact-form-item"
      >
        <n-slider
          v-model:value="maxCurvature"
          :min="0"
          :max="3"
          :step="0.01"
          :disabled="exportInProgress"
        />
        <n-input-number v-model:value="maxCurvature" />
      </n-form-item>

      <n-form-item
        label="Planarity Criterium:"
        label-placement="left"
        class="compact-form-item"
      >
        <n-slider
          v-model:value="acceptablePlanarity"
          :min="0.97"
          :max="1"
          :step="0.00001"
          :disabled="exportInProgress"
        />
        <n-input-number v-model:value="acceptablePlanarity" />
      </n-form-item>

      <n-form-item
        label="File Format:"
        label-placement="left"
        class="compact-form-item"
      >
        <n-select
          v-model:value="fileFormat"
          :disabled="exportInProgress"
          :options="[
            { label: 'OBJ', value: 'obj' },
            { label: 'GLB', value: 'glb' },
          ]"
        />
      </n-form-item>
      <n-form-item
        label="Target Model:"
        label-placement="left"
        class="compact-form-item"
      >
        <n-select
          v-model:value="downloadTarget"
          :disabled="exportInProgress"
          :options="
            models.map((model) => ({ label: model.name, value: model.uuid }))
          "
        />
      </n-form-item>

      <n-form-item
        v-if="downloadTarget == ''"
        label="Merge Model Files"
        label-placement="left"
        class="compact-form-item"
      >
        <n-checkbox
          v-model:checked="mergeModels"
          :disabled="exportInProgress"
        />
      </n-form-item>

      <n-form-item
        label="Include UVs:"
        label-placement="left"
        class="compact-form-item"
      >
        <n-checkbox v-model:checked="includeUVs" :disabled="exportInProgress" />
      </n-form-item>

      <n-form-item
        label="Normal Direction:"
        label-placement="left"
        class="compact-form-item"
      >
        <n-select
          v-model:value="normalType"
          :disabled="exportInProgress"
          :options="[
            { label: 'Default', value: 0 },
            { label: 'Inverse', value: 1 },
            { label: 'Double Sided', value: 2 },
          ]"
        />
      </n-form-item>

      <n-form-item
        label="Division Steps:"
        label-placement="left"
        class="compact-form-item"
      >
        <n-slider
          v-model:value="subdivisionSteps"
          :min="1"
          :max="5"
          :step="1"
          :disabled="exportInProgress"
        />
      </n-form-item>

      <n-button
        @click="startExport"
        :disabled="exportInProgress"
        type="primary"
        block
      >
        Download
      </n-button>

      <n-progress v-if="exportInProgress" :percentage="exportProgress * 100" />

      <n-button @click="cancelExport()" type="error" block> Cancel </n-button>
    </n-space>
  </n-card>
</template>

<style scoped>
.compact-form-item {
  margin-bottom: 4px;
}
</style>
