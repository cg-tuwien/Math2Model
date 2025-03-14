import { assertUnreachable } from "@stefnotch/typestef/assert";
import { ContentFile, fileNameToType } from "./content-file";
import { ContentStore } from "./content-store";
import { setInterval } from "node:timers";

/** Loads everything from the FS into the content store, and then periodically syncs it back to the FS. */
export async function syncedContentStore(): Promise<ContentStore> {
  const contentStore = new ContentStore({
    actionLimits: {
      max: 100,
      maxTimePerFile: 1000,
    },
  });

  const timestamp = Date.now();

  const sceneDirectory = await getSceneDirectory();
  for await (const [name, fileHandle] of sceneDirectory.entries()) {
    if (fileHandle instanceof FileSystemFileHandle) {
      const isText = isTextFile(name);
      if (isText === null) continue;

      const data = isText
        ? await fileHandle.getFile().then((f) => f.text())
        : await fileHandle.getFile().then((f) => f.bytes());
      contentStore.runAction({
        kind: "add",
        file: new ContentFile(name, data),
        timestamp,
      });
    }
  }

  // Makes a list of the latest state of all changed files
  let filesToSync: FilesToSync = new Map();
  contentStore.addListener((action) => {
    if (action.kind === "add") {
      filesToSync.set(action.file.name, action.file.data);
    } else if (action.kind === "remove") {
      filesToSync.set(action.file.name, null);
    } else if (action.kind === "replace") {
      filesToSync.set(action.oldFile.name, null);
      filesToSync.set(action.newFile.name, action.newFile.data);
    } else {
      assertUnreachable(action);
    }
  });

  // Periodically sync the state
  let isSyncing = false;
  setInterval(() => {
    if (filesToSync.size === 0) return;
    if (isSyncing) return;

    isSyncing = true;
    const toSync = filesToSync;
    filesToSync = new Map();
    syncFiles(toSync).finally(() => {
      isSyncing = false;
    });
  }, 5000);

  return contentStore;
}

type FilesToSync = Map<string, Uint8Array | string | null>;

async function syncFiles(toSync: FilesToSync) {
  const sceneDirectory = await getSceneDirectory();
  for (const [name, data] of toSync) {
    if (data === null) {
      await sceneDirectory.removeEntry(name);
    } else {
      const fileHandle = await sceneDirectory.getFileHandle(name, {
        create: true,
      });
      const writable = await fileHandle.createWritable();
      try {
        await writable.write(data);
      } finally {
        await writable.close();
      }
    }
  }
}

/** Returns whether this is a text file or not. */
function isTextFile(name: string): boolean | null {
  const contentType = fileNameToType(name);
  if (
    contentType.kind === "graph" ||
    contentType.kind === "shader" ||
    contentType.kind === "json"
  ) {
    return true;
  } else if (contentType.kind === "image") {
    return false;
  } else if (contentType.kind === "unknown") {
    return null;
  } else {
    assertUnreachable(contentType);
  }
}

const SCENE_FILES = "some-key";

async function getSceneDirectory(): Promise<FileSystemDirectoryHandle> {
  const root = await navigator.storage.getDirectory();
  return await root.getDirectoryHandle(SCENE_FILES, { create: true });
}
