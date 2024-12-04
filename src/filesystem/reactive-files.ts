import { FineMap, type MapChange } from "@/fine-collections";
import { type WatchStopHandle, type WatchOptions } from "vue";

export interface ReadonlyFiles {
  listFiles(): FilePath[];
  readTextFile(
    name: FilePath,
    options?: { signal?: AbortSignal }
  ): Promise<string> | null;
  readBinaryFile(
    name: FilePath,
    options?: { signal?: AbortSignal }
  ): Promise<ArrayBuffer> | null;
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

export class ReactiveFilesystem implements WritableFiles {
  private _files: FineMap<FilePath, FileMetadata> = new FineMap();
  private taskQueue: Promise<void> = Promise.resolve();

  private constructor(public readonly name: FilePath) {}

  /**
   * Watches all changes to the files in the filesystem.
   */
  watch(
    callback: (change: MapChange<FilePath, FileMetadata>) => void,
    options: WatchOptions<boolean> = {}
  ): WatchStopHandle {
    return this._files.watch(callback, options);
  }

  /**
   * Starts from an empty filesystem and watches all changes to the files in the filesystem.
   */
  watchFromStart(
    callback: (change: MapChange<FilePath, FileMetadata>) => void,
    options: WatchOptions<boolean> = {}
  ): WatchStopHandle {
    for (const [key, value] of this._files.entries()) {
      callback({ type: "insert", key, value });
    }
    return this._files.watch(callback, options);
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
   * Guarantees ordering of reads and writes.
   * If multiple reads are requested, they will resolve in the order they were requested.
   */
  readTextFile(
    name: FilePath,
    options?: { signal?: AbortSignal }
  ): Promise<string> | null {
    if (!this._files.has(name)) return null;

    return this.addTask(async (sceneDirectory) => {
      const handle = await sceneDirectory.getFileHandle(encodeFilePath(name));
      const file = await handle.getFile();
      return file.text();
    });
  }

  readBinaryFile(
    name: FilePath,
    options?: { signal?: AbortSignal }
  ): Promise<ArrayBuffer> | null {
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

/**
 *  filename CC BY SA-3.0 https://stackoverflow.com/a/190933/3492994
 */
export function getFileExtension(filename: string): string | null {
  var ext = /^.+\.([^.]+)$/.exec(filename);
  return ext == null ? null : ext[1];
}
