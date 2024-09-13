<script setup lang="ts">
import { type Area, AreaPlugin } from "rete-area-plugin";
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
  reteSocket as socket,
  VPNode,
  ReturnNode,
  VariableOutNode,
  FunctionCallNode,
  InitializeNode,
  VariableInNode,
  NothingNode,
} from "@/vpnodes/basic/nodes";
import { DataflowEngine } from "rete-engine";
import { structures } from "rete-structures";
import { root } from "postcss";
import { ScopesPlugin, Presets as ScopesPresets } from "rete-scopes-plugin";
import {
  BlockNode,
  ConditionNode,
  LogicScopeNode,
} from "@/vpnodes/basic/logic";
import type { Structures } from "rete-structures/_types/types";
import { get } from "@vueuse/core";
import {
  AutoArrangePlugin,
  Presets as ArrangePresets,
} from "rete-auto-arrange-plugin";
import { vec2 } from "webgpu-matrix";
import { MathOpNode, NumberNode } from "@/vpnodes/basic/math";
import { JoinNode, SeparateNode, VectorNode } from "@/vpnodes/basic/vector";
import { DropdownControl } from "@/vpnodes/controls/dropdown";
import DropdownComponent from "@/vpnodes/components/DropdownComponent.vue";
import {
  CallCustomFunctionNode,
  CustomFunctionNode,
  FunctionScopeNode,
} from "@/vpnodes/basic/functions";
import { graphFromJSON, SerializedGraph } from "@/vpnodes/serialization/graph";
import { SerializedNode, toSerializedNode } from "@/vpnodes/serialization/node";
import { type FilePath, makeFilePath } from "@/filesystem/reactive-files";
import type { VirtualModelState } from "@/scenes/scene-state";
import type { SelectMixedOption } from "naive-ui/es/select/src/interface";

const emit = defineEmits<{
  update: [code: () => string];
  save: [fileName: FilePath, content: string];
  load: [fileName: FilePath, add: boolean];
}>();

const props = defineProps<{
  graphs: SelectMixedOption[];
}>();

defineExpose({
  replaceOrAddDeserialize,
});

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
  | LogicScopeNode
  | ReturnNode
  | FunctionCallNode
  | VariableOutNode
  | VariableInNode
  | InitializeNode
  | NothingNode
  | CustomFunctionNode
  | CallCustomFunctionNode
  | FunctionScopeNode;

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
  | Connection<InitializeNode, LogicScopeNode>;

type Schemes = GetSchemes<Nodes, Conns>;

const editor = new NodeEditor<Schemes>();
const engine = new DataflowEngine<Schemes>();
const arrange = new AutoArrangePlugin<Schemes>();

let shouldUpdate = true;

let saving = ref<boolean>(false);
let loading = ref<boolean>(false);
let saveName = ref<string>("new-graph");
let loadName = ref<string>("no file selected");

async function newFunctionNode(area: AreaPlugin<Schemes, AreaExtra>) {
  shouldUpdate = false;
  const cfn = new CustomFunctionNode(
    (n) => area.update("node", n.id),
    (n) => editor.addNode(n),
    (n) => editor.removeNode(n.id),
    (nA, kA, nB, kB) =>
      editor.addConnection(new ClassicPreset.Connection(nA, kA, nB, kB)),
  );
  await editor.addNode(cfn);
  cfn.functionScope.retNode.parent = cfn.functionScope.id;
  await editor.addNode(cfn.functionScope.retNode);
  shouldUpdate = true;
  return new NothingNode();
}

