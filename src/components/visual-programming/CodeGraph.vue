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
  VectorNode,
  SeparateNode,
  JoinNode,
  VPNode,
} from "@/vpnodes/nodes";
import { DataflowEngine } from "rete-engine";
import { DockPlugin, DockPresets } from "rete-dock-plugin";
import { structures } from "rete-structures";
import { root } from "postcss";
import { ScopesPlugin, Presets as ScopesPresets } from "rete-scopes-plugin";
import { BlockNode, ConditionNode, ScopeNode } from "@/vpnodes/blocks";
import type { Structures } from "rete-structures/_types/types";
import { get } from "@vueuse/core";

const container = ref<HTMLElement | null>(null);

onMounted(() => {
  if (container.value === null) return;

  createEditor();
});

type Nodes =
  | NumberNode
  | MathOpNode
  | VectorNode
  | SeparateNode
  | JoinNode
  | ConditionNode
  | ScopeNode;
class Connection<
  A extends Nodes,
  B extends Nodes,
> extends ClassicPreset.Connection<A, B> {}

type Conns =
  | Connection<NumberNode, MathOpNode>
  | Connection<MathOpNode, MathOpNode>
  | Connection<VectorNode, SeparateNode>
  | Connection<SeparateNode, MathOpNode>
  | Connection<SeparateNode, JoinNode>
  | Connection<JoinNode, SeparateNode>
  | Connection<NumberNode, JoinNode>
  | Connection<ConditionNode, JoinNode>
  | Connection<ConditionNode, NumberNode>
  | Connection<ConditionNode, SeparateNode>
  | Connection<ConditionNode, MathOpNode>;

type Schemes = GetSchemes<Nodes, Conns>;

const editor = new NodeEditor<Schemes>();
const engine = new DataflowEngine<Schemes>();

function newConditionNode(
  scope1: string,
  scope2: string,
  area: AreaPlugin<Schemes>,
): BlockNode {
  const sc1 = new ScopeNode(scope1, (n) => area.update("node", n.id));
  const sc2 = new ScopeNode(scope2, (n) => area.update("node", n.id));

  editor.addNode(sc1);
  editor.addNode(sc2);

  return new BlockNode("Condition");
}

async function createEditor() {
  type AreaExtra = VueArea2D<Schemes> | ContextMenuExtra;

  const area = new AreaPlugin<Schemes, AreaExtra>(
    container.value ?? new HTMLElement(),
  );
  AreaExtensions.selectableNodes(area, AreaExtensions.selector(), {
    accumulating: AreaExtensions.accumulateOnCtrl(),
  });
  //AreaExtensions.snapGrid(area, {
  //  size: 20,
  //});

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
      ["Vector2", () => new VectorNode(2, (c) => area.update("control", c.id))],
      ["Vector3", () => new VectorNode(3, (c) => area.update("control", c.id))],
      ["Vector4", () => new VectorNode(4, (c) => area.update("control", c.id))],
      ["Separate", () => new SeparateNode((c) => area.update("control", c.id))],
      ["Join", () => new JoinNode((n) => area.update("node", n.id))],
      ["Equals", () => new ConditionNode("Equals", "==")],
    ]),
  });

  const dock = new DockPlugin<Schemes>();

  const scopes = new ScopesPlugin<Schemes>();

  dock.addPreset(DockPresets.classic.setup({ area, size: 30, scale: 0.7 }));
  //
  //

  area.use(contextMenu);

  const connection = new ConnectionPlugin<Schemes, AreaExtra>();

  connection.addPreset(ConnectionPresets.classic.setup());

  const render = new VuePlugin<Schemes, AreaExtra>();

  render.addPreset(VuePresets.classic.setup());
  render.addPreset(VuePresets.contextMenu.setup());
  scopes.addPreset(ScopesPresets.classic.setup());

  editor.use(area);
  editor.use(engine);
  area.use(connection);
  area.use(render);
  area.use(dock);
  area.use(scopes);

  dock.add(() => new NumberNode((n) => area.update("node", n.id)));
  dock.add(
    () =>
      new MathOpNode("+", (n, c) => {
        area.update("control", c.id);
        area.update("node", n.id);
      }),
  );
  dock.add(
    () =>
      new MathOpNode("-", (n, c) => {
        area.update("control", c.id);
        area.update("node", n.id);
      }),
  );
  dock.add(
    () =>
      new MathOpNode("*", (n, c) => {
        area.update("control", c.id);
        area.update("node", n.id);
      }),
  );
  dock.add(
    () =>
      new MathOpNode("/", (n, c) => {
        area.update("control", c.id);
        area.update("node", n.id);
      }),
  );
  dock.add(
    () =>
      new MathOpNode("%", (n, c) => {
        area.update("control", c.id);
        area.update("node", n.id);
      }),
  );
  dock.add(() => new VectorNode(2, (n) => area.update("node", n.id)));
  dock.add(() => new VectorNode(3, (n) => area.update("node", n.id)));
  dock.add(() => new VectorNode(4, (n) => area.update("node", n.id)));
  dock.add(
    () =>
      new SeparateNode((n) => {
        console.log("Separate.Update(1)");
        area.update("node", n.id);
        console.log("Separate.Update(2)");
      }),
  );
  dock.add(() => new JoinNode((n) => area.update("node", n.id)));

  editor.addPipe((context) => {
    if (
      [
        "connectioncreated",
        "connectionremoved",
        "nodecreated",
        "noderemoved",
      ].includes(context.type)
    ) {
      engine.reset();
      logCode();
    }

    return context;
  });
}

async function getNodesCode(
  node: Nodes,
  visited: string[],
  graph: Structures<Nodes, Conns>,
) {
  const nodeData = await engine.fetch(node.id);
  let fullCode = "";
  if (node instanceof SeparateNode) {
    fullCode += "\t" + nodeData.x.code + "\n";
    fullCode += "\t" + nodeData.y.code + "\n";
    fullCode += nodeData.z.code !== "" ? "\t" + nodeData.z.code + "\n" : "";
    fullCode += nodeData.w.code !== "" ? "\t" + nodeData.w.code + "\n" : "";
  } else if (node instanceof ConditionNode) {
    fullCode += "\t" + nodeData.true.code + "\n";
    const blockContent = graph.outgoers(node.id).nodes();
    for (let content of blockContent) {
      visited.push(content.id);
      fullCode += "\t" + (await getNodesCode(content, visited, graph));
    }
    fullCode += "}\n";
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

  if (allNodes.length <= 0) return;

  let visited = [];
  let fullCode = "{\n";
  for (let node of rootNodes) {
    visited.push(node.id);
    fullCode += await getNodesCode(node, visited, graph);
  }

  let ind = 0;
  let currentNode = rootNodes[ind];
  let rootsDone = false;
  while (!leafNodes.includes(currentNode)) {
    const outgoers = graph.outgoers(currentNode.id).nodes();
    if (!visited.includes(currentNode.id)) {
      fullCode += await getNodesCode(currentNode, visited, graph);
      visited.push(currentNode.id);
    }

    for (let node of outgoers) {
      if (visited.includes(node.id)) continue;
      visited.push(node.id);

      fullCode += await getNodesCode(node, visited, graph);
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

    fullCode += await getNodesCode(node, visited, graph);
  }

  fullCode += "}";
  console.log(fullCode);
}
</script>

<template>
  <div class="rete" ref="container"></div>
</template>

<style scoped></style>
