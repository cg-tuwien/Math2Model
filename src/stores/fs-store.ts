import { computed, ref } from "vue";
import { acceptHMRUpdate, defineStore } from "pinia";
import {
  makeFilePath,
  type ReactiveFilesystem,
} from "@/filesystem/reactive-files";
import { sceneFilesPromise } from "@/globals";
import {
  BlobReader,
  BlobWriter,
  Uint8ArrayWriter,
  ZipReader,
  ZipWriter,
} from "@zip.js/zip.js";
import { SceneFileName } from "@/filesystem/scene-file";
import { assertUnreachable } from "@stefnotch/typestef/assert";

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
  data: ImportFilesList;
};

export type ImportFilesList =
  | {
      type: "files";
      value: FileList;
    }
  | { type: "zip"; value: ZipReader<unknown> }
  | {
      type: "in-memory";
      value: {
        name: string;
        value: ArrayBufferView | ArrayBuffer;
      }[];
    };

/**
 * The filesystem store for the app
 */
export const useFsStore = defineStore("fs-store", () => {
  const filesystemCommands = new FilesystemCommands(sceneFilesPromise);

  function exportToZip() {
    return filesystemCommands.add(async (sceneFiles) => {
      const zip = new ZipWriter(new BlobWriter("application/zip"), {
        bufferedWrite: true,
      });

      await Promise.all(
        sceneFiles.listFiles().map(async (filePath) => {
          const file = await sceneFiles.readFile(filePath);
          if (file === null) return;
          await zip.add(filePath, file.stream());
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

  /**
   * Limitations:
   * - No drag and drop
   * - Doesn't ask before overwriting
   */
  const importProjectDialog = ref<ImportProjectDialog | null>(null);

  function hasProject(): Promise<boolean> {
    return filesystemCommands.add((sceneFiles) =>
      Promise.resolve(sceneFiles.hasFile(SceneFileName))
    );
  }

  /**
   * Can trigger an import dialog.
   */
  async function importFilesOrProject(files: FileList) {
    const maybeProject = await tryImportProject(files);
    if (maybeProject !== null) {
      importProjectDialog.value = maybeProject;
      const noProject = !(await hasProject());
      if (noProject) {
        await finishImport("project");
      }
    } else {
      importFiles({ type: "files", value: files });
    }
  }

  /**
   * Can trigger an import dialog.
   */
  async function importInMemoryProject(
    files: {
      name: string;
      value: ArrayBufferView | ArrayBuffer;
    }[]
  ) {
    importProjectDialog.value = {
      data: { type: "in-memory", value: files },
    };
    const noProject = !(await hasProject());
    if (noProject) {
      await finishImport("project");
    }
  }

  async function importFiles(filesToImport: ImportFilesList) {
    filesystemCommands.add(async (sceneFiles) => {
      if (filesToImport.type === "in-memory") {
        for (let i = 0; i < filesToImport.value.length; i++) {
          const file = filesToImport.value[i];
          await sceneFiles.writeBinaryFile(makeFilePath(file.name), file.value);
        }
      } else if (filesToImport.type === "zip") {
        const entries = filesToImport.value.getEntriesGenerator();
        for await (const entry of entries) {
          const file = (await entry.getData?.(new Uint8ArrayWriter())) ?? null;
          if (file === null) continue;
          await sceneFiles.writeBinaryFile(makeFilePath(entry.filename), file);
        }
      } else if (filesToImport.type === "files") {
        for (let i = 0; i < filesToImport.value.length; i++) {
          const file = filesToImport.value[i];
          await sceneFiles.writeBinaryFile(makeFilePath(file.name), file);
        }
      } else {
        assertUnreachable(filesToImport);
      }
    });
  }

  async function finishImport(importAs: "files" | "project" | "cancel") {
    const dialog = importProjectDialog.value;
    if (dialog === null) return;
    importProjectDialog.value = null;

    if (importAs === "cancel") {
      // Nothing to do
    } else if (importAs === "files") {
      await importFiles(dialog.data);
    } else if (importAs === "project") {
      // Clear the current project before importing the new one
      await clearFiles();
      await importFiles(dialog.data);
    } else {
      assertUnreachable(importAs);
    }
  }

  async function clearFiles() {
    await filesystemCommands.add((sceneFiles) =>
      Promise.all(
        sceneFiles.listFiles().map((file) => sceneFiles.deleteFile(file))
      )
    );
  }

  return {
    importProjectDialog: computed(() => importProjectDialog.value),
    importFilesOrProject,
    importInMemoryProject,
    importFiles,
    finishImport,
    exportToZip,
    clearFiles,
  };
});

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

if (import.meta.hot) {
  import.meta.hot.accept(acceptHMRUpdate(useFsStore, import.meta.hot));
}
