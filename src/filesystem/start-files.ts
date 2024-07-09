import { ReadonlyEulerAngles } from "@/scenes/VirtualScene";
import {
  makeFilePath,
  ReactiveFilesystem,
  type FilePath,
  type WritableFiles,
} from "./reactive-files";
import { type SerializedScene, SceneFileSchemaUrl } from "./scene-file";
import HeartSphere from "@/shaders/HeartSphere.wgsl?raw";
import { assert } from "@stefnotch/typestef/assert";

async function readOrCreateFile(
  sceneFiles: WritableFiles,
  name: FilePath,
  defaultContent: () => string
): Promise<string> {
  let content = await sceneFiles.readTextFile(name);
  if (content === null) {
    content = defaultContent();
    sceneFiles.writeTextFile(name, content);
  }
  return content;
}

// TODO: Use this function to create an example.
/**
 * Creates the default files if they don't exist.
 */
export function createDefaults(files: ReactiveFilesystem, scenePath: FilePath) {
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
    const sceneData = files.readTextFile(defaultScene.name);
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
