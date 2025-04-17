<script setup lang="ts">
import { type ReactiveFilesystem } from "@/filesystem/reactive-files";
import { ref, type Ref } from "vue";
import { watchImmediate } from "@vueuse/core";
import { ExporterInstance } from "./exporter/Exporter";
import { Vector2, Vector3 } from "./exporter/VectorTypes";
import { mainExport } from "./exporter/GPUInteractionExport";
import type { WgpuEngine } from "@/engine/wgpu-engine";
import { analyzeEdges } from "./exporter/EdgeAnalysis";
import { saveFile, saveFileBinary } from "./exporter/FileDownload";
import { MeshExporter3DFormats } from "./exporter/MeshExporter3DFormats";
import { NButton } from "naive-ui";
import {
  SceneFileName,
  deserializeScene,
  type SerializedScene,
} from "@/filesystem/scene-file";
import { assert } from "@stefnotch/typestef/assert";
import { useExportStore } from "@/stores/export-store";

const exportStore = useExportStore();
const props = defineProps<{
  gpuDevice: GPUDevice;
  engine: WgpuEngine;
  fs: ReactiveFilesystem;
}>();

const triggerDownload = ref(false);
const minSize = ref(0.1);
const maxCurvature = ref(1);
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
const toDownload: Ref<{ name: string; currentInstance: number }[]> = ref([]);
const exportProgress: Ref<number> = ref(0);
const currentDownloadItem: Ref<string> = ref("Nothing");
const ignoreMinSize: Ref<boolean> = ref(true);
const ignoreCurvature: Ref<boolean> = ref(true);
const ignorePlanarity: Ref<boolean> = ref(true);

let exportSteps = 0;
let exportStepsDone = 0;

let meshBuffer: any[] = [];
let canceled = false;
function onFrame() {}

async function exportMeshBuffer(name: string) {
  let exportProgressPerStep = 1 / meshBuffer.length;
  let i = 0;
  if (!canceled) {
    if (mergeModels.value) {
      // console.log("Mesh buffer:",meshBuffer);
      await exportMeshes(meshBuffer, name);
      meshBuffer.length = 0;
    } else {
      for (let mesh of meshBuffer) {
        exportProgress.value = i * exportProgressPerStep;
        await exportMeshes([mesh], mesh.name, exportProgressPerStep);
        i++;
      }
    }
  }

  toDownload.value = [];
  meshBuffer.length = 0;
  exportInProgress.value = false;
  triggerDownload.value = false;
  exportProgress.value = 0;
  exportStepsDone = 0;
  canceled = false;
}

