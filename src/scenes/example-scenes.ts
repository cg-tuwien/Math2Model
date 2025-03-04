import { SceneFileName, SceneFileSchemaUrl } from "@/filesystem/scene-file";
import { makeFilePath, type FilePath } from "@/filesystem/reactive-files";
import DefaultParametric from "@/../parametric-renderer-core/shaders/DefaultParametric.wgsl?raw";
import { ReadonlyEulerAngles } from "./scene-state";
import { showError } from "@/notification";
import type { ImportFilesList } from "@/stores/fs-store";
import { assert } from "@stefnotch/typestef/assert";
import { ZipReader } from "@zip.js/zip.js";

type ExampleProject = {
  key: string;
  name: string;
  files: () => Promise<ImportFilesList>;
};

const textEncoder = new TextEncoder();

export function createDefaultProject(): Promise<ImportFilesList> {
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

  return Promise.resolve({
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
  });
}

async function getZipExample(path: FilePath): Promise<File | undefined> {
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
    const zipFile = new File([blob], path, {
      type: "application/zip",
    });

    console.log("Zip file loaded:", zipFile);
    return zipFile;
  } catch (error) {
    showError("Could not load example " + path, { error: error });
  }
}

async function importZipExample(path: string): Promise<ImportFilesList> {
  const zipFile = await getZipExample(makeFilePath(path));
  assert(zipFile !== undefined);
  return {
    type: "zip",
    value: new ZipReader(zipFile.stream()),
  };
}

export const ExampleProjects: ExampleProject[] = [
  {
    key: crypto.randomUUID(),
    name: "Example Scene",
    files: createDefaultProject,
  },
  {
    key: crypto.randomUUID(),
    name: "Heart Sphere Scene",
    files: () => importZipExample("./HeartSphereMorph.zip"),
  },
  {
    key: crypto.randomUUID(),
    name: "Temple Scene",
    files: () => importZipExample("./TempleExample.zip"),
  },
  {
    key: crypto.randomUUID(),
    name: "Tower Scene",
    files: () => importZipExample("./TowerExample.zip"),
  },
  {
    key: crypto.randomUUID(),
    name: "Terrain and Trees",
    files: () => importZipExample("./TreesAndTerrainExample.zip"),
  },
];
