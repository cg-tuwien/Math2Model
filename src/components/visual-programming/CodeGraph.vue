<script setup lang="ts">
import { AreaPlugin } from "rete-area-plugin";
import {
  VuePlugin,
  Presets as VuePresets,
  type VueArea2D,
} from "rete-vue-plugin";
import { NodeEditor, ClassicPreset, type GetSchemes } from "rete";
import { type DeepReadonly, onMounted, ref } from "vue";
import { AreaExtensions } from "rete-area-plugin";
import {
  BidirectFlow,
  ConnectionPlugin,
  Presets as ConnectionPresets,
} from "rete-connection-plugin";
import {
  type ContextMenuExtra,
  ContextMenuPlugin,
  Presets as ContextMenuPresets,
} from "rete-context-menu-plugin";
import {
  NumberNode,
  reteSocket as socket,
  MathOpNode,
  Vector2Node,
  Seperate2Node,
  Join2Node,
  Vector3Node,
} from "@/vpnodes/nodes";
import { DataflowEngine } from "rete-engine";
import { DockPlugin, DockPresets } from "rete-dock-plugin";
import { structures } from "rete-structures";
import { root } from "postcss";

const container = ref<HTMLElement | null>(null);

onMounted(() => {
  if (container.value === null) return;

  createEditor();
});

type Nodes =
  | NumberNode
  | MathOpNode
  | Vector2Node
  | Seperate2Node
  | Join2Node
  | Vector3Node;
class Connection<
  A extends Nodes,
  B extends Nodes,
> extends ClassicPreset.Connection<A, B> {}

type Conns =
  | Connection<NumberNode, MathOpNode>
  | Connection<MathOpNode, MathOpNode>
  | Connection<Vector2Node, Seperate2Node>
  | Connection<Seperate2Node, MathOpNode>
  | Connection<Seperate2Node, Join2Node>
  | Connection<Join2Node, Seperate2Node>
  | Connection<NumberNode, Join2Node>
  | Connection<Vector3Node, Seperate2Node>;

type Schemes = GetSchemes<Nodes, Conns>;

const editor = new NodeEditor<Schemes>();
const engine = new DataflowEngine<Schemes>();

async function createEditor() {
  type AreaExtra = VueArea2D<Schemes> | ContextMenuExtra;

  const area = new AreaPlugin<Schemes, AreaExtra>(
    container.value ?? new HTMLElement(),
  );
  AreaExtensions.selectableNodes(area, AreaExtensions.selector(), {
    accumulating: AreaExtensions.accumulateOnCtrl(),
  });
  AreaExtensions.snapGrid(area, {
    size: 20,
  });

  const contextMenu = new ContextMenuPlugin<Schemes>({
    items: ContextMenuPresets.classic.setup([
      ["Number", () => new NumberNode()],
      ["Add", () => new MathOpNode("+", (c) => area.update("control", c.id))],
      [
        "Subtract",
        () => new MathOpNode("-", (c) => area.update("control", c.id)),
      ],
      [
        "Multiply",
        () => new MathOpNode("*", (c) => area.update("control", c.id)),
      ],
      [
        "Divide",
        () => new MathOpNode("/", (c) => area.update("control", c.id)),
      ],
      [
        "Modulo",
        () => new MathOpNode("%", (c) => area.update("control", c.id)),
      ],
      ["Vector2", () => new Vector2Node((c) => area.update("control", c.id))],
      ["Vector3", () => new Vector3Node((c) => area.update("control", c.id))],
      [
        "Seperate2",
        () => new Seperate2Node((c) => area.update("control", c.id)),
      ],
      ["Join2", () => new Join2Node((c) => area.update("control", c.id))],
    ]),
  });

  const dock = new DockPlugin<Schemes>();

  dock.addPreset(DockPresets.classic.setup({ area, size: 30, scale: 0.7 }));
  //
  //

  area.use(contextMenu);

  const connection = new ConnectionPlugin<Schemes, AreaExtra>();

  connection.addPreset(ConnectionPresets.classic.setup());

  const render = new VuePlugin<Schemes, AreaExtra>();

  render.addPreset(VuePresets.classic.setup());
  render.addPreset(VuePresets.contextMenu.setup());

  editor.use(area);
  editor.use(engine);
  area.use(connection);
  area.use(render);
  area.use(dock);

  dock.add(() => new NumberNode());
  dock.add(() => new MathOpNode("+", (c) => area.update("control", c.id)));
  dock.add(() => new MathOpNode("-", (c) => area.update("control", c.id)));
  dock.add(() => new MathOpNode("*", (c) => area.update("control", c.id)));
  dock.add(() => new MathOpNode("/", (c) => area.update("control", c.id)));
  dock.add(() => new MathOpNode("%", (c) => area.update("control", c.id)));
  dock.add(() => new Vector2Node((c) => area.update("control", c.id)));
  dock.add(() => new Vector3Node((c) => area.update("control", c.id)));
  dock.add(() => new Seperate2Node((c) => area.update("control", c.id)));
  dock.add(() => new Join2Node((c) => area.update("control", c.id)));

  editor.addPipe((context) => {
    if (["connectioncreated", "connectionremoved"].includes(context.type)) {
      engine.reset();

      editor
        .getNodes()
        .filter((n) => n instanceof MathOpNode)
        .forEach((n) => engine.fetch(n.id));
      logCode();
    }

    return context;
  });
}

