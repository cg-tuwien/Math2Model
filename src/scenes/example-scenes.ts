import type { ImportFilesList } from "@/stores/fs-store";
import { DefaultScene, fileName } from "./default-scene";

type ExampleProject = {
  key: string;
  name: string;
  files: () => Promise<ImportFilesList>;
};

const temple = import.meta.glob("./example-scene/temple/*", {
  query: "?url",
  import: "default",
  eager: true,
});

const tower = import.meta.glob("./example-scene/tower/*", {
  query: "?url",
  import: "default",
  eager: true,
});

const treesAndTerrain = import.meta.glob(
  "./example-scene/trees-and-terrain/*",
  {
    query: "?url",
    import: "default",
    eager: true,
  }
);

async function toLazyProject(
  files: Record<string, unknown>
): Promise<ImportFilesList> {
  return {
    type: "in-memory",
    value: await Promise.all(
      Object.entries(files).map(async ([name, url]) => {
        return {
          name: fileName(name),
          value: await fetch(url as string).then((v) => v.arrayBuffer()),
        };
      })
    ),
  };
}

export const ExampleProjects: ExampleProject[] = [
  {
    key: crypto.randomUUID(),
    name: "Morph Heart to Sphere",
    files: async () => DefaultScene,
  },
  {
    key: crypto.randomUUID(),
    name: "Parametric Temple",
    files: () => toLazyProject(temple),
  },
  {
    key: crypto.randomUUID(),
    name: "Parametric Tower",
    files: () => toLazyProject(tower),
  },
  {
    key: crypto.randomUUID(),
    name: "Terrain and Trees",
    files: () => toLazyProject(treesAndTerrain),
  },
];
