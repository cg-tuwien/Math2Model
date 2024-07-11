import { computed, ref, shallowRef, type Ref } from "vue";
import {
  SceneFileSchemaUrl,
  type SerializedModel,
  type SerializedScene,
} from "@/filesystem/scene-file";
import { makeFilePath, type FilePath } from "@/filesystem/reactive-files";
import { assert } from "@stefnotch/typestef/assert";
import type { ObjectUpdate } from "@/components/input/object-update";

export class ReadonlyVector3 {
  constructor(
    public readonly x: number,
    public readonly y: number,
    public readonly z: number
  ) {}

  static readonly zero = new ReadonlyVector3(0, 0, 0);

  serialize(): [number, number, number] {
    return [this.x, this.y, this.z];
  }

  static fromSerialized(data: [number, number, number]) {
    return new ReadonlyVector3(data[0], data[1], data[2]);
  }
}
export class ReadonlyEulerAngles {
  constructor(
    public readonly x: number,
    public readonly y: number,
    public readonly z: number
  ) {}

  static readonly identity = new ReadonlyEulerAngles(0, 0, 0);

  serialize(): [number, number, number] {
    return [this.x, this.y, this.z];
  }

  static fromSerialized(data: [number, number, number]) {
    return new ReadonlyEulerAngles(data[0], data[1], data[2]);
  }
}
export class ReadonlyQuaternion {
  constructor(
    public readonly x: number,
    public readonly y: number,
    public readonly z: number,
    public readonly w: number
  ) {}

  static readonly identity = new ReadonlyQuaternion(0, 0, 0, 1);

  serialize(): [number, number, number, number] {
    return [this.x, this.y, this.z, this.w];
  }

  static fromSerialized(data: [number, number, number, number]) {
    return new ReadonlyQuaternion(data[0], data[1], data[2], data[3]);
  }
}

export interface MaterialParameter {
  color: ReadonlyVector3;
  roughness: number;
  metallic: number;
  emissive: ReadonlyVector3;
}

export interface VirtualModelState {
  id: string;
  name: string;
  code: FilePath;
  position: ReadonlyVector3;
  rotation: ReadonlyEulerAngles;
  scale: number;
  material: MaterialParameter;
}

export interface VirtualSceneState {
  models: VirtualModelState[];
}

export class VirtualScene {
  private state: Ref<VirtualSceneState>;

  public readonly parametricShaders = computed(() => {
    const shaders = new Set<FilePath>();
    for (const model of this.state.value.models) {
      shaders.add(model.code);
    }
    return shaders;
  });
  constructor(state: Ref<VirtualSceneState>) {
    this.state = state;
  }

  clear() {
    this.state.value.models = [];
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
      (model) => model.id === key
    );
    if (index !== -1) {
      this.state.value.models.splice(index, 1);
      return true;
    }
    return false;
  }

  updateModels(keys: string[], update: ObjectUpdate<any>) {
    const keysMap = new Set(keys);
    for (const model of this.state.value.models) {
      if (!keysMap.has(model.id)) {
        continue;
      }
      update.applyTo(model);
    }
  }
}

export function useVirtualScene() {
  const state = ref<VirtualSceneState>({
    models: [],
  });
  const api = shallowRef<VirtualScene>(new VirtualScene(state));

  return {
    state: computed<VirtualSceneState>(() => state.value),
    api,
  };
}

function serializeModel(model: VirtualModelState): SerializedModel {
  return {
    type: "model",
    id: model.id,
    name: model.name,
    parametricShader: model.code,
    position: model.position.serialize(),
    rotation: model.rotation.serialize(),
    scale: model.scale,
    material: {
      color: model.material.color.serialize(),
      roughness: model.material.roughness,
      metallic: model.material.metallic,
      emissive: model.material.emissive.serialize(),
    },
  };
}

function deserializeModel(data: SerializedModel): VirtualModelState {
  assert(data.type === "model");
  return {
    id: data.id,
    name: data.name,
    code: makeFilePath(data.parametricShader),
    position: ReadonlyVector3.fromSerialized(data.position),
    rotation: ReadonlyEulerAngles.fromSerialized(data.rotation),
    scale: data.scale,
    material: {
      color: ReadonlyVector3.fromSerialized(data.material.color),
      roughness: data.material.roughness,
      metallic: data.material.metallic,
      emissive: ReadonlyVector3.fromSerialized(data.material.emissive),
    },
  };
}
