import { SceneFileName, SceneFileSchemaUrl } from "@/filesystem/scene-file";
import { makeFilePath } from "@/filesystem/reactive-files";
import DefaultParametric from "@/../parametric-renderer-core/shaders/DefaultParametric.wgsl?raw";
import { ReadonlyEulerAngles } from "./scene-state";
import type { ImportFilesList } from "@/stores/fs-store";

import sceneJson from "./example-scene/heart-sphere-morph/scene.json?raw";
import heartSphereGraph from "./example-scene/heart-sphere-morph/heart-sphere.graph?raw";
import heartSphereGraphWgsl from "./example-scene/heart-sphere-morph/heart-sphere.graph.wgsl?raw";
import heartTextureUrl from "./example-scene/heart-sphere-morph/heart-texture.png";

const textEncoder = new TextEncoder();

// The default scene has entirely different perfomance requirements, so we don't do the lazy loading
export const DefaultScene: ImportFilesList = {
  type: "in-memory",
  value: [
    {
      name: "scene.json",
      value: textEncoder.encode(sceneJson),
    },
    {
      name: "heart-sphere.graph",
      value: textEncoder.encode(heartSphereGraph),
    },
    {
      name: "heart-sphere.graph.wgsl",
      value: textEncoder.encode(heartSphereGraphWgsl),
    },
    {
      name: "heart-texture.png",
      value: await fetch(heartTextureUrl).then((v) => v.arrayBuffer()),
    },
  ],
};

export function createNewProject(): ImportFilesList {
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
          color: [45 / 255, 255 / 255, 150 / 255],
          roughness: 0.67,
          metallic: 0.0,
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

export function fileName(path: string) {
  const slash = path.lastIndexOf("/");
  if (slash === -1) {
    return path;
  } else {
    return path.substring(slash + 1);
  }
}
