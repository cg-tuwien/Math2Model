import type { FilePath } from "@/filesystem/reactive-files";
import type { Nodes } from "@/components/visual-programming/CodeGraph.vue";
import type { Component } from "vue";

export interface UINode {
  name: string;
  type: "SHAPE" | "APPLY" | "MANIPULATE";
  prefix: string;
  image: Component;
  get: () => Nodes;
  create: (uiNode: UINode) => void;
  draggable: boolean;
}
