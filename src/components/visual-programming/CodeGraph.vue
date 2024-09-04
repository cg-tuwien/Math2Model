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
  ReturnNode,
  VariableOutNode,
  FunctionCallNode,
  InitializeNode,
  VariableInNode,
  NULL,
  NothingNode,
} from "@/vpnodes/nodes";
import { DataflowEngine } from "rete-engine";
import { DockPlugin, DockPresets } from "rete-dock-plugin";
import { structures } from "rete-structures";
import { root } from "postcss";
import { ScopesPlugin, Presets as ScopesPresets } from "rete-scopes-plugin";
import { BlockNode, ConditionNode, ScopeNode } from "@/vpnodes/blocks";
import type { Structures } from "rete-structures/_types/types";
import { get } from "@vueuse/core";
import {
  AutoArrangePlugin,
  Presets as ArrangePresets,
} from "rete-auto-arrange-plugin";
import { vec2 } from "webgpu-matrix";
import type { NodeFactory } from "rete-context-menu-plugin/_types/presets/classic/types";

const emit = defineEmits<{ update: [code: () => string] }>();

const container = ref<HTMLElement | null>(null);

onMounted(() => {
  if (container.value === null) return;

  createEditor();
});

export type Nodes =
  | NumberNode
  | MathOpNode
  | VectorNode
  | SeparateNode
  | JoinNode
  | ConditionNode
  | ScopeNode
  | ReturnNode
  | FunctionCallNode
  | VariableOutNode
  | VariableInNode
  | InitializeNode
  | NothingNode;

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
  | Connection<ConditionNode, MathOpNode>
  | Connection<ReturnNode, VectorNode>
  | Connection<NumberNode, FunctionCallNode>
  | Connection<InitializeNode, ScopeNode>;

type Schemes = GetSchemes<Nodes, Conns>;

const editor = new NodeEditor<Schemes>();
const engine = new DataflowEngine<Schemes>();
const arrange = new AutoArrangePlugin<Schemes>();

let shouldUpdate = true;

async function newConditionNode(
  scope1: string,
  scope2: string,
  area: AreaPlugin<Schemes, AreaExtra>,
  operator: "==" | "!=" | ">" | "<" | ">=" | "<=",
): Promise<Nodes> {
  shouldUpdate = false;
  const sc1 = new ScopeNode(
    scope1,
    (n) => area.update("node", n.id),
    (n) => editor.addNode(n),
    (n) => editor.removeNode(n.id),
  );
  const sc2 = new ScopeNode(
    scope2,
    (n) => area.update("node", n.id),
    (n) => editor.addNode(n),
    (n) => editor.removeNode(n.id),
  );

  await editor.addNode(sc1);
  await editor.addNode(sc2);

  const c = new ConditionNode(operator, operator);

  await editor.addNode(c);
  editor
    .addConnection(new ClassicPreset.Connection(c, "true", sc1, "context"))
    .catch((e) => console.log(e));
  editor
    .addConnection(new ClassicPreset.Connection(c, "false", sc2, "context"))
    .catch((e) => console.log(e));

  await arrange.layout();

  shouldUpdate = true;

  return new NothingNode();
}
type AreaExtra = VueArea2D<Schemes> | ContextMenuExtra;

const endNode = new ReturnNode("vec3f(input2.x, 0, input2.y)", "Output Vertex");
const startNode = new VariableOutNode(vec2.create(1, 1), "input2;", "input2");
const piNode = new VariableOutNode(
  3.14159265359,
  "var PI = 3.14159265359;",
  "PI",
);
const halfPiNode = new VariableOutNode(
  3.14159265359 / 2,
  "var HALF_PI = 3.14159265359 / 2.0;",
  "HALF_PI",
);
const twoPiNode = new VariableOutNode(
  3.14159265359 * 2,
  "var TWO_PI = 3.14159265359 * 2.0;",
  "TWO_PI",
);

