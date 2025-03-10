<script setup lang="ts">
import { useStore } from "@/stores/store";
import type { DropdownOption } from "naive-ui/es/dropdown/src/interface";
import { computed, h, ref } from "vue";
import IconMoon from "~icons/mdi/moon-and-stars";
import IconSun from "~icons/mdi/white-balance-sunny";
import IconGithub from "~icons/mdi/github";
import { homepage, version } from "@/../package.json";
import { useFsStore } from "@/stores/fs-store";
import { useExportStore } from "@/stores/export-store";

// TODO: Lazy load the examples
import { ExampleProjects } from "@/scenes/example-scenes";

const store = useStore();
const fsStore = useFsStore();
const exportStore = useExportStore();

type ActionDropdownOption = DropdownOption & {
  action: () => void;
};
function isActionDropdownOption(
  option: DropdownOption
): option is ActionDropdownOption {
  return "action" in option;
}

async function handleDropdownOption(
  _key: string | number,
  option: DropdownOption
) {
  if (isActionDropdownOption(option)) {
    option.action();
  }
}

const exampleProjectsDropdown = ExampleProjects.map(
  (v): ActionDropdownOption => ({
    label: v.name,
    key: v.key,
    action: async () => {
      const data = await v.files();
      await fsStore.importProject(data);
    },
  })
);

const inputFileElement = ref<HTMLInputElement | null>(null);
const fileOptions = computed((): ActionDropdownOption[] => {
  return [
    {
      label: "Open",
      key: "open",
      action: () => {
        inputFileElement.value?.click();
      },
    },
    {
      label: "Save As",
      key: "save-as",
      action: async () => {
        await fsStore.exportToZip();
      },
    },
    {
      label: "Toggle Export GUI",
      key: "export",
      action: () => {
        exportStore.isExportMode = !exportStore.isExportMode;
      },
    },
    {
      label: "Examples",
      key: "examples",
      children: exampleProjectsDropdown,
      action: () => {},
    },
  ];
});

const viewOptions = computed((): ActionDropdownOption[] => {
  return [
    store.isDark
      ? {
          label: "Light",
          key: "switch-to-light",
          icon: () => h(IconSun),
          action: () => store.setIsDark(false),
        }
      : {
          label: "Dark",
          key: "switch-to-dark",
          icon: () => h(IconMoon),
          action: () => store.setIsDark(true),
        },
  ];
});

const helpOptions = computed((): ActionDropdownOption[] => {
  return [
    {
      label: `Version ${version}`,
      key: "version",
      action: () => {},
    },
    {
      label: "GitHub",
      key: "go-to-github",
      icon: () => h(IconGithub),
      action: () => {
        window.open(homepage, "_blank")?.focus();
      },
    },
  ];
});

async function openFiles(inputFiles: FileList) {
  fsStore.importFilesOrProject(inputFiles);
}
</script>

<template>
  <n-page-header class="px-2 border-b border-gray-300">
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
            @select="handleDropdownOption"
          >
            <n-button :bordered="false" size="small" quaternary>
              File
            </n-button>
          </n-dropdown>
          <n-dropdown
            trigger="click"
            :options="viewOptions"
            @select="handleDropdownOption"
          >
            <n-button :bordered="false" size="small" quaternary>
              View
            </n-button>
          </n-dropdown>
          <n-dropdown
            trigger="click"
            :options="helpOptions"
            @select="handleDropdownOption"
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
