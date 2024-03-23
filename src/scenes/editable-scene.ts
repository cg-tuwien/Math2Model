import type { ShaderStore } from "@babylonjs/core";
import { computed, reactive, type ComputedRef } from "vue";

export interface ShaderInfo {
  /**
   * The name of the shader.
   */
  name: string;

  /**
   * If the shader can be edited by the user.
   */
  isEditable: boolean;

  /**
   * The source code of the shader.
   */
  source: string;
}

export type Shaders = ReadonlyMap<string, Readonly<ShaderInfo>>;

export interface EditableScene {
  /**
   * A *reactive* map of shaders.
   */
  readonly shaders: ComputedRef<Shaders>;

  /**
   * Updates a shader in the scene.
   * @param name The name of the shader to update.
   * @param shader The new shader.
   *
   * @returns A promise that resolves when the shader has been updated. TODO: Add compile errors here.
   */
  updateShaderSource(name: string, source: string): Promise<void>;
}

export class ShaderSources {
  /**
   * A *reactive* map of shaders.
   */
  private _shaders = reactive(new Map<string, ShaderInfo>());

  constructor(
    public readonly key: string,
    private shaderStore: typeof ShaderStore
  ) {}

  public shaders: ComputedRef<Shaders> = computed(() => this._shaders);

  shaderKey(name: string) {
    return this.key + "-" + name;
  }

  setShader(name: string, source: string, isEditable: boolean = true) {
    if (this._shaders.get(name)?.source === source) {
      return;
    }
    this._shaders.set(name, { name, source, isEditable });
    this.shaderStore.ShadersStoreWGSL[this.shaderKey(name)] = source;
  }

  canEditShader(name: string): boolean {
    const shader = this._shaders.get(name);
    if (!shader) {
      return false;
    }
    return shader.isEditable;
  }
}
