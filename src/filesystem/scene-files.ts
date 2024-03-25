import { reactive, type ComputedRef, computed } from "vue";

export interface ReadonlySceneFiles {
  listFiles(): string[];
  readFile(name: string): string | null;
  hasFile(name: string): boolean;
}

export interface SceneFiles extends ReadonlySceneFiles {
  writeFile(name: string, content: string): void;
  deleteFile(name: string): void;
}

export function readOrCreateFile(
  sceneFiles: SceneFiles,
  name: string,
  defaultContent: () => string
): string {
  let content = sceneFiles.readFile(name);
  if (content === null) {
    content = defaultContent();
    sceneFiles.writeFile(name, content);
  }
  return content;
}

/**
 * An implementation of SceneFiles that delegates to another SceneFiles implementation.
 */
export class ReactiveSceneFiles implements SceneFiles {
  /**
   * A *reactive* map of file names and a version ID.
   */
  private _fileNames: Map<string, number> = reactive(new Map());
  public fileNames: ComputedRef<Map<string, number>> = computed(
    () => this._fileNames
  );

  private constructor(public readonly sceneFiles: SceneFiles) {}

  /**
   * Create a new ReactiveSceneFiles instance that delegates to the given SceneFiles instance.
   */
  static async create(sceneFiles: SceneFiles) {
    const instance = new ReactiveSceneFiles(sceneFiles);
    const files = instance.listFiles();
    for (const file of files) {
      instance._fileNames.set(file, 0);
    }
    return instance;
  }

  listFiles() {
    return this.sceneFiles.listFiles();
  }

  readFile(name: string) {
    return this.sceneFiles.readFile(name);
  }

  hasFile(name: string) {
    return this.sceneFiles.hasFile(name);
  }

  writeFile(name: string, content: string) {
    this.sceneFiles.writeFile(name, content);
    this._fileNames.set(name, (this._fileNames.get(name) ?? 0) + 1);
  }

  deleteFile(name: string) {
    this.sceneFiles.deleteFile(name);
    this._fileNames.delete(name);
  }
}

export class SceneFilesWithFilesystem implements SceneFiles {
  private files: Map<string, string> = new Map();
  private taskQueue: Promise<void> = Promise.resolve();
  private constructor(public readonly name: string) {}

  static async create(name: string) {
    const instance = new SceneFilesWithFilesystem(name);
    const sceneDirectory = await instance.getSceneDirectory();
    // TODO: Remove "as any" once TypeScript has support for this API
    for await (const [key, value] of (sceneDirectory as any).entries()) {
      console.log({ key, value });
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

  listFiles() {
    return Array.from(this.files.keys());
  }

  async writeFile(name: string, content: string) {
    this.files.set(name, content);
    this.taskQueue = this.taskQueue.then(async () => {
      const sceneDirectory = await this.getSceneDirectory();
      const fileHandle = await sceneDirectory.getFileHandle(name, {
        create: true,
      });
      const writable = await fileHandle.createWritable();
      await writable.write(content);
      writable.close();
    });
  }

  async deleteFile(name: string) {
    this.files.delete(name);
    this.taskQueue = this.taskQueue.then(async () => {
      const sceneDirectory = await this.getSceneDirectory();
      await sceneDirectory.removeEntry(name);
    });
  }

  readFile(name: string) {
    if (this.files.has(name)) {
      return this.files.get(name) ?? null;
    }
    return null;
  }

  hasFile(name: string) {
    return this.files.has(name);
  }
}
