import { showError } from "@/notification";
import { computedAsync } from "@vueuse/core";
import {
  reactive,
  computed,
  type Ref,
  type MaybeRefOrGetter,
  toValue,
  shallowRef,
  watchEffect,
  type WatchStopHandle,
} from "vue";

export interface ReadonlyFiles {
  listFiles(): FilePath[];
  readTextFile(name: FilePath): Promise<string> | null;
  readBinaryFile(name: FilePath): Promise<ArrayBuffer> | null;
  hasFile(name: FilePath): boolean;
}

export interface WritableFiles extends ReadonlyFiles {
  writeTextFile(name: FilePath, content: string): Promise<void>;
  writeBinaryFile(name: FilePath, content: ArrayBuffer | File): Promise<void>;
  renameFile(oldName: FilePath, newName: FilePath): Promise<void>;
  deleteFile(name: FilePath): Promise<void>;
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

export type AsyncComputed<T> = Readonly<
  Ref<T | null> & { stop: WatchStopHandle }
>;

// Basically https://github.com/vueuse/vueuse/blob/e71eb1e9818be73e179915c5efda4f37b8a460c9/packages/core/computedAsync/index.ts#L47
// but it can be stopped
export function asyncComputed<T>(
  fn: () => Promise<T>,
  onError: (error: any) => void
): AsyncComputed<T> {
  const result = shallowRef<T | null>(null);
  let counter = 0;
  let stop = watchEffect(async () => {
    counter += 1;
    const expectedCounter = counter;
    fn().then(
      (value) => {
        if (counter === expectedCounter) {
          result.value = value;
        }
      },
      (error) => {
        onError(error);
      }
    );
  });
  (result as any).stop = stop;
  return result as any as AsyncComputed<T>;
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

  static async create(name: FilePath): Promise<ReactiveFilesystem> {
    const instance = new ReactiveFilesystem(name);
    await instance.addTask(async (sceneDirectory) => {
      for await (const entry of (sceneDirectory as any).entries()) {
        const key: string = entry[0];
        instance._files.set(makeFilePath(key), new FileMetadata(0));
      }
    });
    return instance;
  }

  private async getSceneDirectory(): Promise<FileSystemDirectoryHandle> {
    const root = await navigator.storage.getDirectory();
    return await root.getDirectoryHandle(this.name, { create: true });
  }

  private addTask<T>(
    makeTask: (sceneDirectory: FileSystemDirectoryHandle) => Promise<T>
  ): Promise<T> {
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

  private setOrIncrementVersion(name: FilePath): void {
    const metadata = this._files.get(name);
    if (metadata !== undefined) {
      this._files.set(name, new FileMetadata(metadata.version + 1));
    } else {
      this._files.set(name, new FileMetadata(0));
    }
  }

  writeTextFile(name: FilePath, content: string): Promise<void> {
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

  writeBinaryFile(name: FilePath, content: ArrayBuffer | File): Promise<void> {
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

  deleteFile(name: FilePath): Promise<void> {
    this._files.delete(name);
    return this.addTask(async (sceneDirectory) => {
      await sceneDirectory.removeEntry(encodeFilePath(name));
    });
  }

  renameFile(oldName: FilePath, newName: FilePath): Promise<void> {
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

  /**
   * Synchronously accesses the file metadata. Thus, is will be tracked in a reactive context.
   */
  readTextFile(name: FilePath): Promise<string> | null {
    if (!this._files.has(name)) return null;

    return this.addTask(async (sceneDirectory) => {
      const handle = await sceneDirectory.getFileHandle(encodeFilePath(name));
      const file = await handle.getFile();
      return file.text();
    });
  }

  readBinaryFile(name: FilePath): Promise<ArrayBuffer> | null {
    if (!this._files.has(name)) return null;

    return this.addTask(async (sceneDirectory) => {
      const handle = await sceneDirectory.getFileHandle(encodeFilePath(name));
      const file = await handle.getFile();
      return file.arrayBuffer();
    });
  }

  hasFile(name: FilePath): boolean {
    return this._files.has(name);
  }
}

function encodeFilePath(name: FilePath): FilePath {
  return name;
}