async function newConditionNode(
  scope1: string,
  scope2: string,
  area: AreaPlugin<Schemes, AreaExtra>,
  operator: "==" | "!=" | ">" | "<" | ">=" | "<=",
): Promise<Nodes> {
  shouldUpdate = false;
  const sc1 = new LogicScopeNode(
    scope1,
    (n) => area.update("node", n.id),
    (n) => editor.addNode(n),
    (n) => editor.removeNode(n.id),
  );
  const sc2 = new LogicScopeNode(
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
const startNode = new VariableOutNode(vec2.create(1, 1), "", "input2");
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

let area: AreaPlugin<Schemes, AreaExtra>;
async function createEditor() {
  area = new AreaPlugin<Schemes, AreaExtra>(
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
        "Rearrange",
        () => {
          arrange.layout();
          return new NothingNode();
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
        "Functions",
        [
          ["Sin", () => new FunctionCallNode("sin", 1)],
          ["Cos", () => new FunctionCallNode("cos", 1)],
          ["Tan", () => new FunctionCallNode("tan", 1)],
          ["Sqrt", () => new FunctionCallNode("sqrt", 1)],
          ["Abs", () => new FunctionCallNode("abs", 1)],
          ["Exp", () => new FunctionCallNode("exp", 1)],
          ["Round", () => new FunctionCallNode("round", 1)],
          ["Pow", () => new FunctionCallNode("pow", 2)],
          ["Mix", () => new FunctionCallNode("mix", 3)],
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
      [
        "Custom",
        [
          ["New Function", () => newFunctionNode(area)],
          [
            "Call Custom Function",
            () => new CallCustomFunctionNode((n) => area.update("node", n.id)),
          ],
        ],
      ],
    ]),
  });

  const scopes = new ScopesPlugin<Schemes>();

  arrange.addPreset(ArrangePresets.classic.setup());

  area.use(contextMenu);

  const connection = new ConnectionPlugin<Schemes, AreaExtra>();

  connection.addPreset(ConnectionPresets.classic.setup());

  const render = new VuePlugin<Schemes, AreaExtra>();

  render.addPreset(
    VuePresets.classic.setup({
      customize: {
        control(data) {
          if (data.payload instanceof DropdownControl) {
            return DropdownComponent;
          }
          if (data.payload instanceof ClassicPreset.InputControl) {
            return VuePresets.classic.Control;
          }
        },
      },
    }),
  );
  render.addPreset(VuePresets.contextMenu.setup());
  scopes.addPreset(ScopesPresets.classic.setup());

  editor.use(area);
  editor.use(engine);
  area.use(connection);
  area.use(render);
  area.use(scopes);
  area.use(arrange);

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
        .filter(
          (n) =>
            !(n instanceof LogicScopeNode || n instanceof FunctionScopeNode),
        )
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
  indent: string = "",
) {
  console.log(node.id);
  const nodeData = await engine.fetch(node.id);
  let fullCode = "";
  if (node instanceof SeparateNode) {
    fullCode += indent + nodeData.x.code + "\n";
    fullCode += indent + nodeData.y.code + "\n";
    fullCode += nodeData.z.code !== "" ? indent + nodeData.z.code + "\n" : "";
    fullCode += nodeData.w.code !== "" ? indent + nodeData.w.code + "\n" : "";
  } else if (node instanceof ConditionNode) {
    let trueCode = "";
    let falseCode = "";
    // fullCode += "\t" + nodeData.true.code + "\n";

    const blockContent = graph.outgoers(node.id).nodes();
    for (let content of blockContent) {
      const scopeIncomers = graph.incomers(content.id).nodes();
      let incomersCode = "";
      if (scopeIncomers.length > 0) {
        incomersCode += await orderedCode(scopeIncomers, visited, indent);
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
        trueCode += await getScopeCode(scopeChildren, visited, indent);
      }

      fullCode +=
        indent +
        nodeData.true.code +
        "\n" +
        trueCode +
        (await getNodesCode(content, visited, graph, indent));
    }

    // fullCode += "\t" + nodeData.false.code + "\n";
    for (let content of blockContent) {
      if (content.label != "False") continue;
      visited.push(content.id);
      const scopeChildren = editor
        .getNodes()
        .filter((n) => n.parent === content.id);
      if (scopeChildren.length > 0) {
        falseCode += await getScopeCode(scopeChildren, visited, indent);
      }

      fullCode +=
        indent +
        nodeData.false.code +
        "\n" +
        falseCode +
        (await getNodesCode(content, visited, graph, indent));
    }
  } else if (node instanceof CustomFunctionNode) {
    let scopeCode = "";
    fullCode += nodeData.value.code + "\n";

    const blockContent = graph.outgoers(node.id).nodes();
    for (let content of blockContent) {
      visited.push(content.id);
      const scopeChildren = editor
        .getNodes()
        .filter((n) => n.parent === content.id);
      if (scopeChildren.length > 0) {
        scopeCode += await getScopeCode(scopeChildren, visited, indent);
      }

      fullCode +=
        scopeCode + (await getNodesCode(content, visited, graph, indent));
    }
  } else {
    if (nodeData.value.code !== "")
      fullCode += indent + nodeData.value.code + "\n";
  }

  return fullCode;
}

async function getScopeCode(
  scopeChildren: Nodes[],
  visited: string[],
  prevIndent: string,
) {
  if (scopeChildren.length <= 0) return;

  return orderedCode(scopeChildren, visited, prevIndent + "\t");
}

async function logCode() {
  const graph = structures(editor);
  const allNodes = graph
    .nodes()
    .filter((n) => !n.parent)
    .filter((n) => !(n instanceof CustomFunctionNode));
  const customFunctionNodes = graph
    .nodes()
    .filter((n) => n instanceof CustomFunctionNode);
  console.log(allNodes.map((n) => n.id));

  if (allNodes.length <= 0) return;

  let visited: string[] = [];
  let fullCode =
    (await orderedCode(customFunctionNodes, visited)) +
    "\n\nfn evaluateImage(input2: vec2f) -> vec3f {\n" +
    (await orderedCode(allNodes, visited, "\t"));

  // fullCode += (await getNodesCode(endNode, [], graph, "\t")) + "}";
  fullCode += "\n}";
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
  let nodeQueue = allNodes;

  while (nodeQueue.length > 0) {
    const node = nodeQueue.shift();
    if (!node) break;
    console.log(node.id);
    if (visited.includes(node.id)) continue;
    if (node instanceof NothingNode) {
      await editor.removeNode(node.id);
      continue;
    }
    const incomers = graph.incomers(node.id).nodes();
    if (
      incomers.some((inc) => !visited.includes(inc.id)) ||
      (node instanceof ReturnNode && nodeQueue.length > 0)
    ) {
      nodeQueue.push(node);
      continue;
    }

    visited.push(node.id);
    fullCode += await getNodesCode(node, visited, graph, indent);
  }

  return fullCode;
}

async function serialize() {
  const sg = new SerializedGraph();
  for (let node of editor.getNodes()) {
    const sn = new SerializedNode();
    sg.add(node.serialize(sn));
  }

  for (let connection of editor.getConnections()) {
    const sn = sg.getNode(connection.target);
    if (!sn) continue;
    sn.inputs.push({
      type: "node",
      value: connection.source,
      keyFrom: connection.sourceOutput,
      keyTo: connection.targetInput,
    });
  }

  emit("save", makeFilePath(`${saveName.value}.graph`), sg.toJSON());
}

async function replaceOrAddDeserialize(
  name: string,
  json: string,
  add: boolean,
) {
  console.log(`replaceOrAddDeserialize(${name}, ${json}, ${add});`);
  if (!add) {
    await editor.clear();
    await deserialize(json);
  } else {
    shouldUpdate = false;
    const func = new CustomFunctionNode(
      (n) => area.update("node", n.id),
      (n) => editor.addNode(n),
      (n) => editor.removeNode(n.id),
      (nA, kA, nB, kB) =>
        editor.addConnection(new ClassicPreset.Connection(nA, kA, nB, kB)),
    );
    func.nameControl.setValue(name);
    func.label = name;
    await editor.addNode(func);
    shouldUpdate = true;
    await deserialize(json, func.functionScope.id);
  }
}

async function deserialize(json: string, parent?: string) {
  shouldUpdate = false;
  const sg = graphFromJSON(json);

  for (let snObj of sg.graph) {
    const sn = toSerializedNode(snObj);
    const node = serializedNodeToNode(sn);
    node.deserialize(sn);
    node.parent = parent;
    await editor.addNode(node);
  }

  for (let snObj of sg.graph) {
    const sn = toSerializedNode(snObj);
    for (let input of sn.inputs) {
      if (input.type === "node") {
        await editor.addConnection(
          new ClassicPreset.Connection(
            editor.getNodes().filter((n) => n.id === input.value)[0],
            input.keyFrom,
            editor.getNodes().filter((n) => n.id === sn.uuid)[0],
            input.keyTo,
          ),
        );
      }
    }
  }

  shouldUpdate = true;
  await arrange.layout();
}

function serializedNodeToNode(sn: SerializedNode): Nodes {
  let node: Nodes;
  switch (sn.nodeType) {
    case "Number":
      node = new NumberNode((n) => area.update("node", n.id));
      break;
    case "Math":
      node = new MathOpNode("+", (n, c) => area.update("node", n.id));
      break;
    case "Vector":
      node = new VectorNode(4, (n) => area.update("node", n.id));
      break;
    case "Separate":
      node = new SeparateNode((n) => area.update("node", n.id));
      break;
    case "Join":
      node = new JoinNode((n) => area.update("node", n.id));
      break;
    case "FunctionCall":
      node = new FunctionCallNode();
      break;
    case "Return":
      node = new ReturnNode("0");
      break;
    case "VariableOut":
      node = new VariableOutNode(0, "");
      break;
    case "VariableIn":
      node = new VariableInNode("");
      break;
    case "Initialize":
      node = new InitializeNode();
      break;
    case "FunctionScope":
      node = new FunctionScopeNode("", (n) => area.update("node", n.id));
      break;
    case "CustomFunction":
      node = new CustomFunctionNode((n) => area.update("node", n.id));
      break;
    case "CallCustomFunction":
      node = new CallCustomFunctionNode((n) => area.update("node", n.id));
      break;
    case "LogicScope":
      node = new LogicScopeNode("", (n) => area.update("node", n.id));
      break;
    case "Condition":
      node = new ConditionNode("", "==");
      break;
  }

  node.id = sn.uuid;
  node.width = sn.size[0];
  node.height = sn.size[1];
  node.parent = sn.parent;
  return node;
}
</script>

<template>
  <n-modal :show="saving || loading" :mask-closable="false">
    <n-card
      class="w-full sm:w-1/2 lg:w-1/3"
      title="Save Graph"
      closable
      @close="saving = false"
      v-on:pointerdown.stop=""
      v-if="saving"
    >
      Please enter a file name to save the graph to.
      <template #action>
        <div class="flex justify-around">
          <n-input v-model:value="saveName"></n-input>
        </div>
        <div class="flex justify-around">
          <n-button
            type="primary"
            @click="
              serialize();
              saving = false;
            "
            v-on:pointerdown.stop=""
          >
            Save
          </n-button>
          <n-button
            type="warning"
            @click="saving = false"
            v-on:pointerdown.stop=""
          >
            Cancel
          </n-button>
        </div>
      </template>
    </n-card>
    <n-card
      class="w-full sm:w-1/2 lg:w-1/3"
      title="Load Graph"
      closable
      @close="loading = false"
      v-on:pointerdown.stop=""
      v-if="loading"
    >
      <template #header-extra>
        <n-p>Please select the graph file to load.</n-p>
      </template>
      <template #action>
        <n-flex vertical>
          <n-select v-model:value="loadName" :options="props.graphs"></n-select>
          <div class="flex justify-around">
            <n-button
              type="primary"
              @click="
                emit('load', makeFilePath(loadName), false);
                loading = false;
              "
              v-on:pointerdown.stop=""
            >
              Replace Graph
            </n-button>
            <n-button
              type="primary"
              @click="
                emit('load', makeFilePath(loadName), true);
                loading = false;
              "
              v-on:pointerdown.stop=""
            >
              Add to Graph
            </n-button>
          </div>
        </n-flex>
      </template>
      <template #footer>
        <n-text type="info" strong
          >Note that adding the graph to this graph will create a Custom
          Function Node containing the loaded graph. <br />Replacing the graph
          will delete all nodes currently contained within this graph.</n-text
        >
      </template>
    </n-card>
  </n-modal>
  <n-flex vertical style="width: 70%">
    <div class="rete" ref="container" style="width: 100%; height: 71%"></div>
    <n-flex
      ><n-button @click="saving = true" v-on:pointerdown.stop="">Save </n-button
      ><n-button
        @click="
          //deserialize(
          //  `{&quot;graph&quot;:[{&quot;size&quot;:[180,140],&quot;uuid&quot;:&quot;ffec57f1db36b382&quot;,&quot;inputs&quot;:[],&quot;nodeType&quot;:&quot;VariableOut&quot;,&quot;extraStringInformation&quot;:[{&quot;key&quot;:&quot;code&quot;,&quot;value&quot;:&quot;&quot;},{&quot;key&quot;:&quot;ref&quot;,&quot;value&quot;:&quot;input2&quot;}],&quot;extraNumberInformation&quot;:[{&quot;key&quot;:&quot;value&quot;,&quot;value&quot;:{&quot;0&quot;:1,&quot;1&quot;:1}}]},{&quot;size&quot;:[180,140],&quot;uuid&quot;:&quot;c813ab16c5471e3c&quot;,&quot;inputs&quot;:[],&quot;nodeType&quot;:&quot;VariableOut&quot;,&quot;extraStringInformation&quot;:[{&quot;key&quot;:&quot;code&quot;,&quot;value&quot;:&quot;var PI = 3.14159265359;&quot;},{&quot;key&quot;:&quot;ref&quot;,&quot;value&quot;:&quot;PI&quot;}],&quot;extraNumberInformation&quot;:[{&quot;key&quot;:&quot;value&quot;,&quot;value&quot;:3.14159265359}]},{&quot;size&quot;:[180,140],&quot;uuid&quot;:&quot;0ecbb9b9a229ce19&quot;,&quot;inputs&quot;:[],&quot;nodeType&quot;:&quot;VariableOut&quot;,&quot;extraStringInformation&quot;:[{&quot;key&quot;:&quot;code&quot;,&quot;value&quot;:&quot;var HALF_PI = 3.14159265359 / 2.0;&quot;},{&quot;key&quot;:&quot;ref&quot;,&quot;value&quot;:&quot;HALF_PI&quot;}],&quot;extraNumberInformation&quot;:[{&quot;key&quot;:&quot;value&quot;,&quot;value&quot;:1.570796326795}]},{&quot;size&quot;:[180,140],&quot;uuid&quot;:&quot;4a065e8821ad82e0&quot;,&quot;inputs&quot;:[],&quot;nodeType&quot;:&quot;VariableOut&quot;,&quot;extraStringInformation&quot;:[{&quot;key&quot;:&quot;code&quot;,&quot;value&quot;:&quot;var TWO_PI = 3.14159265359 * 2.0;&quot;},{&quot;key&quot;:&quot;ref&quot;,&quot;value&quot;:&quot;TWO_PI&quot;}],&quot;extraNumberInformation&quot;:[{&quot;key&quot;:&quot;value&quot;,&quot;value&quot;:6.28318530718}]},{&quot;size&quot;:[180,140],&quot;uuid&quot;:&quot;4e2cf9be57e0e973&quot;,&quot;inputs&quot;:[{&quot;key&quot;:&quot;def&quot;,&quot;value&quot;:&quot;vec3f(input2.x, 0, input2.y)&quot;,&quot;type&quot;:&quot;text&quot;}],&quot;nodeType&quot;:&quot;Return&quot;}]}`,
          //);
          loading = true
        "
        v-on:pointerdown.stop=""
        >Load</n-button
      >
    </n-flex>
  </n-flex>
</template>

<style scoped></style>
