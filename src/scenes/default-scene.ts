import { SceneFileName, SceneFileSchemaUrl } from "@/filesystem/scene-file";
import { makeFilePath } from "@/filesystem/reactive-files";
import DefaultParametric from "@/../parametric-renderer-core/shaders/DefaultParametric.wgsl?raw";
import { ReadonlyEulerAngles } from "./scene-state";
import type { ImportFilesList } from "@/stores/fs-store";

const textEncoder = new TextEncoder();

const heartSphereMorph = import.meta.glob(
  "./example-scene/heart-sphere-morph/*",
  {
    query: "?raw",
    import: "default",
    eager: true,
  }
);

export const DefaultScene = toProject(heartSphereMorph);

// Unused, shows how to programmatically create an example scene.
function createDefaultProject(): ImportFilesList {
  const shaderName = makeFilePath("my-shader.wgsl");
  const shader = DefaultParametric;

  const scene = {
    $schema: SceneFileSchemaUrl,
    models: [
      {
        type: "model",
        id: crypto.randomUUID(),
        name: "Basic Plane",
        parametricShader: shaderName,
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

  return {
    type: "in-memory",
    value: [
      {
        name: SceneFileName,
        value: textEncoder.encode(JSON.stringify(scene, null, 2)),
      },
      {
        name: shaderName,
        value: textEncoder.encode(shader),
      },
    ],
  };
}

function toProject(files: Record<string, unknown>): ImportFilesList {
  return {
    type: "in-memory",
    value: Object.entries(files).map(([name, contents]) => {
      return {
        name: fileName(name),
        value: textEncoder.encode(contents as string),
      };
    }),
  };
}

export function fileName(path: string) {
  const slash = path.lastIndexOf("/");
  if (slash === -1) {
    return path;
  } else {
    return path.substring(slash + 1);
  }
}
