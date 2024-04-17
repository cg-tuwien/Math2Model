import { z } from "zod";

export const SceneFileSchemaUrl = "http://virtual/scene-schema.json" as const;

export const ShaderSchema = z.object({
  type: z.literal("parametric-shader"),
  main: z.string(),
});

export type Shader = z.infer<typeof ShaderSchema>;

export const ActorSchema = z.object({
  type: z.literal("model"),
  id: z.string(),
  name: z.string(),
  position: z.tuple([z.number(), z.number(), z.number()]),
  rotation: z.tuple([z.number(), z.number(), z.number()]),
  scale: z.tuple([z.number(), z.number(), z.number()]),
  shader: z.string(),
});

export type Actor = z.infer<typeof ActorSchema>;

export const SceneFileSchema = z
  .object({
    $schema: z.literal(SceneFileSchemaUrl),
    models: z.array(ActorSchema),
    shaders: z.record(ShaderSchema),
  })
  .superRefine((value, ctx) => {
    value.models.forEach((model, index) => {
      if (value.shaders[model.shader] === undefined) {
        ctx.addIssue({
          code: z.ZodIssueCode.custom,
          message: `shader ${model.shader} is not defined`,
          path: ["models", index, "shader"],
        });
      }
    });
  });

export type SceneFile = z.infer<typeof SceneFileSchema>;
