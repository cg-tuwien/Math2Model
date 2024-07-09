import { computedAsync } from "@vueuse/core";
import {
  reactive,
  computed,
  type Ref,
  type MaybeRefOrGetter,
  toValue,
} from "vue";

export interface ReadonlyFiles {
  listFiles(): FilePath[];
  readTextFile(name: FilePath): Promise<string> | null;
  readBinaryFile(name: FilePath): Promise<ArrayBuffer> | null;
  hasFile(name: FilePath): boolean;
}

export interface WritableFiles extends ReadonlyFiles {
  writeFile(name: FilePath, content: string): Promise<void>;
  renameFile(oldName: FilePath, newName: FilePath): Promise<void>;
  deleteFile(name: FilePath): Promise<void>;
}

export async function readOrCreateFile(
  sceneFiles: WritableFiles,
  name: FilePath,
  defaultContent: () => string
): Promise<string> {
  let content = await sceneFiles.readTextFile(name);
  if (content === null) {
    content = defaultContent();
    sceneFiles.writeFile(name, content);
  }
  return content;
}

export function makeFilePath(path: string): FilePath {
  return path as FilePath;
}

export type FilePath = string & { __filePath: never };

export class FileMetadata {
  constructor(public readonly version: number) {}
  equals(other: FileMetadata) {
    return this.version === other.version;
  }
}

export function useTextFile(
  name: MaybeRefOrGetter<FilePath | null>,
  fs: ReactiveFilesystem
): Readonly<Ref<string | null>> {
  return computedAsync(async () => {
    // Accessed synchronously, will be tracked
    const fileName = toValue(name);
    if (fileName === null) return null;
    const metadata = fs.files.value.get(fileName) ?? null;
    if (metadata === null) return null;
    // No longer tracked
    return await fs.readTextFile(fileName);
  }, null);
}

export function useBinaryFile(
  name: MaybeRefOrGetter<FilePath | null>,
  fs: ReactiveFilesystem
): Ref<ArrayBuffer | null> {
  return computedAsync(async () => {
    // Accessed synchronously, will be tracked
    const fileName = toValue(name);
    if (fileName === null) return null;
    const metadata = fs.files.value.get(fileName) ?? null;
    if (metadata === null) return null;
    // No longer tracked
    return await fs.readBinaryFile(fileName);
  }, null);
}

export class ReactiveFilesystem implements WritableFiles {
  private _files: Map<FilePath, FileMetadata> = reactive(new Map());
  private taskQueue: Promise<void> = Promise.resolve();
  private _filesReadonly = computed(
    () => this._files as ReadonlyMap<FilePath, FileMetadata>
  );
  private constructor(public readonly name: FilePath) {}

  get files() {
    return this._filesReadonly;
  }

  static async create(name: FilePath) {
    const instance = new ReactiveFilesystem(name);
    await instance.addTask(async (sceneDirectory) => {
      for await (const entry of (sceneDirectory as any).entries()) {
        const key: string = entry[0];
        instance._files.set(makeFilePath(key), new FileMetadata(0));
      }
    });
    return instance;
  }

  private async getSceneDirectory() {
    const root = await navigator.storage.getDirectory();
    return await root.getDirectoryHandle(this.name, { create: true });
  }

  private addTask<T>(
    makeTask: (sceneDirectory: FileSystemDirectoryHandle) => Promise<T>
  ) {
    const orderedTask = this.taskQueue.then(async () => {
      const sceneDirectory = await this.getSceneDirectory();
      return makeTask(sceneDirectory);
    });
    this.taskQueue = orderedTask.then(
      () => {},
      () => {}
    );
    return orderedTask;
  }

  listFiles(): FilePath[] {
    return Array.from(this._files.keys());
  }

  private setOrIncrementVersion(name: FilePath) {
    const metadata = this._files.get(name);
    if (metadata !== undefined) {
      this._files.set(name, new FileMetadata(metadata.version + 1));
    } else {
      this._files.set(name, new FileMetadata(0));
    }
  }

  writeFile(name: FilePath, content: string) {
    this.setOrIncrementVersion(name);
    return this.addTask(async (sceneDirectory) => {
      const fileHandle = await sceneDirectory.getFileHandle(
        encodeFilePath(name),
        {
          create: true,
        }
      );
      const writable = await fileHandle.createWritable();
      await writable.write(content);
      await writable.close();
    });
  }

  deleteFile(name: FilePath) {
    this._files.delete(name);
    return this.addTask(async (sceneDirectory) => {
      await sceneDirectory.removeEntry(encodeFilePath(name));
    });
  }

  renameFile(oldName: FilePath, newName: FilePath) {
    if (!this._files.has(oldName)) return Promise.resolve();
    this._files.delete(oldName);
    this.setOrIncrementVersion(newName);
    return this.addTask(async (sceneDirectory) => {
      // Ugh, the filesystem API doesn't have a rename function.
      const oldFileHandle = await sceneDirectory.getFileHandle(
        encodeFilePath(oldName)
      );
      const oldFile = await oldFileHandle.getFile();

      const newFileHandle = await sceneDirectory.getFileHandle(
        encodeFilePath(newName),
        { create: true }
      );
      const writable = await newFileHandle.createWritable();
      writable.write(oldFile);
      await writable.close();

      sceneDirectory.removeEntry(encodeFilePath(oldName));
    });
  }

  readTextFile(name: FilePath) {
    if (!this._files.has(name)) return null;

    return this.addTask(async (sceneDirectory) => {
      const handle = await sceneDirectory.getFileHandle(encodeFilePath(name));
      const file = await handle.getFile();
      return file.text();
    });
  }

  readBinaryFile(name: FilePath) {
    if (!this._files.has(name)) return null;

    return this.addTask(async (sceneDirectory) => {
      const handle = await sceneDirectory.getFileHandle(encodeFilePath(name));
      const file = await handle.getFile();
      return file.arrayBuffer();
    });
  }

  hasFile(name: FilePath) {
    return this._files.has(name);
  }
}

function encodeFilePath(name: FilePath) {
  return name;
}
