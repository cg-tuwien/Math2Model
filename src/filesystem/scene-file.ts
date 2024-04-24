import { z } from "zod";

export const SceneFileSchemaUrl = "http://virtual/scene-schema.json" as const;

export const ModelSchema = z.object({
  type: z.literal("model"),
  id: z.string(),
  name: z.string(),
  position: z.tuple([z.number(), z.number(), z.number()]),
  rotation: z.tuple([z.number(), z.number(), z.number(), z.number()]),
  scale: z.number(),
  parametricShader: z.string(),
  fragmentShader: z.string(),
});

export type SerializedModel = z.infer<typeof ModelSchema>;

// TODO: Hook this up in monaco-setup.ts
export const SceneFileSchema = z.object({
  $schema: z.literal(SceneFileSchemaUrl),
  models: z.array(ModelSchema),
});

export type SerializedScene = z.infer<typeof SceneFileSchema>;

export function serializeScene(scene: SerializedScene) {
  return JSON.stringify(scene);
}

export function deserializeScene(scene: string): SerializedScene | null {
  try {
    return SceneFileSchema.parse(JSON.parse(scene));
  } catch (e) {
    console.error(e);
    return null;
  }
}
