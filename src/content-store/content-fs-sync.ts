import { assertUnreachable } from "@stefnotch/typestef/assert";
import { ContentFile, fileNameToType } from "./content-file";
import { ContentStore } from "./content-store";

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

  const filesOnDisk = new Set<string>(
    contentStore.getFiles().map((v) => v.name)
  );

  contentStore.addListener((action) => {
    // TODO: Track the changed files, and periodically sync them back
    // https://stackoverflow.com/a/65620774/3492994
  });

  return contentStore;
}

async function syncFiles(
  contentStore: ContentStore,
  filesOnDisk: Set<string>
) {}

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
