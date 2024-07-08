import { readonly } from "vue";
import { acceptHMRUpdate, defineStore } from "pinia";
import { useDark } from "@vueuse/core";
import type { ReactiveFiles } from "@/filesystem/reactive-files";
import { sceneFilesPromise } from "@/globals";
import { BlobWriter, ZipWriter } from "@zip.js/zip.js";

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
