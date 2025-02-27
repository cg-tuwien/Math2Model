<script setup lang="ts">
import { useStore } from "@/stores/store";
import type { DropdownOption } from "naive-ui/es/dropdown/src/interface";
import { computed, h, ref } from "vue";
import IconMoon from "~icons/mdi/moon-and-stars";
import IconSun from "~icons/mdi/white-balance-sunny";
import IconGithub from "~icons/mdi/github";
import { assertUnreachable } from "@stefnotch/typestef/assert";
import { homepage, version } from "@/../package.json";
import { useFsStore, type ImportFilesList } from "@/stores/fs-store";
import { useExportStore } from "@/stores/export-store";
import { makeFilePath } from "@/filesystem/reactive-files";
import { ZipReader } from "@zip.js/zip.js";

const store = useStore();
const fsStore = useFsStore();
const exportStore = useExportStore();

const examplesModule = () => import("@/scenes/example-scenes");

const heartSphereMorphExamplePath = "./HeartSphereMorph.zip";
const templeExamplePath = "./TempleExample.zip";
const towerExamplePath = "./TowerExample.zip";
const treesAndTerrainExamplePath = "./TreesAndTerrainExample.zip";

type FileDropdownOption = DropdownOption & {

  key:
    | "open"
    | "save-as"
    | "examples"
    | "example-scene"
    | "heart-sphere-scene"
    | "temple-scene"
    | "tower-scene"
    | "terrain-trees-scene"
    | "export";
};

const inputFileElement = ref<HTMLInputElement | null>(null);
const fileOptions = computed((): FileDropdownOption[] => {
  return [
    {
      label: "Open",
      key: "open",
    },
    {
      label: "Save As",
      key: "save-as",
    },
    {
      label: "Toggle Export GUI",
      key: "export"
    },
    {
      label: "Examples",
      key: "examples",
      children: [
        {
          label: "Example Scene",
          key: "example-scene",
        },
        {
          label: "Heart Sphere Scene",
          key: "heart-sphere-scene",
        },
        {
          label: "Temple Scene",
          key: "temple-scene",
        },
        {
          label: "Tower Scene",
          key: "tower-scene",
        },
        {
          label: "Terrain and Trees",
          key: "terrain-trees-scene",
        },
      ] as FileDropdownOption[],
    },
  ];
});
async function handleFile(key: FileDropdownOption["key"]) {
  if (key === "open") {
    inputFileElement.value?.click();
  } else if (key === "save-as") {
    await fsStore.exportToZip();
  } else if (key === "examples") {
    // Do nothing
  } else if (key === "example-scene") {
    const module = await examplesModule();
    fsStore.importInMemoryProject(module.createDefaultProject().files);
  } else if (key === "heart-sphere-scene") {
    const module = await examplesModule();
    const heartSphereScene = await module.getZipExample(
      makeFilePath(heartSphereMorphExamplePath)
    );
    if (heartSphereScene) {
      const data: ImportFilesList = {
        type: "zip",
        value: new ZipReader(heartSphereScene.stream()),
      };
      await fsStore.clearFiles();
      fsStore.importFiles(data);
    }
  } else if (key === "temple-scene") {
    const module = await examplesModule();
    const templeScene = await module.getZipExample(
      makeFilePath(templeExamplePath)
    );
    if (templeScene) {
      const data: ImportFilesList = {
        type: "zip",
        value: new ZipReader(templeScene.stream()),
      };
      await fsStore.clearFiles();
      fsStore.importFiles(data);
    }
  } else if (key === "terrain-trees-scene") {
    const module = await examplesModule();
    const treesAndTerrainScene = await module.getZipExample(
      makeFilePath(treesAndTerrainExamplePath)
    );
    if (treesAndTerrainScene) {
      const data: ImportFilesList = {
        type: "zip",
        value: new ZipReader(treesAndTerrainScene.stream()),
      };
      await fsStore.clearFiles();
      fsStore.importFiles(data);
    }
  } else if (key === "tower-scene") {
    const module = await examplesModule();
    const towerScene = await module.getZipExample(
      makeFilePath(towerExamplePath)
    );
    if (towerScene) {
      const data: ImportFilesList = {
        type: "zip",
        value: new ZipReader(towerScene.stream()),
      };
      await fsStore.clearFiles();
      fsStore.importFiles(data);
    }
    fsStore.importInMemoryProject(module.createHeartSphereProject().files);
  } else if(key === "export") {
    exportStore.isExportMode = !exportStore.isExportMode;
  } else {
    assertUnreachable(key);
  }
}

type ViewDropdownOption = DropdownOption & {
  key: "switch-to-dark" | "switch-to-light";
};
const viewOptions = computed((): ViewDropdownOption[] => {
  return [
    store.isDark
      ? {
          label: "Light",
          key: "switch-to-light",
          icon: () => h(IconSun),
        }
      : {
          label: "Dark",
          key: "switch-to-dark",
          icon: () => h(IconMoon),
        },
  ];
});
function handleView(key: ViewDropdownOption["key"]) {
  if (key === "switch-to-dark") {
    store.setIsDark(true);
  } else if (key === "switch-to-light") {
    store.setIsDark(false);
  } else {
    assertUnreachable(key);
  }
}

type HelpDropdownOption = DropdownOption & {
  key: "go-to-github" | "version";
};
const helpOptions = computed((): HelpDropdownOption[] => {
  return [
    {
      label: `Version ${version}`,
      key: "version",
    },
    {
      label: "GitHub",
      key: "go-to-github",
      icon: () => h(IconGithub),
    },
  ];
});

function handleHelp(key: HelpDropdownOption["key"]) {
  if (key === "go-to-github") {
    window.open(homepage, "_blank")?.focus();
  } else if (key === "version") {
    // Do nothing
  } else {
    assertUnreachable(key);
  }
}

async function openFiles(inputFiles: FileList) {
  fsStore.importFilesOrProject(inputFiles);
}
</script>

<template>
  <n-page-header class="px-2 border-b border-gray-200">
    <template #title>
      <div class="flex">
        <span class="relative inline-block w-8">
          <img
            src="@/assets/logo.svg"
            alt="logo"
            class="absolute h-5 mt-1 ml-0.5 inline"
          />
        </span>
        <span class="ml-1">
          <input
            type="file"
            multiple
            class="hidden"
            ref="inputFileElement"
            @change="
              (ev) =>
                openFiles(
                  inputFileElement?.files ?? (ev.target as any).files ?? []
                )
            "
          />
          <n-dropdown
            trigger="click"
            :options="fileOptions"
            @select="handleFile"
          >
            <n-button :bordered="false" size="small" quaternary>
              File
            </n-button>
          </n-dropdown>
          <n-dropdown
            trigger="click"
            :options="viewOptions"
            @select="handleView"
          >
            <n-button :bordered="false" size="small" quaternary>
              View
            </n-button>
          </n-dropdown>
          <n-dropdown
            trigger="click"
            :options="helpOptions"
            @select="handleHelp"
          >
            <n-button :bordered="false" size="small" quaternary>
              Help
            </n-button>
          </n-dropdown>
        </span>
      </div>
    </template>
  </n-page-header>
</template>

<style scoped></style>
