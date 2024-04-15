import { z } from "zod";

export const CacheFileSchemaUrl = "http://virtual/cache-schema.json" as const;

export const CacheFileSchema = z.object({
  $schema: z.literal(CacheFileSchemaUrl),
  camera: z
    .discriminatedUnion("type", [
      z.object({
        type: z.literal("arc-rotate-camera"),
        target: z.tuple([z.number(), z.number(), z.number()]),
        alpha: z.number(),
        beta: z.number(),
        radius: z.number(),
      }),
    ])
    .optional(),
});

export type CacheFile = z.infer<typeof CacheFileSchema>;

const CacheKey = "cache";

export function readCacheFile(): CacheFile | null {
  let content = localStorage.getItem(CacheKey);
  if (content === null) {
    return null;
  }
  try {
    return CacheFileSchema.parse(JSON.parse(content));
  } catch (e) {
    return null;
  }
}

export function writeCacheFile(cache: CacheFile) {
  console.log("Writing cache", cache);
  localStorage.setItem(CacheKey, JSON.stringify(cache));
}
