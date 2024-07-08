<script setup lang="ts">
import { useStore } from "@/stores/store";
import type { DropdownOption } from "naive-ui/es/dropdown/src/interface";
import { computed, h } from "vue";
import { useRouter } from "vue-router";
import IconMoon from "~icons/mdi/moon-and-stars";
import IconSun from "~icons/mdi/white-balance-sunny";
import IconGithub from "~icons/mdi/github";
import { assertUnreachable } from "@stefnotch/typestef/assert";
import { homepage } from "@/../package.json";
import type { ReactiveFiles } from "@/filesystem/reactive-files";
import { BlobWriter, ZipWriter } from "@zip.js/zip.js";

const store = useStore();
const router = useRouter();

const props = defineProps<{
  files: ReactiveFiles | null;
}>();

type FileDropdownOption = DropdownOption & {
  key: "open" | "save-as" | "examples";
};

const fileOptions = computed((): FileDropdownOption[] => {
  return [
    {
      label: "Open",
      key: "open",
      disabled: props.files === null,
    },
    {
      label: "Save As",
      key: "save-as",
      disabled: props.files === null,
    },
    {
      label: "Examples",
      key: "examples",
      disabled: props.files === null,
    },
  ];
});
async function handleFile(key: FileDropdownOption["key"]) {
  if (key === "open") {
    // TODO: Open any file (or folder?)
    // Then
    // - If it is a file list: Import all files
    // - If it is a folder: Import all files
    // - If it looks like a zip: Import all files
    // TODO: If the file list has a `scene.json`, then ask the user
    // - Add to current project
    // - Or open as new project
    // TODO: Drag and drop support (onto the file list)
  } else if (key === "save-as") {
    // TODO: Save project as zip
    if (props.files === null) return;
    const files = props.files.listFiles().flatMap((filePath) => {
      const file = props.files?.readFile(filePath) ?? null;
      return file === null ? [] : [{ filePath, file }];
    });
    const zip = new ZipWriter(new BlobWriter("application/zip"), {
      bufferedWrite: true,
    });
    await Promise.all(
      files.map(({ filePath, file }) =>
        zip.add(filePath, new Blob([file]).stream())
      )
    );
    const blobUrl = URL.createObjectURL(await zip.close());
    const a = document.createElement("a");
    a.href = blobUrl;
    a.download = "project.zip";
    a.click();
    URL.revokeObjectURL(blobUrl);
  } else if (key === "examples") {
    // Do nothing
  } else {
    assertUnreachable(key);
  }
}

type ViewDropdownOption = DropdownOption & {
  key:
    | "switch-to-dark"
    | "switch-to-light"
    | "switch-to-babylon"
    | "switch-to-webgpu";
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
    {
      label: "Babylon.js Renderer",
      key: "switch-to-babylon",
    },
    {
      label: "WebGPU Renderer",
      key: "switch-to-webgpu",
    },
  ];
});
function handleView(key: ViewDropdownOption["key"]) {
  if (key === "switch-to-dark") {
    store.setIsDark(true);
  } else if (key === "switch-to-light") {
    store.setIsDark(false);
  } else if (key === "switch-to-babylon") {
    router.push("/");
  } else if (key === "switch-to-webgpu") {
    router.push("/webgpu");
  } else {
    assertUnreachable(key);
  }
}

type HelpDropdownOption = DropdownOption & {
  key: "go-to-github";
};
const helpOptions = computed((): HelpDropdownOption[] => {
  return [
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
  } else {
    assertUnreachable(key);
  }
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
