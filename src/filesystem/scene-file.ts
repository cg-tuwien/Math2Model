import { z } from "zod";
import { makeFilePath } from "./reactive-files";

// Warning:
// Changes to this file mean that you could be breaking someone's scene out there!
// So make sure to keep compatibility in mind.

export const SceneFileName = makeFilePath("scene.json");

export const SceneFileSchemaUrl = "http://virtual/scene-schema.json" as const;

export const MaterialParameterSchema = z.object({
  color: z.tuple([z.number(), z.number(), z.number()]),
  roughness: z.number(),
  metallic: z.number(),
  emissive: z.tuple([z.number(), z.number(), z.number()]),
});

export const ModelSchema = z.object({
  type: z.literal("model"),
  id: z.string(),
  name: z.string(),
  position: z.tuple([z.number(), z.number(), z.number()]),
  rotation: z.tuple([z.number(), z.number(), z.number()]),
  scale: z.number(),
  parametricShader: z.string(),
  material: MaterialParameterSchema,
  instanceCount: z.number().catch(() => 1),
});

export type SerializedModel = z.infer<typeof ModelSchema>;

export const SceneFileSchema = z.object({
  $schema: z.literal(SceneFileSchemaUrl),
  models: z.array(ModelSchema),
});

export type SerializedScene = z.infer<typeof SceneFileSchema>;

/**
 * @throws Error if the scene cannot be serialized.
 */
export function serializeScene(scene: SerializedScene, pretty = false) {
  const result = SceneFileSchema.parse(scene);
  return JSON.stringify(result, null, pretty ? 2 : undefined);
}

/**
 * @throws Error if the scene is invalid.
 */
export function deserializeScene(scene: string): SerializedScene {
  return SceneFileSchema.parse(JSON.parse(scene));
}
