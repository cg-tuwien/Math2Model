import type { Component } from "vue";
import type { Nodes } from "../nodes-list";

export interface UINode {
  name: string;
  type: "ARRANGE" | "CONSTANT" | "CALCULATE" | "SHAPE" | "APPLY" | "MANIPULATE";
  prefix: string;
  image: Component;
  get: () => Nodes;
  create: (uiNode: UINode) => void;
  draggable: boolean;
}
