<script setup lang="ts">
import { AreaExtensions, AreaPlugin } from "rete-area-plugin";
import { Presets as VuePresets, VuePlugin } from "rete-vue-plugin";
import { ClassicPreset, NodeEditor } from "rete";
import { computed, type DeepReadonly, onMounted, ref, watch } from "vue";
import {
  ConnectionPlugin,
  Presets as ConnectionPresets,
} from "rete-connection-plugin";
import {
  ContextMenuPlugin,
  Presets as ContextMenuPresets,
} from "rete-context-menu-plugin";
import {
  FunctionCallNode,
  InitializeNode,
  NothingNode,
  ReturnNode,
  VariableInNode,
  VariableOutNode,
} from "@/vpnodes/basic/nodes";
import { DataflowEngine } from "rete-engine";
import { structures } from "rete-structures";
import { Presets as ScopesPresets, ScopesPlugin } from "rete-scopes-plugin";
import { ConditionNode, LogicScopeNode } from "@/vpnodes/basic/logic";
import type { Structures } from "rete-structures/_types/types";
import { useThrottleFn } from "@vueuse/core";
import {
  ArrangeAppliers,
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
import {
  type FilePath,
  makeFilePath,
  type ReactiveFilesystem,
} from "@/filesystem/reactive-files";
import type { SelectMixedOption } from "naive-ui/es/select/src/interface";
import { showError } from "@/notification";
import BasicGraph from "@/../parametric-renderer-core/graphs/BasicGraph.graph?raw";
import HeartWGSL from "@/../parametric-renderer-core/graphs/Heart.graph.wgsl?raw";
import SphereWGSL from "@/../parametric-renderer-core/graphs/Sphere.graph.wgsl?raw";
import PlaneWGSL from "@/../parametric-renderer-core/graphs/Plane.graph.wgsl?raw";
import CylinderWGSL from "@/../parametric-renderer-core/graphs/Cylinder.graph.wgsl?raw";
import NoiseFunction from "@/../parametric-renderer-core/graphs/NoiseFunction.wgsl?raw";
import {
  type HistoryActions,
  HistoryPlugin,
  Presets as HistoryPresets,
} from "rete-history-plugin";
import {
  newHeartShape,
  newPlaneShape,
  newSphereShape,
  ShapeNode,
} from "@/vpnodes/simple-mode/shapes";
import { CombineNode, MathFunctionNode } from "@/vpnodes/simple-mode/apply";
import NodesDock from "@/components/visual-programming/NodesDock.vue";
import type { UINode } from "@/vpnodes/ui/uinode";
import { SliderControl } from "@/vpnodes/controls/slider";
import SliderComponent from "@/vpnodes/components/SliderComponent.vue";
import Code from "~icons/mdi/code";
import ReturnNodeStyle from "@/components/visual-programming/CustomNodeStyles/ReturnNodeStyle.vue";
import VariableOutNodeStyle from "@/components/visual-programming/CustomNodeStyles/VariableOutNodeStyle.vue";
import DefaultNodeStyle from "@/components/visual-programming/CustomNodeStyles/DefaultNodeStyle.vue";
import SocketStyle from "@/components/visual-programming/CustomNodeStyles/SocketStyle.vue";
import ConnectionStyle from "@/components/visual-programming/CustomNodeStyles/ConnectionStyle.vue";
import {
  Connection,
  useUiNodes,
  type AreaExtra,
  type Conns,
  type Nodes,
  type Schemes,
} from "@/vpnodes/nodes-list";

const emit = defineEmits<{
  update: [content: string];
  save: [content: string];
  code: [];
}>();

export interface KeyedGraph {
  readonly id: string;
  readonly code: string;
}

const props = defineProps<{
  fs: ReactiveFilesystem;
  keyedGraph: DeepReadonly<KeyedGraph> | null;
}>();

const fileNames = ref(new Set<FilePath>());
props.fs.watchFromStart((change) => {
  if (!change.key.endsWith(".graph")) return;
  if (change.type === "insert") {
    fileNames.value.add(change.key);
  } else if (change.type === "remove") {
    fileNames.value.delete(change.key);
  }
});

const graphsDropdown = computed<SelectMixedOption[]>(() => {
  return [...fileNames.value]
    .toSorted()
    .filter((fileName) => fileName.endsWith(".graph"))
    .map(
      (fileName): SelectMixedOption => ({
        label: fileName.substring(
          0,
          fileName.valueOf().length - ".graph".length
        ),
        value: fileName,
      })
    );
});

const container = ref<HTMLElement | null>(null);

const editor = new NodeEditor<Schemes>();
const engine = new DataflowEngine<Schemes>();
const arrange = new AutoArrangePlugin<Schemes>();
const area: AreaPlugin<Schemes, AreaExtra> = new AreaPlugin<Schemes, AreaExtra>(
  document.createElement("div")
);
area.container.classList.add("flex-1");
const scopes = new ScopesPlugin<Schemes>();
const history = new HistoryPlugin<Schemes, HistoryActions<Schemes>>();
const applier = new ArrangeAppliers.TransitionApplier<Schemes, never>({
  duration: 100,
  timingFunction: (t) => t,
  async onTick() {
    // called on every frame update
  },
});

AreaExtensions.selectableNodes(area, AreaExtensions.selector(), {
  accumulating: AreaExtensions.accumulateOnCtrl(),
});

const nodes = useUiNodes(editor, area);
let uiNodes: Map<string, Map<string, UINode>> = nodes.uiNodes;
let shouldUpdate = true;

let loading = ref<boolean>(false);
let loadName = ref<string>("no file selected");

watch(container, (container) => {
  container?.append(area.container);
});

onMounted(() => {
  createEditor();
});

watch(
  () => props.keyedGraph?.id,
  async (id) => {
    console.log("watch keyedGraph.id", { id });
    shouldUpdate = false;
    let code = props.keyedGraph?.code ?? "";
    await editor.clear();
    await deserialize(code);
    shouldUpdate = true;
  }
);

async function newFunctionNode(area: AreaPlugin<Schemes, AreaExtra>) {
  shouldUpdate = false;
  const cfn = new CustomFunctionNode(
    (n) => area.update("node", n.id),
    (n) => editor.addNode(n),
    (n) => editor.removeNode(n.id),
    (nA, kA, nB, kB) => editor.addConnection(new Connection(nA, kA, nB, kB))
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
  operator: "==" | "!=" | ">" | "<" | ">=" | "<="
): Promise<Nodes> {
  shouldUpdate = false;
  const sc1 = new LogicScopeNode(
    scope1,
    (n) => area.update("node", n.id),
    (n) => editor.addNode(n),
    (n) => editor.removeNode(n.id)
  );
  const sc2 = new LogicScopeNode(
    scope2,
    (n) => area.update("node", n.id),
    (n) => editor.addNode(n),
    (n) => editor.removeNode(n.id)
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

  await arrange.layout({ applier });

  shouldUpdate = true;

  return new NothingNode();
}

async function addNodeAtMousePosition(node: Nodes, x: number, y: number) {
  console.log("Adding", node, "at", x, ",", y);
  await editor.addNode(node);
  void area.translate(node.id, { x: x, y: y });
}

async function rearrange() {
  await arrange.layout({ applier });
  await area.translate(endNode.id, { x: 200, y: 200 });
  return new NothingNode();
}

async function checkForUnsafeConnections(
  connection: ClassicPreset.Connection<Nodes, Nodes>
) {
  const start = connection.source;
  const end = connection.target;
  let removeFlag = false;
  if (start === end) {
    await editor.removeConnection(connection.id);
    showError(
      "This connection is not allowed, since it would create a cycle!",
      { title: "Invalid Connection" }
    );
    return;
  }

  shouldUpdate = false;
  // await editor.removeConnection(connection.id);

  const graph = structures(editor);

  // Check ancestors of start node
  const ancestors = graph.predecessors(start).nodes();
  // if end node is part of ancestors -> Cycle!
  removeFlag = ancestors.map((n) => n.id).includes(end);

  shouldUpdate = true;

  if (removeFlag) {
    await editor.removeConnection(connection.id);
    showError(
      "This connection is not allowed, since it would create a cycle!",
      { title: "Invalid Connection" }
    );
    return;
  }

  await editor.addNode(new NothingNode());
}

const endNode = new ReturnNode("vec3f(input2.x, 0, input2.y)", "Output Vertex");

async function createEditor() {
  const contextMenu = new ContextMenuPlugin<Schemes>({
    items: ContextMenuPresets.classic.setup([
      ["Rearrange", () => rearrange()],
      ["Initialize", () => new InitializeNode()],
      [
        "Advanced",
        [
          ["Equals", () => newConditionNode("True", "False", area, "==")],
          ["Not Equals", () => newConditionNode("True", "False", area, "!=")],
          ["Lt", () => newConditionNode("True", "False", area, "<")],
          ["Le", () => newConditionNode("True", "False", area, "<=")],
          ["Gt", () => newConditionNode("True", "False", area, ">")],
          ["Ge", () => newConditionNode("True", "False", area, ">=")],
          ["New Function", () => newFunctionNode(area)],
          [
            "Call Custom Function",
            () => new CallCustomFunctionNode((n) => area.update("node", n.id)),
          ],
        ],
      ],
    ]),
  });

  arrange.addPreset(ArrangePresets.classic.setup());
  history.addPreset(HistoryPresets.classic.setup());

  document.addEventListener("keydown", (e) => {
    if (!e.ctrlKey && !e.metaKey) return;

    switch (e.code) {
      case "KeyY":
        void history.undo();
        break;
      case "KeyZ":
        void history.redo();
        break;
      case "KeyS":
        e.preventDefault();
        e.stopPropagation();
        void rearrange();
        break;
      default:
    }
  });

  area.use(contextMenu);

  const connection = new ConnectionPlugin<Schemes, AreaExtra>();

  connection.addPreset(ConnectionPresets.classic.setup());

  const render = new VuePlugin<Schemes, AreaExtra>();

  // Setup custom Components for Nodes, Connections and Sockets
  render.addPreset(
    VuePresets.classic.setup({
      customize: {
        control(data) {
          if (data.payload instanceof DropdownControl) {
            return DropdownComponent;
          }
          if (data.payload instanceof SliderControl) {
            return SliderComponent;
          }
          if (data.payload instanceof ClassicPreset.InputControl) {
            return VuePresets.classic.Control;
          }
        },
        node(context) {
          if (context.payload instanceof ReturnNode) {
            return ReturnNodeStyle;
          }
          if (context.payload instanceof VariableOutNode) {
            return VariableOutNodeStyle;
          }
          return DefaultNodeStyle;
        },
        socket() {
          return SocketStyle;
        },
        connection() {
          return ConnectionStyle;
        },
      },
    })
  );

  render.addPreset(VuePresets.contextMenu.setup());
  scopes.addPreset(ScopesPresets.classic.setup());

  editor.use(area);
  editor.use(engine);
  area.use(connection);
  area.use(render);
  area.use(scopes);
  area.use(arrange);
  area.use(history);

  await deserialize(props.keyedGraph?.code ?? "");
  await arrange.layout({ applier });

  editor.addPipe((context) => {
    if (context.type === "connectioncreated") {
      if (shouldUpdate) {
        checkForUnsafeConnections(
          context.data as ClassicPreset.Connection<Nodes, Nodes>
        ).then(update); // TODO: FIX IT!;
      }
    }
    if (
      [
        "connectionremoved",
        "nodecreated",
        "noderemoved",
        "scopeupdated",
      ].includes(context.type)
    ) {
      update();
    }

    return context;
  });

  await logCode();
}

function update() {
  if (!shouldUpdate) return;
  engine.reset();

  editor
    .getNodes()
    .filter(
      (n) => !(n instanceof LogicScopeNode || n instanceof FunctionScopeNode)
    )
    .forEach((n) => n.updateSize(area));

  saveGraphThrottled();
}

async function saveGraph() {
  await Promise.all([serialize(), logCode()]);
}
const saveGraphThrottled = useThrottleFn(saveGraph, 500, true, false);

async function getNodesCode(
  node: Nodes,
  visited: string[],
  graph: Structures<Nodes, Conns>,
  indent: string = ""
) {
  const nodeData = await engine.fetch(node.id);
  let fullCode = "";
  if (node instanceof SeparateNode) {
    fullCode += indent + nodeData.x.code + "\n";
    fullCode += indent + nodeData.y.code + "\n";
    fullCode += nodeData.z.code !== "" ? indent + nodeData.z.code + "\n" : "";
    fullCode += nodeData.w.code !== "" ? indent + nodeData.w.code + "\n" : "";
  } else if (node instanceof ConditionNode) {
    // TODO Decide if ConditionNodes are to be removed
    let trueCode = "";
    let falseCode = "";

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
  prevIndent: string
) {
  if (scopeChildren.length <= 0) return;

  return await orderedCode(scopeChildren, visited, prevIndent + "\t");
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

  if (allNodes.length <= 0) return;

  let visited: string[] = [];
  /**
   * Order of code assembling
   *  1. Write all custom function blocks outside of main (sampleObject) method
   *  2. Write template functions (Heart, Sphere, Plane, Cylinder) outside of main method
   *  3. Write code into main method in correct order (e.g. all needed variables for a line are declared before that line)
   */
  let fullCode =
    (await orderedCode(customFunctionNodes, visited)) +
    "\n" +
    HeartWGSL +
    "\n" +
    SphereWGSL +
    "\n" +
    PlaneWGSL +
    "\n" +
    CylinderWGSL +
    "\n" +
    NoiseFunction +
    "\n\nfn sampleObject(input2: vec2f) -> vec3f {\n" +
    (await orderedCode(allNodes, visited, "\t"));

  fullCode += "\n}";
  console.log(fullCode);
  emit("update", fullCode);
}

async function orderedCode(
  allNodes: Nodes[],
  visited: string[],
  indent: string = ""
) {
  if (allNodes.length <= 0) return "";
  const graph = structures(editor);
  let fullCode = "";
  let nodeQueue = allNodes;

  while (nodeQueue.length > 0) {
    const node = nodeQueue.shift();
    if (!node) break;
    if (visited.includes(node.id)) continue;
    if (node instanceof NothingNode) {
      await editor.removeNode(node.id);
      continue;
    }
    const incomers = graph.incomers(node.id).nodes();
    // Check if any incomers of current node weren't visited yet and ignore this node for now if so
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
    const nodePos = area.nodeViews.get(node.id)?.position;
    sn.position = [nodePos?.x ?? 0, nodePos?.y ?? 0];
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

  emit("save", sg.toJSON());
}

async function replaceOrAddDeserialize(
  name: string,
  json: string,
  add: boolean
) {
  if (!add) {
    await editor.clear();
    await deserialize(json);
  } else {
    // TODO this is currently unused. Logic for loading another code graph into this one as a custom callable function
    shouldUpdate = false;
    const func = new CustomFunctionNode(
      (n) => area.update("node", n.id),
      (n) => editor.addNode(n),
      (n) => editor.removeNode(n.id),
      (nA, kA, nB, kB) => editor.addConnection(new Connection(nA, kA, nB, kB)),
      true
    );
    func.nControl.setValue(1);
    func.paramControls[0].cont.selected = "vec2f";
    func.setParamName(0, "input2");
    func.retControl.selected = "vec3f";
    func.nameControl.setValue(name);
    func.label = name;

    await editor.addNode(func);
    shouldUpdate = true;
    await editor.addNode(new NothingNode());
    await deserialize(json, func.functionScope.id);
  }
}

async function deserialize(json: string, parent?: string) {
  shouldUpdate = false;
  if (json === "") {
    json = BasicGraph;
    emit("save", json);
  }
  const sg = graphFromJSON(json);
  const idMap = new Map<string, string>();
  const nodes = new Map<string, Nodes>();

  for (let snObj of sg.graph) {
    const sn = toSerializedNode(snObj);
    const node = serializedNodeToNode(sn, idMap);
    nodes.set(sn.uuid, node);
    node.deserialize(sn);
    await editor.addNode(node);
    if (sn.position)
      void area.translate(node.id, { x: sn.position[0], y: sn.position[1] });
  }

  for (let snObj of sg.graph) {
    const sn = toSerializedNode(snObj);
    const node = nodes.get(sn.uuid);
    if (
      node &&
      !node.parent &&
      !sn.parent &&
      sn.nodeType !== "CustomFunction" &&
      sn.nodeType !== "FunctionScope"
    ) {
      node.parent = parent;
      scopes
        .update(parent ?? node.id)
        .catch((reason) => showError("Could not update parent", reason));
    } else if (node && !node.parent && sn.parent) {
      node.parent = idMap.get(sn.parent);
      scopes
        .update(node.parent ?? node.id)
        .catch((reason) => showError("Could not update parent", reason));
    }
    for (let input of sn.inputs) {
      if (input.type === "node") {
        await editor.addConnection(
          new ClassicPreset.Connection(
            editor.getNodes().filter((n) => n.id === idMap.get(input.value))[0],
            input.keyFrom,
            editor.getNodes().filter((n) => n.id === idMap.get(sn.uuid))[0],
            input.keyTo
          )
        );
      }
    }
    node?.updateSize(area);
  }

  shouldUpdate = true;
  await editor.addNode(new NothingNode());
  // await rearrange();
}

function serializedNodeToNode(
  sn: SerializedNode,
  idMap: Map<string, string>
): Nodes {
  let node: Nodes;
  switch (sn.nodeType) {
    case "Number":
      node = new NumberNode((n) => area.update("node", n.id));
      break;
    case "Math":
      node = new MathOpNode("+", (n) => area.update("node", n.id));
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
    case "Shape":
      node = new ShapeNode("", "");
      break;
    case "Combine":
      node = new CombineNode(
        (id) => {
          area.update("node", id);
          editor.addNode(new NothingNode());
        },
        (c) => {
          area.update("control", c.id);
        }
      );
      break;
    case "MathFunction":
      node = new MathFunctionNode(
        "",
        "",
        (id) => {
          area.update("node", id);
          editor.addNode(new NothingNode());
        },
        (c) => {
          area.update("control", c.id);
        }
      );
      break;
  }

  idMap.set(sn.uuid, node.id);
  node.width = sn.size[0];
  node.height = sn.size[1];
  //node.parent = sn.parent;
  return node;
}

function replaceOrAddGraph(filePath: FilePath, add: boolean) {
  props.fs
    .readTextFile(filePath)
    ?.then((content) =>
      replaceOrAddDeserialize(filePath.replace(".graph", ""), content, add)
    )
    .catch((reason) => showError("Could not load graph " + filePath, reason));
}
</script>

<template>
  <n-modal :show="loading" :mask-closable="false">
    <n-card
      class="w-full sm:w-1/2 lg:w-1/3"
      title="Load Graph"
      closable
      @close="loading = false"
      v-on:pointerdown.stop=""
    >
      <template #header-extra>
        <n-p>Please select the graph file to load.</n-p>
      </template>
      <template #action>
        <n-flex vertical>
          <n-select
            v-model:value="loadName"
            :options="graphsDropdown"
          ></n-select>
          <div class="flex justify-around">
            <n-button
              type="primary"
              @click="
                replaceOrAddGraph(makeFilePath(loadName), false);
                loading = false;
              "
              v-on:pointerdown.stop=""
            >
              Replace Graph
            </n-button>
            <n-button
              type="primary"
              @click="
                replaceOrAddGraph(makeFilePath(loadName), true);
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
  <n-flex vertical style="width: 100%">
    <n-flex style="height: 100%">
      <NodesDock
        :display-nodes="uiNodes"
        :editor="editor"
        header="Nodes"
        style="width: 25%"
      ></NodesDock>
      <div
        class="flex flex-1"
        ref="container"
        v-on:dragover="
          (ev) => {
            ev.preventDefault();
            if (ev.dataTransfer) ev.dataTransfer.dropEffect = 'copy';
          }
        "
        v-on:drop="
          (ev) => {
            ev.preventDefault();
            console.log(ev);
            if (ev.dataTransfer == null) return;
            const node = JSON.parse(
              ev.dataTransfer.getData('text/plain')
            ) as UINode;
            let toCreate: Nodes | null = null;
            uiNodes.forEach((value: Map<string, UINode>, key: string) => {
              if (value && value.has(node.name)) {
                const uiNode = value.get(node.name);
                toCreate = uiNode ? uiNode.get() : null;
              }
            });
            if (toCreate) {
              area.area.setPointerFrom(ev);
              addNodeAtMousePosition(
                toCreate,
                area.area.pointer.x,
                area.area.pointer.y
              );
            }
          }
        "
      >
        <n-button
          quaternary
          circle
          class="float-right m-2 z-10 mt-7"
          type="primary"
          v-on:click="emit('code')"
        >
          <template #icon>
            <n-icon :component="Code" size="32px"></n-icon>
          </template>
        </n-button>
      </div>
    </n-flex>
  </n-flex>
</template>
