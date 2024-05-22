import { ReadonlyEulerAngles } from "@/scenes/VirtualScene";
import {
  makeFilePath,
  readOrCreateFile,
  type ReactiveFiles,
  type FilePath,
} from "./reactive-files";
import {
  type SerializedScene,
  SceneFileSchemaUrl,
  deserializeScene,
} from "./scene-file";
import HeartSphere from "../../parametric-renderer-core/shaders/HeartSphere.wgsl?raw";
import { assert } from "@stefnotch/typestef/assert";
import { showError } from "@/notification";

export function getOrCreateScene(files: ReactiveFiles, scenePath: FilePath) {
  const { sceneData } = createDefaults(files, scenePath);
  const sceneFile = deserializeScene(sceneData);
  if (sceneFile === null) {
    // TODO: Properly show the error in the UI
    showError("Failed to deserialize scene file", sceneData);
    // Deserialization failed, which is likely due to a syntax error in the scene file.
    // Point the start file to the scene file so that the user can fix it!
    return { sceneFile: null, startFile: scenePath };
  } else {
    // Tries to find a parametric shader that exists in the scene file.
    // Otherwise it falls back to the scene file itself.
    const startFile = makeFilePath(
      sceneFile.models.find((model) =>
        files.hasFile(makeFilePath(model.parametricShader))
      )?.parametricShader ?? scenePath
    );
    return { sceneFile, startFile };
  }
}

/**
 * Creates the default files if they don't exist.
 */
function createDefaults(files: ReactiveFiles, scenePath: FilePath) {
  const defaultParametricShader = {
    name: makeFilePath("my-shader.wgsl"),
    value: () => HeartSphere,
  };

  const defaultScene = {
    name: scenePath,
    value: () => {
      const scene: SerializedScene = {
        $schema: SceneFileSchemaUrl,
        models: [
          {
            type: "model",
            id: crypto.randomUUID(),
            name: "Heart Sphere",
            parametricShader: defaultParametricShader.name,
            position: [0, 0, 0],
            rotation: ReadonlyEulerAngles.identity.serialize(),
            scale: 1,
            material: {
              color: [1, 0, 0],
              roughness: 0.5,
              metallic: 0.5,
              emissive: [0, 0, 0],
            },
          },
        ],
      };
      return JSON.stringify(scene, null, 2);
    },
  };
  if (files.hasFile(defaultScene.name)) {
    const sceneData = files.readFile(defaultScene.name);
    assert(sceneData !== null);
    return { sceneData };
  }

  const sceneData = readOrCreateFile(
    files,
    defaultScene.name,
    defaultScene.value
  );

  readOrCreateFile(
    files,
    defaultParametricShader.name,
    defaultParametricShader.value
  );
  return { sceneData };
}
