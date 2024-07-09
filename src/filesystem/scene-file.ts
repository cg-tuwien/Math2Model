import { z } from "zod";
import { makeFilePath } from "./reactive-files";

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
});

export type SerializedModel = z.infer<typeof ModelSchema>;

export const SceneFileSchema = z.object({
  $schema: z.literal(SceneFileSchemaUrl),
  models: z.array(ModelSchema),
});

export type SerializedScene = z.infer<typeof SceneFileSchema>;

export function serializeScene(scene: SerializedScene, pretty = false) {
  const result = SceneFileSchema.safeParse(scene);
  if (result.success) {
    return JSON.stringify(result.data, null, pretty ? 2 : undefined);
  } else {
    console.error(result.error);
    return null;
  }
}

export function deserializeScene(scene: string): SerializedScene | null {
  try {
    return SceneFileSchema.parse(JSON.parse(scene));
  } catch (e) {
    console.error(e);
    return null;
  }
}
