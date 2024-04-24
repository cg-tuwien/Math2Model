import type { ComputeShader } from "@babylonjs/core";
import type {
  FilePath,
  HasReactiveFiles,
  ReadonlyFiles,
} from "./reactive-files";
import { computed } from "vue";
import { assert } from "@stefnotch/typestef/assert";

export class ShaderFiles {
  constructor(private files: ReadonlyFiles & HasReactiveFiles) {}

  getComputeShader(
    file: FilePath,
    createShader: (shaderCode: string) => ComputeShader
  ) {
    return computed(() => {
      const fileId = this.files.fileNames.value.get(file);
      if (fileId === undefined) {
        return null;
      }
      const shaderCode = this.files.readFile(file);
      assert(shaderCode !== null);

      // Because we're calling this function inside a computed, Vue.js will track all the reactive dependencies
      // So if createShader depends on a reactive engine, then the engine will be a reactive dependency
      return createShader(shaderCode);
    });
  }
}
