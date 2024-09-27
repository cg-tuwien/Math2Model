import type { FilePath } from "@/filesystem/reactive-files";
import type { Nodes } from "@/components/visual-programming/CodeGraph.vue";

export interface UINode {
  name: string;
  prefix: string;
  image: string;
  create: () => Nodes;
}