async function getNodesCode(node: Nodes) {
  const nodeData = await engine.fetch(node.id);
  let fullCode = "";
  if (node instanceof Seperate2Node) {
    fullCode += "\t" + nodeData.x.code + "\n";
    fullCode += "\t" + nodeData.y.code + "\n";
  } else {
    fullCode += "\t" + nodeData.value.code + "\n";
  }

  return fullCode;
}

async function logCode() {
  const graph = structures(editor);
  const rootNodes = graph.roots().nodes();
  const leafNodes = graph.leaves().nodes();
  const allNodes = graph.nodes();
  let visited = [];
  let fullCode = "{\n";
  for (let node of rootNodes) {
    visited.push(node.id);
    fullCode += await getNodesCode(node);
  }

  let ind = 0;
  let currentNode = rootNodes[ind];
  let rootsDone = false;
  while (!leafNodes.includes(currentNode)) {
    const outgoers = graph.outgoers(currentNode.id).nodes();
    if (!visited.includes(currentNode.id)) {
      fullCode += await getNodesCode(currentNode);
      visited.push(currentNode.id);
    }

    for (let node of outgoers) {
      if (visited.includes(node.id)) continue;
      visited.push(node.id);

      fullCode += await getNodesCode(node);
    }

    if (!rootsDone) {
      ind += 1;
      if (ind >= rootNodes.length) {
        ind = 0;
        rootsDone = true;
        currentNode = allNodes[ind];
      } else {
        currentNode = rootNodes[ind];
      }
    } else {
      ind += 1;
      if (ind >= allNodes.length) {
        break;
      }
      currentNode = allNodes[ind];
    }
  }

  for (let node of leafNodes) {
    if (visited.includes(node.id)) continue;
    visited.push(node.id);

    fullCode += await getNodesCode(node);
  }
  /**const rest = graph.nodes();
  for (let node of rest) {
    if (rootNodes.includes(node)) continue;
    const nodeData = await engine.fetch(node.id);

    if (node instanceof Seperate2Node) {
      fullCode += "\t" + nodeData.x.code + "\n";
      fullCode += "\t" + nodeData.y.code + "\n";
      continue;
    }
    fullCode += "\t" + nodeData.value.code + "\n";
  }*/

  fullCode += "}";
  console.log(fullCode);
}
</script>

<template>
  <div class="rete" ref="container"></div>
</template>

<style scoped></style>
