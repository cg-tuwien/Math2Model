import { SceneFileName, SceneFileSchemaUrl } from "@/filesystem/scene-file";
import { makeFilePath, type FilePath } from "@/filesystem/reactive-files";
import DefaultParametric from "@/../parametric-renderer-core/shaders/DefaultParametric.wgsl?raw";
import HeartSphere from "@/../parametric-renderer-core/shaders/HeartSphere.wgsl?raw";
import { ReadonlyEulerAngles } from "./scene-state";
import { showError, showInfo } from "@/notification";

type ExampleProject = {
  name: string;
  files: {
    name: string;
    value: Uint8Array;
  }[];
};

const textEncoder = new TextEncoder();

export const createDefaultProject = (): ExampleProject => {
  const shaderName = makeFilePath("my-shader.wgsl");
  const shader = DefaultParametric;

  const scene = {
    $schema: SceneFileSchemaUrl,
    models: [
      {
        type: "model",
        id: crypto.randomUUID(),
        name: "Heart Sphere",
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
    name: "Example Project",
    files: [
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
};

export const createHeartSphereProject = (): ExampleProject => {
  const shaderName = makeFilePath("heart-sphere.wgsl");
  const shader = HeartSphere;

  const scene = {
    $schema: SceneFileSchemaUrl,
    models: [
      {
        type: "model",
        id: crypto.randomUUID(),
        name: "Heart Sphere",
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
    name: "Example Project",
    files: [
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
};

export async function getZipExample(path: FilePath): Promise<File | undefined> {
  try {
    // Fetch the file from the public directory
    const response = await fetch(path);

    // Check if the response is valid
    if (!response.ok) {
      showError(
        "Could not load example " +
          path +
          "\nServer responded with " +
          response.statusText
      );
    }

    // Convert the response to a Blob
    const blob = await response.blob();

    // Create a File object from the Blob
    const zipFile = new File([blob], path.substring(1), {
      type: "application/zip",
    });

    console.log("Zip file loaded:", zipFile);
    return zipFile;
  } catch (error) {
    showError("Could not load example " + path, { error: error });
  }
}
