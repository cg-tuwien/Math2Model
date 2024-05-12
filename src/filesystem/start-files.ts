import { ReadonlyQuaternion } from "@/engine/VirtualScene";
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
    name: makeFilePath("my-shader.vert.wgsl"),
    value: () => HeartSphere,
  };
  const defaultFragmentShader = {
    name: makeFilePath("my-shader.frag.wgsl"),
    value: () => `
    varying vNormal : vec3<f32>;
    varying vUV : vec2<f32>;
    @fragment
    fn main(input : FragmentInputs) -> FragmentOutputs {
        fragmentOutputs.color = vec4<f32>(input.vUV,1.0, 1.0);
    }
`,
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
            fragmentShader: defaultFragmentShader.name,
            parametricShader: defaultParametricShader.name,
            position: [0, 0, 0],
            rotation: ReadonlyQuaternion.identity.serialize(),
            scale: 1,
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
  readOrCreateFile(
    files,
    defaultFragmentShader.name,
    defaultFragmentShader.value
  );
  return { sceneData };
}
