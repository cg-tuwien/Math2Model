import { reactive, type ComputedRef, computed } from "vue";

export interface ReadonlyFiles {
  listFiles(): FilePath[];
  readFile(name: FilePath): string | null;
  hasFile(name: FilePath): boolean;
  waitFinished(): Promise<void>;
}

export interface WritableFiles extends ReadonlyFiles {
  writeFile(name: FilePath, content: string): void;
  deleteFile(name: FilePath): void;
}

export function readOrCreateFile(
  sceneFiles: WritableFiles,
  name: FilePath,
  defaultContent: () => string
): string {
  let content = sceneFiles.readFile(name);
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

/**
 * An implementation of SceneFiles that delegates to another SceneFiles implementation.
 */
export class ReactiveFiles implements WritableFiles {
  /**
   * A *reactive* map of file names and a version ID.
   */
  private _fileNames: Map<FilePath, number> = reactive(new Map());
  public fileNames: ComputedRef<Map<FilePath, number>> = computed(
    () => this._fileNames
  );

  private constructor(public readonly sceneFiles: WritableFiles) {}

  /**
   * Create a new ReactiveSceneFiles instance that delegates to the given SceneFiles instance.
   */
  static async create(sceneFiles: WritableFiles) {
    const instance = new ReactiveFiles(sceneFiles);
    const files = instance.listFiles();
    for (const file of files) {
      instance._fileNames.set(file, 0);
    }
    return instance;
  }

  listFiles() {
    return this.sceneFiles.listFiles();
  }

  readFile(name: FilePath) {
    return this.sceneFiles.readFile(name);
  }

  hasFile(name: FilePath) {
    return this.sceneFiles.hasFile(name);
  }

  writeFile(name: FilePath, content: string) {
    this.sceneFiles.writeFile(name, content);
    this._fileNames.set(name, (this._fileNames.get(name) ?? 0) + 1);
  }

  deleteFile(name: FilePath) {
    this.sceneFiles.deleteFile(name);
    this._fileNames.delete(name);
  }

  waitFinished() {
    return this.sceneFiles.waitFinished();
  }
}

export class FilesWithFilesystem implements WritableFiles {
  private files: Map<FilePath, string> = new Map();
  private taskQueue: Promise<void> = Promise.resolve();
  private constructor(public readonly name: FilePath) {}

  static async create(name: FilePath) {
    const instance = new FilesWithFilesystem(name);
    const sceneDirectory = await instance.getSceneDirectory();
    // TODO: Remove "as any" once TypeScript has support for this API
    for await (const [key, value] of (sceneDirectory as any).entries()) {
      // key: string
      // value: FileSystemFileHandle
      const file = await value.getFile();
      const text = await file.text();
      instance.files.set(key, text);
    }
    return instance;
  }

  private async getSceneDirectory() {
    const root = await navigator.storage.getDirectory();
    return await root.getDirectoryHandle(this.name, { create: true });
  }

  listFiles(): FilePath[] {
    return Array.from(this.files.keys());
  }

  writeFile(name: FilePath, content: string) {
    this.files.set(name, content);
    this.taskQueue = this.taskQueue.then(
      async () => {
        const sceneDirectory = await this.getSceneDirectory();
        const fileHandle = await sceneDirectory.getFileHandle(
          encodeFilePath(name),
          {
            create: true,
          }
        );
        const writable = await fileHandle.createWritable();
        await writable.write(content);
        await writable.close();
      },
      (error) => {
        console.error(error);
      }
    );
  }

  deleteFile(name: FilePath) {
    this.files.delete(name);
    this.taskQueue = this.taskQueue.then(
      async () => {
        const sceneDirectory = await this.getSceneDirectory();
        await sceneDirectory.removeEntry(encodeFilePath(name));
      },
      (error) => {
        console.error(error);
      }
    );
  }

  readFile(name: FilePath) {
    if (this.files.has(name)) {
      return this.files.get(name) ?? null;
    }
    return null;
  }

  hasFile(name: FilePath) {
    return this.files.has(name);
  }

  waitFinished() {
    return this.taskQueue.finally(() => {});
  }
}

function encodeFilePath(name: FilePath) {
  return name;
}
