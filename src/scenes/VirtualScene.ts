import { Quaternion, Vector3 } from "@babylonjs/core";
import { computed, readonly, ref, shallowRef, type Ref } from "vue";
import {
  SceneFileSchemaUrl,
  type SerializedModel,
  type SerializedScene,
} from "@/filesystem/scene-file";
import { makeFilePath, type FilePath } from "@/filesystem/reactive-files";
import { assert } from "@stefnotch/typestef/assert";

export class ReadonlyVector3 {
  constructor(
    public readonly x: number,
    public readonly y: number,
    public readonly z: number,
  ) {}

  static readonly zero = new ReadonlyVector3(0, 0, 0);

  static fromVector3(v: Vector3) {
    return new ReadonlyVector3(v.x, v.y, v.z);
  }

  toVector3() {
    return new Vector3(this.x, this.y, this.z);
  }

  serialize(): [number, number, number] {
    return [this.x, this.y, this.z];
  }

  static fromSerialized(data: [number, number, number]) {
    return new ReadonlyVector3(data[0], data[1], data[2]);
  }
}
export class ReadonlyQuaternion {
  constructor(
    public readonly x: number,
    public readonly y: number,
    public readonly z: number,
    public readonly w: number,
  ) {}

  static readonly identity = new ReadonlyQuaternion(0, 0, 0, 1);

  static fromQuaternion(q: Quaternion) {
    return new ReadonlyQuaternion(q.x, q.y, q.z, q.w);
  }

  toQuaternion() {
    return new Quaternion(this.x, this.y, this.z, this.w);
  }

  serialize(): [number, number, number, number] {
    return [this.x, this.y, this.z, this.w];
  }

  static fromSerialized(data: [number, number, number, number]) {
    return new ReadonlyQuaternion(data[0], data[1], data[2], data[3]);
  }
}

export interface VirtualModelState {
  id: string;
  name: string;
  code: ShaderCodeRef;
  position: ReadonlyVector3;
  rotation: ReadonlyQuaternion;
  scale: number;
}

export interface VirtualModelUpdate {
  name?: string;
  code?: ShaderCodeRef;
  position?: ReadonlyVector3;
  rotation?: ReadonlyVector3;
  scale?: number;
}

export interface ShaderCodeRef {
  readonly vertexFile: FilePath;
  readonly fragmentFile: FilePath;
}

export interface VirtualSceneState {
  models: VirtualModelState[];
}

export class VirtualScene {
  private state: Ref<VirtualSceneState>;

  public readonly parametricShaders = computed(() => {
    const shaders = new Set<FilePath>();
    for (const model of this.state.value.models) {
      shaders.add(model.code.vertexFile);
    }
    return shaders;
  });
  constructor(state: Ref<VirtualSceneState>) {
    this.state = state;
  }

  serialize(): SerializedScene {
    const models = this.state.value.models.map(serializeModel);
    return {
      $schema: SceneFileSchemaUrl,
      models,
    };
  }

  fromSerialized(data: SerializedScene) {
    this.state.value.models = data.models.map(deserializeModel);
  }

  addModel(model: VirtualModelState) {
    this.state.value.models.push(model);
  }

  removeModel(key: string): boolean {
    const index = this.state.value.models.findIndex(
      (model) => model.id === key,
    );
    if (index !== -1) {
      this.state.value.models.splice(index, 1);
      return true;
    }
    return false;
  }

  updateModels(keys: string[], update: VirtualModelUpdate) {
    const keysMap = new Set(keys);
    for (const model of this.state.value.models) {
      if (keysMap.has(model.id)) {
        if (update.name !== undefined) {
          model.name = update.name;
        }
        if (update.code !== undefined) {
          model.code = update.code;
        }
        if (update.position !== undefined) {
          model.position = update.position;
        }
        if (update.rotation !== undefined) {
          model.rotation = ReadonlyQuaternion.fromQuaternion(
            Quaternion.FromEulerAngles(
              update.rotation.x,
              update.rotation.y,
              update.rotation.z,
            ),
          );
        }
        if (update.scale !== undefined) {
          model.scale = update.scale;
        }
      }
    }
  }
}

export function useVirtualScene() {
  const state = ref<VirtualSceneState>({
    models: [],
  });
  const api = shallowRef<VirtualScene>(new VirtualScene(state));

  return {
    state: computed(() => state.value),
    api,
  };
}

function serializeModel(model: VirtualModelState): SerializedModel {
  return {
    type: "model",
    id: model.id,
    name: model.name,
    parametricShader: model.code.vertexFile,
    fragmentShader: model.code.fragmentFile,
    position: model.position.serialize(),
    rotation: model.rotation.serialize(),
    scale: model.scale,
  };
}

function deserializeModel(data: SerializedModel): VirtualModelState {
  assert(data.type === "model");
  return {
    id: data.id,
    name: data.name,
    code: {
      vertexFile: makeFilePath(data.parametricShader),
      fragmentFile: makeFilePath(data.fragmentShader),
    },
    position: ReadonlyVector3.fromSerialized(data.position),
    rotation: ReadonlyQuaternion.fromSerialized(data.rotation),
    scale: data.scale,
  };
}
