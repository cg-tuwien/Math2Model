import { readonly } from "vue";
import { acceptHMRUpdate, defineStore } from "pinia";
import { useDark } from "@vueuse/core";
import { makeFilePath, type ReactiveFiles } from "@/filesystem/reactive-files";
import { sceneFilesPromise } from "@/globals";
import { BlobReader, BlobWriter, ZipReader, ZipWriter } from "@zip.js/zip.js";
import { SceneFileName } from "@/filesystem/scene-file";

class FilesystemCommands {
  private commands: Promise<any>;
  private sceneFiles: ReactiveFiles = null as any;
  constructor(sceneFilesPromise: Promise<ReactiveFiles>) {
    this.commands = sceneFilesPromise.then((v) => {
      this.sceneFiles = v;
    });
  }

  add<T>(callback: (sceneFiles: ReactiveFiles) => Promise<T>): Promise<T> {
    const newPromise = this.commands.then(() => callback(this.sceneFiles));
    this.commands = newPromise;
    return newPromise;
  }
}

const textFileExtensions = [
  "wgsl",
  "json",
  "txt",
  "md",
  "glsl",
  "vert",
  "frag",
];

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
      const files = sceneFiles.listFiles().flatMap((filePath) => {
        const file = sceneFiles.readFile(filePath);
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
    });
  }

  return {
    isDark: readonly(isDark),
    setIsDark: (newValue: boolean) => {
      isDark.value = newValue;
    },
    exportToZip,
  };
});

if (import.meta.hot) {
  import.meta.hot.accept(acceptHMRUpdate(useStore, import.meta.hot));
}

export async function startImportFiles(files: FileList): Promise<{
  isProject: boolean;
  files: FileList;
}> {
  if (files.length === 1 && files[0].name.endsWith(".zip")) {
    const entries = new ZipReader(
      new BlobReader(files[0])
    ).getEntriesGenerator();
    for await (const entry of entries) {
      if (entry.filename === SceneFileName) {
        return { isProject: true, files };
      }
    }
  }
  for (let i = 0; i < files.length; i++) {
    if (makeFilePath(files[i].name) === SceneFileName) {
      return { isProject: true, files };
    }
  }
  return { isProject: false, files };

  /*const filesList: {
    name: string;
    value: string;
  }[] = [];
  if (files.length === 1) {
    const isZip = files[0].name.endsWith(".zip");
  }

  for (let i = 0; i < files.length; i++) {
    const file = files[i];
    const reader = new FileReader();
    reader.onload = () => {
      const value = reader.result as string;
      filesList.push({ name: file.name, value });
    };

    reader.readAsText(files[i], "utf-8");
  }*/

  // TODO: Open any file (or folder?)
  // Then
  // - If it is a file list: Import all files
  // - If it is a folder: Import all files
  // - If it looks like a zip: Import all files
  // TODO: If the file list has a `scene.json`, then ask the user
  // - Add to current project
  // - Or open as new project
  // TODO: Drag and drop support (onto the file list)
}
