import { readonly, ref } from "vue";
import { acceptHMRUpdate, defineStore } from "pinia";
import { useDark } from "@vueuse/core";
import {
  makeFilePath,
  type ReactiveFilesystem,
} from "@/filesystem/reactive-files";
import { sceneFilesPromise } from "@/globals";
import { BlobReader, BlobWriter, ZipReader, ZipWriter } from "@zip.js/zip.js";
import { SceneFileName } from "@/filesystem/scene-file";

/**
 * Deals with the fact that the filesystem is created asynchronously.
 * It just queues up commands until the filesystem is ready.
 */
class FilesystemCommands {
  private commands: Promise<any>;
  private sceneFiles: ReactiveFilesystem = null as any;
  constructor(sceneFilesPromise: Promise<ReactiveFilesystem>) {
    this.commands = sceneFilesPromise.then((v) => {
      this.sceneFiles = v;
    });
  }

  add<T>(callback: (sceneFiles: ReactiveFilesystem) => Promise<T>): Promise<T> {
    const newPromise = this.commands.then(() => callback(this.sceneFiles));
    this.commands = newPromise;
    return newPromise;
  }
}

export type ImportProjectDialog = {
  data:
    | {
        type: "files";
        value: FileList;
      }
    | { type: "zip"; value: ZipReader<unknown> }
    | {
        type: "in-memory";
        value: {
          name: string;
          value: ArrayBuffer;
        }[];
      };
};

/**
 * The main store for the app
 */
export const useStore = defineStore("store", () => {
  // Changes the theme of the app and tells TailwindCSS about it
  const isDark = useDark({
    selector: "body",
  });

  const filesystemCommands = new FilesystemCommands(sceneFilesPromise);

  function exportToZip() {
    return filesystemCommands.add(async (sceneFiles) => {
      const zip = new ZipWriter(new BlobWriter("application/zip"), {
        bufferedWrite: true,
      });

      await Promise.all(
        sceneFiles.listFiles().map(async (filePath) => {
          const file = await sceneFiles.readBinaryFile(filePath);
          if (file === null) return;
          await zip.add(filePath, new Blob([file]).stream());
        })
      );

      const blobUrl = URL.createObjectURL(await zip.close());
      const a = document.createElement("a");
      a.href = blobUrl;
      a.download = "project.zip";
      a.click();
      URL.revokeObjectURL(blobUrl);
    });
  }

  const importProjectDialog = ref<ImportProjectDialog | null>(null);

  async function importFilesOrProject(files: FileList) {
    const maybeProject = await tryImportProject(files);
    if (maybeProject !== null) {
      importProjectDialog.value = maybeProject;
      // TODO: If the file list has a `scene.json`, then ask the user
      // - Add to current project
      // - Or open as new project
    } else {
      filesystemCommands.add(async (sceneFiles) => {
        for (let i = 0; i < files.length; i++) {
          const file = files[i];
          await sceneFiles.writeBinaryFile(makeFilePath(file.name), file);
        }
      });
    }

    // TODO: Drag and drop support (onto the file list)
  }

  async function importInMemoryProject(
    files: {
      name: string;
      value: ArrayBuffer;
    }[]
  ) {
    importProjectDialog.value = {
      data: { type: "in-memory", value: files },
    };
  }

  return {
    isDark: readonly(isDark),
    setIsDark: (newValue: boolean) => {
      isDark.value = newValue;
    },
    importFilesOrProject,
    importInMemoryProject,
    exportToZip,
  };
});

if (import.meta.hot) {
  import.meta.hot.accept(acceptHMRUpdate(useStore, import.meta.hot));
}

async function tryImportProject(
  files: FileList
): Promise<ImportProjectDialog | null> {
  if (files.length === 1 && files[0].name.endsWith(".zip")) {
    const reader = new ZipReader(new BlobReader(files[0]));
    const entries = reader.getEntriesGenerator();
    for await (const entry of entries) {
      if (entry.filename === SceneFileName) {
        return { data: { type: "zip", value: reader } };
      }
    }
  }
  for (let i = 0; i < files.length; i++) {
    if (makeFilePath(files[i].name) === SceneFileName) {
      return { data: { type: "files", value: files } };
    }
  }
  return null;
}