async function exportMeshes(
  meshes: any[],
  name: string,
  progressToMake?: number
) {
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

async function loadSceneFile(): Promise<string> {
  let f: File | null = await props.fs.readFile(SceneFileName);
  let text: string | undefined = await f?.text();
  assert(text !== undefined, "Missing scene file");
  return text as string;
}
async function bufferMeshesForExport(
  instanceMapPatches: Map<number, any>,
  includeUVs: boolean,
  uuid: string,
  buffer: boolean
) {
  console.log("Buffer meshes for export call started");
  let progressPerStep = 1 / exportSteps;
  let name = uuid;
  //let looped = false;
  //if (toDownload.value.length == 0) {
  //  buffer = false;
  //  looped = true;
  //}

  let scenePromise = fetchScene();

  exportProgress.value = progressPerStep * (exportStepsDone + 0.5);
  let scene = await scenePromise;
  for (const [instance_id, patches] of instanceMapPatches) {
    // Bake edge information
    // console.log("Patch count",patches.length, " on mesh " + uuid);
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

    // console.log("Exporter instance gave out ",exporterInstance.tris.length);
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
    // console.log(
    //   "Mesh has " +
    //     exporterInstance.vertPositions.length +
    //     " vertices on instance#" +
    //     instance_id
    // );
  }
  exportStepsDone++;
  // console.log("To download: ", toDownload);

  console.log("Buffer meshes for export call checking if export done");
  if (buffer) return;
  console.log("Downloading. To download: ", JSON.stringify(toDownload.value));
  exportMeshBuffer(name);
}

async function fetchScene(): Promise<SerializedScene> {
  let sceneFile = loadSceneFile();
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
  let progressForStep = (index / slicedStream.length) * progressPerStep;
  progressForStep /= 2;
  exportProgress.value = progressPerStep * (exportStepsDone + progressForStep);

  while (index < slicedStream.length) {
    let vertex = slicedStream.slice(index, index + 3);
    let uv = slicedStream.slice(index + 4, index + 6);
    if (vertex[0] == 0 && vertex[1] == 0 && vertex[2] == 0) {
      let endofdata = true;
      for (let x = 0; x < 100; x++) {
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

    if (!patches.has(0)) {
      patches.set(0, []);
    }
    patches.get(0)?.push(patch_verts);

    patch_verts = [];
  }

  // console.log("Got request to export mesh " + name);
  // console.log(toDownload.value);
  for (const [instance_id, patchstructure] of patches) {
    for (let i = 0; i < patchstructure.length; i++) {
      for (let j = 0; j < 4; j++) {
        let p = patchstructure[i][j];
        p.globalIndex = -1;
      }
    }
  }
  bufferMeshesForExport(patches, includeUVs, name, buffer);
}

let scene = fetchScene();
scene.then((actualScene) => {
  actualScene.models.forEach((model) => {
    models.value.push({
      path: model.parametricShader,
      name: model.name,
      uuid: model.id,
      instanceCount: model.instanceCount,
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
          instanceCount: 0,
        },
      ];
      let actualScene = deserializeScene(v);
      actualScene.models.forEach((model) =>
        models.value.push({
          path: model.parametricShader,
          name: model.name,
          uuid: model.id,
          instanceCount: model.instanceCount,
        })
      );
    });
  }
});

// Main function
let lodStageCallback = await mainExport(
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
    subdivisionSteps: subdivisionSteps,
    toDownload: toDownload,
    currentItem: currentDownloadItem,
    ignoreMinSize: ignoreMinSize,
    ignoreCurvature: ignoreCurvature,
    ignorePlanarity: ignorePlanarity,
  },
  onFrame
);
async function beginExportProcess() {
  canceled = false;
  triggerDownload.value = true;
  exportInProgress.value = true;
  toDownload.value.length = 0;
  let sceneFile = loadSceneFile();
  let toDownloadSteps = 0;
  let scene = deserializeScene(await sceneFile);
  props.engine.setLodStage(lodStageCallback);
  scene.models.forEach((model) => {
    if (downloadTarget.value == "") {
      toDownload.value.push({ name: model.id, currentInstance: 0 });
      toDownloadSteps += model.instanceCount;
    }
    if (model.id == downloadTarget.value) {
      toDownload.value.push({ name: model.id, currentInstance: 0 });
      toDownloadSteps += model.instanceCount;
      return;
    }
  });
  exportSteps = toDownloadSteps; //toDownload.value.length;
  // console.log("To download: ", toDownload);
}

function abortExport() {
  exportStore.isExportMode = false;
  meshBuffer = [];
  exportInProgress.value = false;
  exportProgress.value = 0;
  toDownload.value = [];
  canceled = true;
  updateExportPreview();
}

function updateExportPreview() {
  if (exportStore.showExportPreview) props.engine.setLodStage(lodStageCallback);
  else props.engine.setLodStage(null);
}
// Update the preview whenever it changes
watchImmediate(
  () => exportStore.showExportPreview,
  () => {
    updateExportPreview();
  }
);
</script>

<template>
  <n-card title="Exporter" size="small">
    <div class="h-full overflow-y-auto flex flex-col gap-6">
      <div class="flex gap-4">
        <LabeledFormItem
          tooltip="Interactively preview the export. Warning: performance heavy"
          ><template #label>Preview Export</template>
        </LabeledFormItem>
        <n-switch v-model:value="exportStore.showExportPreview">
          <template #checked> On </template>
          <template #unchecked> Off </template>
        </n-switch>
      </div>

      <n-collapse arrow-placement="right" :default-expanded-names="['1', '2']">
        <n-collapse-item title="LOD Configuration" name="1">
          <div class="flex flex-col gap-6">
            <LabeledFormItem>
              <template #label>Models to export</template>
              <n-select
                v-model:value="downloadTarget"
                size="small"
                :disabled="exportInProgress"
                :options="
                  models.map((model) => ({
                    label: model.name,
                    value: model.uuid,
                  }))
                "
              />
            </LabeledFormItem>

            <LabeledFormItem
              tooltip="Controls the minimum size of the exported quads"
            >
              <template #label>
                <n-checkbox
                  v-model:checked="ignoreMinSize"
                  style="margin-right: -8px"
                >
                  Minimum Patch Size
                </n-checkbox>
              </template>
              <div v-if="ignoreMinSize" class="flex gap-4">
                <n-slider
                  class="mt-2"
                  v-model:value="minSize"
                  :min="0"
                  :max="30"
                  :step="0.0001"
                  :disabled="exportInProgress"
                />
                <n-input-number v-model:value="minSize" class="w-38" />
              </div>
            </LabeledFormItem>

            <LabeledFormItem tooltip="TODO: fill this out">
              <template #label>
                <n-checkbox
                  v-model:checked="ignoreCurvature"
                  style="margin-right: -8px"
                >
                  Curvature Criterium
                </n-checkbox>
              </template>
              <div v-if="ignoreCurvature" class="flex gap-4">
                <n-slider
                  class="mt-2"
                  v-model:value="maxCurvature"
                  :min="0"
                  :max="1"
                  :step="0.01"
                  :disabled="exportInProgress"
                />
                <n-input-number v-model:value="maxCurvature" class="w-38" />
              </div>
            </LabeledFormItem>

            <LabeledFormItem tooltip="TODO: fill this out">
              <template #label>
                <n-checkbox
                  v-model:checked="ignorePlanarity"
                  style="margin-right: -8px"
                >
                  Planarity Criterium
                </n-checkbox>
              </template>
              <div v-if="ignorePlanarity" class="flex gap-4">
                <n-slider
                  class="mt-2"
                  v-model:value="acceptablePlanarity"
                  :min="0.0"
                  :max="1"
                  :step="0.000001"
                  :disabled="exportInProgress"
                />
                <n-input-number
                  v-model:value="acceptablePlanarity"
                  class="w-38"
                />
              </div>
            </LabeledFormItem>

            <LabeledFormItem tooltip="How fine grained the details will be">
              <template #label> Subdivision steps </template>
              <div class="flex gap-4">
                <n-slider
                  class="mt-2"
                  v-model:value="subdivisionSteps"
                  :min="1"
                  :max="5"
                  :step="1"
                  :disabled="exportInProgress"
                />
                <n-input-number v-model:value="subdivisionSteps" class="w-38" />
              </div>
            </LabeledFormItem>
          </div>
        </n-collapse-item>

        <n-collapse-item title="File Configuration" name="2">
          <n-space vertical :size="0">
            <n-form-item
              label="File Format"
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
              label="Include UVs"
              label-placement="left"
              class="compact-form-item"
            >
              <n-checkbox
                v-model:checked="includeUVs"
                :disabled="exportInProgress"
              />
            </n-form-item>

            <n-form-item
              label="Normal Direction"
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
          </n-space>
        </n-collapse-item>
      </n-collapse>
      <n-space vertical :size="8">
        <n-button
          @click="beginExportProcess"
          :disabled="exportInProgress"
          type="primary"
          block
        >
          Download
        </n-button>

        <n-progress
          v-if="exportInProgress"
          :percentage="Math.floor(exportProgress * 100)"
        />
        <n-text v-if="exportInProgress">{{ currentDownloadItem }}</n-text>
        <n-button @click="abortExport()" type="error" block> Cancel </n-button>
      </n-space>
    </div>
  </n-card>
</template>

<style scoped>
.compact-form-item {
  margin-bottom: 4px;
}
</style>
