import type { SerializedNode } from "@/vpnodes/serialization/node";

export class SerializedGraph {
  public nodes: SerializedNode[] = [];

  constructor(nodes?: SerializedNode[]) {
    this.nodes = nodes ?? this.nodes;
  }

  add(node: SerializedNode) {
    this.nodes.push(node);
  }

  getNode(uuid: string) {
    for (let node of this.nodes) {
      if (node.uuid === uuid) return node;
    }
    return undefined;
  }

  toJSON() {
    return JSON.stringify({ graph: this.nodes }, null, 2);
  }
}

export function graphFromJSON(json: string) {
  return JSON.parse(json);
}