async function createEditor() {
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
      [
        "Refresh",
        () => {
          arrange.layout();
          if (editor.getNode(NULL.id)) editor.removeNode(NULL.id);
          return NULL;
        },
      ],
      ["Initialize", () => new InitializeNode()],
      [
        "Math",
        [
          ["Number", () => new NumberNode()],
          [
            "Add",
            () => new MathOpNode("+", (c) => area.update("control", c.id)),
          ],
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
          ["Sin", () => new FunctionCallNode("sin", 1)],
          ["Cos", () => new FunctionCallNode("cos", 1)],
          ["Tan", () => new FunctionCallNode("tan", 1)],
          ["Sqrt", () => new FunctionCallNode("sqrt", 1)],
          ["Abs", () => new FunctionCallNode("abs", 1)],
        ],
      ],
      [
        "Vectors",
        [
          [
            "Vector2",
            () => new VectorNode(2, (c) => area.update("control", c.id)),
          ],
          [
            "Vector3",
            () => new VectorNode(3, (c) => area.update("control", c.id)),
          ],
          [
            "Vector4",
            () => new VectorNode(4, (c) => area.update("control", c.id)),
          ],
          [
            "Separate",
            () => new SeparateNode((c) => area.update("control", c.id)),
          ],
          ["Join", () => new JoinNode((n) => area.update("node", n.id))],
        ],
      ],
      [
        "Logic",
        [
          ["Equals", () => newConditionNode("True", "False", area, "==")],
          ["Not Equals", () => newConditionNode("True", "False", area, "!=")],
          ["Lt", () => newConditionNode("True", "False", area, "<")],
          ["Le", () => newConditionNode("True", "False", area, "<=")],
          ["Gt", () => newConditionNode("True", "False", area, ">")],
          ["Ge", () => newConditionNode("True", "False", area, ">=")],
        ],
      ],
    ]),
  });

  const dock = new DockPlugin<Schemes>();

  const scopes = new ScopesPlugin<Schemes>();

  dock.addPreset(DockPresets.classic.setup({ area, size: 30, scale: 0.7 }));
  //
  //

  arrange.addPreset(ArrangePresets.classic.setup());

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
  area.use(arrange);

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
        area.update("node", n.id);
      }),
  );
  dock.add(() => new JoinNode((n) => area.update("node", n.id)));

  await editor.addNode(startNode);
  await editor.addNode(piNode);
  await editor.addNode(halfPiNode);
  await editor.addNode(twoPiNode);
  await editor.addNode(endNode);
  //await editor.addConnection(
  //  new ClassicPreset.Connection(startNode, "out", endNode, "returnIn"),
  //);
  await arrange.layout();

  editor.addPipe((context) => {
    if (
      [
        "connectioncreated",
        "connectionremoved",
        "nodecreated",
        "noderemoved",
        "scopeupdated",
      ].includes(context.type)
    ) {
      engine.reset();
      if (!shouldUpdate) return;
      // arrange.layout();
      editor
        .getNodes()
        .filter((n) => !(n instanceof BlockNode))
        .forEach((n) => n.updateSize(area));
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
    let trueCode = "";
    let falseCode = "";
    // fullCode += "\t" + nodeData.true.code + "\n";

    const blockContent = graph.outgoers(node.id).nodes();
    for (let content of blockContent) {
      const scopeIncomers = graph.incomers(content.id).nodes();
      let incomersCode = "";
      if (scopeIncomers.length > 0) {
        incomersCode += await orderedCode(scopeIncomers, visited);
      }
      fullCode += incomersCode;
    }
    for (let content of blockContent) {
      if (content.label != "True") continue;
      visited.push(content.id);
      const scopeChildren = editor
        .getNodes()
        .filter((n) => n.parent === content.id);
      if (scopeChildren.length > 0) {
        trueCode += await getScopeCode(scopeChildren, visited);
      }

      fullCode +=
        "\t" +
        nodeData.true.code +
        "\n" +
        trueCode +
        (await getNodesCode(content, visited, graph));
    }

    // fullCode += "\t" + nodeData.false.code + "\n";
    for (let content of blockContent) {
      if (content.label != "False") continue;
      visited.push(content.id);
      const scopeChildren = editor
        .getNodes()
        .filter((n) => n.parent === content.id);
      if (scopeChildren.length > 0) {
        falseCode += await getScopeCode(scopeChildren, visited);
      }

      fullCode +=
        "\t" +
        nodeData.false.code +
        "\n" +
        falseCode +
        (await getNodesCode(content, visited, graph));
    }
  } else {
    fullCode += "\t" + nodeData.value.code + "\n";
  }

  return fullCode;
}

async function getScopeCode(scopeChildren: Nodes[], visited: string[]) {
  if (scopeChildren.length <= 0) return;

  return orderedCode(scopeChildren, visited, "\t");
}

async function logCode() {
  const graph = structures(editor);
  const allNodes = graph
    .nodes()
    .filter((n) => n.id !== endNode.id)
    .filter((n) => !n.parent);

  if (allNodes.length <= 0) return;

  let visited: string[] = [startNode.id, endNode.id];
  let fullCode =
    "fn evaluateImage(input2: vec2f) -> vec3f {\n" +
    (await orderedCode(allNodes, visited));

  fullCode += (await getNodesCode(endNode, [], graph)) + "}";
  console.log(fullCode);
  emit("update", () => fullCode);
}

async function orderedCode(
  allNodes: Nodes[],
  visited: string[],
  indent: string = "",
) {
  if (allNodes.length <= 0) return "";
  const graph = structures(editor);
  let fullCode = "";
  let nodeQueue = allNodes.filter(
    (n) => n.id !== endNode.id && n.id !== startNode.id,
  );

  while (nodeQueue.length > 0) {
    const node = nodeQueue.shift();
    if (!node) break;
    if (visited.includes(node.id)) continue;
    if (node instanceof NothingNode) {
      await editor.removeNode(node.id);
      continue;
    }
    const incomers = graph.incomers(node.id).nodes();
    if (incomers.some((inc) => !visited.includes(inc.id))) {
      nodeQueue.push(node);
      continue;
    }

    visited.push(node.id);
    fullCode += indent + (await getNodesCode(node, visited, graph));
  }

  return fullCode;
}
</script>

<template>
  <div class="rete" ref="container"></div>
</template>

<style scoped></style>
