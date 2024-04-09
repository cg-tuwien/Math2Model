import {
  Vector3,
  HemisphericLight,
  UniformBuffer,
  Observable,
} from "@babylonjs/core";
import vertexShader from "./ModelDisplayVirtualScene.vert.wgsl?raw";
import {
  makeFilePath,
  readOrCreateFile,
  type FilePath,
  type SceneFiles,
} from "@/filesystem/scene-files";
import type { BaseScene } from "./BaseScene";
import { VirtualModel, type SerializedActor } from "./VirtualModel";
import { DisposableStack } from "../disposable-stack";
import { showError } from "@/notification";

interface SerializedScene {
  models: SerializedActor[];
}

export class ModelDisplayVirtualScene implements Disposable {
  private _disposables = new DisposableStack();
  private _models: VirtualModel[] = [];
  public readonly updateObservable: Observable<void> = new Observable();
  public readonly globalUBO: UniformBuffer;
  public readonly key = ModelDisplayVirtualScene.name;
  constructor(public readonly scene: BaseScene, public files: SceneFiles) {
    let light = this._disposables.addDisposable(
      new HemisphericLight("light1", new Vector3(0, 1, 0), scene)
    );
    light.intensity = 0.7;

    this.globalUBO = this._disposables.addDisposable(
      new UniformBuffer(scene.engine)
    );
    this.globalUBO.addUniform("iTime", 1);
    this.globalUBO.addUniform("iTimeDelta", 1);
    this.globalUBO.addUniform("iFrame", 1);
    this.globalUBO.update();
    this.updateObservable.add(() => {
      this.globalUBO.updateFloat("iTime", scene.time / 1000);
      this.globalUBO.updateFloat("iTimeDelta", scene.deltaTime / 1000);
      this.globalUBO.updateFloat("iFrame", scene.frame);
      this.globalUBO.update();
    });

    // TODO: Gizmo
    // Including
    // - GPU picking
    // - Dragging and then updating the filesystem (aka serialize the scene)
    // Excluding
    // - Switching to the shader. Instead we should have a scene tree/property editor, where the user can click on the referenced shaders to edit them.

    this.loadScene();
  }

  loadScene() {
    let sceneFile = readOrCreateFile(
      this.files,
      makeFilePath("scene.json"),
      () =>
        JSON.stringify(
          {
            models: [
              {
                key: "some-random-key",
                name: "my-model",
                code: {
                  vertexFile: makeFilePath("my-shader.vert"),
                  fragmentFile: makeFilePath("my-shader.frag"),
                },
                position: [0, 0, 0],
                rotation: [0, 0, 0, 1],
              },
            ],
          } as SerializedScene,
          undefined,
          2
        )
    );
    try {
      let data = JSON.parse(sceneFile);
      this.deserializeInto(data);
    } catch (e) {
      showError("Failed to load scene json.", e);
    }
  }

  update() {
    this.updateObservable.notifyObservers();
  }

  [Symbol.dispose](): void {
    this.updateObservable.clear();
    this._disposables[Symbol.dispose]();
  }

  // TODO: Use the referenced files for granular shader hot-reloading
  getVertexAndFragmentShaders() {
    return new ShaderFileReader(this.files);
  }

  internalAddModel(model: VirtualModel) {
    this._models.push(model);
    this._disposables.addDisposable(model);
  }

  static serialize(scene: ModelDisplayVirtualScene): SerializedScene {
    return {
      models: scene._models.map(VirtualModel.serialize),
    };
  }

  private deserializeInto(data: SerializedScene) {
    for (const actorData of data.models) {
      VirtualModel.deserialize(actorData, this);
    }
  }
}

export class ShaderFileReader {
  public readonly referencedFiles: FilePath[] = [];
  constructor(private files: SceneFiles) {}

  readVertexShader(name: FilePath): string {
    this.referencedFiles.push(name);
    return readOrCreateFile(this.files, name, () => vertexShader);
  }

  readFragmentShader(name: FilePath): string {
    this.referencedFiles.push(name);
    return readOrCreateFile(
      this.files,
      name,
      () => `
    varying vNormal : vec3<f32>;
    varying vUV : vec2<f32>;
    @fragment
    fn main(input : FragmentInputs) -> FragmentOutputs {
        fragmentOutputs.color = vec4<f32>(input.vUV,1.0, 1.0);
    }
`
    );
  }
}
