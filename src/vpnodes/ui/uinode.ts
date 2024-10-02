import type { FilePath } from "@/filesystem/reactive-files";
import type { Nodes } from "@/components/visual-programming/CodeGraph.vue";

export interface UINode {
  name: string;
  type: "SHAPE" | "APPLY" | "MANIPULATE";
  prefix: string;
  image: string;
  create: () => Nodes;
}
