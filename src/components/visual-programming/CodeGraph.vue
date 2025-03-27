<script setup lang="ts">
import { AreaExtensions, AreaPlugin } from "rete-area-plugin";
import { Presets as VuePresets, VuePlugin } from "rete-vue-plugin";
import { ClassicPreset, NodeEditor } from "rete";
import { computed, type DeepReadonly, onMounted, ref, watch } from "vue";
import {
  ConnectionPlugin,
  Presets as ConnectionPresets,
} from "rete-connection-plugin";
import { ContextMenuPlugin } from "rete-context-menu-plugin";
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
import { MathOpNode, NumberNode } from "@/vpnodes/basic/math";
import { JoinNode, SeparateNode, VectorNode } from "@/vpnodes/basic/vector";
import { DropdownControl } from "@/vpnodes/controls/dropdown";
import DropdownComponent from "@/vpnodes/components/DropdownComponent.vue";
import { graphFromJSON, SerializedGraph } from "@/vpnodes/serialization/graph";
import { SerializedNode, toSerializedNode } from "@/vpnodes/serialization/node";
import {
  type FilePath,
  type ReactiveFilesystem,
} from "@/filesystem/reactive-files";
import type { SelectMixedOption } from "naive-ui/es/select/src/interface";
import { showError, showInfo } from "@/notification";
import BasicGraph from "@/../parametric-renderer-core/graphs/BasicGraph.graph?raw";
import HeartWGSL from "@/../parametric-renderer-core/graphs/Heart.graph.wgsl?raw";
import SphereWGSL from "@/../parametric-renderer-core/graphs/Sphere.graph.wgsl?raw";
import PlaneWGSL from "@/../parametric-renderer-core/graphs/Plane.graph.wgsl?raw";
import CylinderWGSL from "@/../parametric-renderer-core/graphs/Cylinder.graph.wgsl?raw";
import CubeWGSL from "@/../parametric-renderer-core/graphs/Cube.graph.wgsl?raw";
import NoiseFunction from "@/../parametric-renderer-core/graphs/NoiseFunction.wgsl?raw";
import {
  type HistoryActions,
  HistoryPlugin,
  Presets as HistoryPresets,
} from "rete-history-plugin";
import { ShapeNode } from "@/vpnodes/simple-mode/shapes";
import { CombineNode, MathFunctionNode } from "@/vpnodes/simple-mode/apply";
import NodesDock from "@/components/visual-programming/NodesDock.vue";
import type { UINode } from "@/vpnodes/ui/uinode";
import { SliderControl } from "@/vpnodes/controls/slider";
import SliderComponent from "@/vpnodes/components/SliderComponent.vue";
import ReturnNodeStyle from "@/components/visual-programming/CustomNodeStyles/ReturnNodeStyle.vue";
import VariableOutNodeStyle from "@/components/visual-programming/CustomNodeStyles/VariableOutNodeStyle.vue";
import DefaultNodeStyle from "@/components/visual-programming/CustomNodeStyles/DefaultNodeStyle.vue";
import SocketStyle from "@/components/visual-programming/CustomNodeStyles/SocketStyle.vue";
import ConnectionStyle from "@/components/visual-programming/CustomNodeStyles/ConnectionStyle.vue";
import {
  callGenericUpdate,
  Connection,
  genericUpdate,
  genericUpdateControl,
  sliderUpdateConrol,
  useUiNodes,
  type AreaExtra,
  type Conns,
  type Nodes,
  type Schemes,
} from "@/vpnodes/nodes-list";
import { NumberControl } from "@/vpnodes/controls/number";
import NumberComponent from "@/vpnodes/components/NumberComponent.vue";
import type { Item } from "rete-context-menu-plugin/_types/types";
import type { WgpuEngine } from "@/engine/wgpu-engine";

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
  engine: WgpuEngine;
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

const container = ref<HTMLElement | null>(null);

const editor = new NodeEditor<Schemes>();
const engine = new DataflowEngine<Schemes>();
const arrange = new AutoArrangePlugin<Schemes>();
const area: AreaPlugin<Schemes, AreaExtra> = new AreaPlugin<Schemes, AreaExtra>(
  document.createElement("div")
);
area.container.classList.add("flex-1");
// area.area.setZoomHandler(new CustomZoom(0.1));
const scopes = new ScopesPlugin<Schemes>();
const history = new HistoryPlugin<Schemes, HistoryActions<Schemes>>();
const applier = new ArrangeAppliers.TransitionApplier<Schemes, never>({
  duration: 100,
  timingFunction: (t) => t,
  async onTick() {
    // called on every frame update
  },
});

const nodeClipBoard: Nodes[] = [];

const selector = AreaExtensions.selector();

AreaExtensions.selectableNodes(area, selector, {
  accumulating: AreaExtensions.accumulateOnCtrl(),
});
const nodes = useUiNodes(editor, area, engine, props.engine, update);
let uiNodes: Map<string, Map<string, UINode>> = nodes.uiNodes;
let shouldUpdate = true;

watch(container, (container) => {
  container?.append(area.container);
});

onMounted(() => {
  createEditor();
});

watch(
  () => props.keyedGraph?.id,
  async () => {
    shouldUpdate = false;
    let code = props.keyedGraph?.code ?? "";
    await editor.clear();
    await deserialize(code);
    shouldUpdate = true;
  }
);

async function addNodeAtPosition(node: Nodes, x: number, y: number) {
  console.log("Adding", node, "at", x, ",", y);
  await editor.addNode(node);
  void area.translate(node.id, { x: x, y: y });
}

async function rearrange() {
  await arrange.layout({ applier });
  showInfo("Your nodes got automatically arranged! To undo press CTRL+Z.");
  return new NothingNode();
}

function copy() {
  // Clear clipboard
  while (nodeClipBoard.length > 0) {
    nodeClipBoard.pop();
  }
  if (selector.entities.size == 0) return;
  // Get selected nodes
  selector.entities.forEach((entity) => {
    const node = editor.getNode(entity.id);
    if (node) {
      nodeClipBoard.push(node);
    }
  });

  showInfo(
    "Copied " +
      nodeClipBoard.length +
      " node" +
      (nodeClipBoard.length > 1 ? "s" : "") +
      " to clipboard."
  );
}

async function del() {
  console.log(selector.entities);
  if (selector.entities.size == 0) return;
  const toDelete: string[] = [];
  selector.entities.forEach((entity) => toDelete.push(entity.id));
  if (toDelete.length == 0) return;
  shouldUpdate = false;
  selector.unselectAll();
  for (let id of toDelete) {
    const node = editor.getNode(id);
    if (node?.label === "input2" || node?.label === "Return") continue;
    const connections = editor.getConnections().filter((c) => {
      return c.source === id || c.target === id;
    });

    for (const connection of connections) {
      await editor.removeConnection(connection.id);
    }
    await editor.removeNode(id);
  }
  shouldUpdate = true;
  showInfo(
    "Deleted " +
      toDelete.length +
      " node" +
      (toDelete.length > 1 ? "s" : "") +
      ". Press CTRL + Z to undo deletion of last deleted node."
  );
  await editor.addNode(new NothingNode());
}

async function duplicate() {
  const oldClipboard = nodeClipBoard.copyWithin(0, 0);
  copy();
  await paste();

  while (nodeClipBoard.length > 0) {
    nodeClipBoard.pop();
  }

  for (let node of oldClipboard) {
    nodeClipBoard.push(node);
  }
}

async function paste() {
  if (nodeClipBoard.length == 0) return;
  shouldUpdate = false;
  for (let node of nodeClipBoard) {
    const clone = node.clone();
    const nodePos = area.nodeViews.get(node.id)?.position;
    if (clone) {
      await addNodeAtPosition(
        clone,
        area.area.pointer.x + (nodePos?.x ?? 0),
        area.area.pointer.y + (nodePos?.y ?? 0)
      );
    }
  }
  shouldUpdate = true;
  showInfo(
    "Pasted " +
      nodeClipBoard.length +
      " node" +
      (nodeClipBoard.length > 1 ? "s" : "") +
      " from clipboard."
  );
  await editor.addNode(new NothingNode());
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

async function createEditor() {
  const contextMenu = new ContextMenuPlugin<Schemes>({
    items(context) {
      if (context === "root") {
        return {
          searchBar: false,
          list: [
            {
              label: "Rearrange (CTRL+A)",
              key: "1",
              handler: () => {
                rearrange();
              },
            },
            {
              label: "Undo (CTRL+Z)",
              key: "2",
              handler: () => {
                history.undo();
              },
            },
            {
              label: "Redo (CTRL+Y)",
              key: "3",
              handler: () => {
                history.redo();
              },
            },
            {
              label: "Paste (CTRL+V)",
              key: "4",
              handler: () => {
                void paste();
              },
            },
          ],
        };
      }
      const deleteItem: Item = {
        label: "Delete (del)",
        key: "delete",
        async handler() {
          if ("source" in context && "target" in context) {
            // connection
            const connectionId = context.id;

            await editor.removeConnection(connectionId);
          } else {
            // node
            const nodeId = context.id;
            const node = editor.getNode(nodeId);
            if (node?.label === "input2" || node?.label === "Return") return;
            const connections = editor.getConnections().filter((c) => {
              return c.source === nodeId || c.target === nodeId;
            });

            for (const connection of connections) {
              await editor.removeConnection(connection.id);
            }
            await editor.removeNode(nodeId);
          }
        },
      };

      const clone = context.clone?.bind(context);
      const cloneItem: undefined | Item = clone && {
        label: "Clone (CTRL+D)",
        key: "clone",
        async handler() {
          const node = clone();
          if (node) {
            await editor.addNode(node);

            void area.translate(node.id, area.area.pointer);
          }
        },
      };

      const copyItem: Item = {
        label: "Copy (CTRL+C)",
        key: "copy",
        handler: () => {
          copy();
        },
      };

      return {
        searchBar: false,
        list: [deleteItem, ...(cloneItem ? [cloneItem] : []), copyItem],
      };
    },
  });

  arrange.addPreset(ArrangePresets.classic.setup());
  history.addPreset(HistoryPresets.classic.setup());

  document.addEventListener("keydown", (e) => {
    if (e.code === "Delete") void del();
    if (!e.ctrlKey && !e.metaKey) return;

    switch (e.code) {
      case "KeyY":
        history.undo();
        break;
      case "KeyZ":
        history.redo();
        break;
      case "KeyA":
        e.preventDefault();
        e.stopPropagation();
        rearrange();
        break;
      case "KeyS":
        e.preventDefault();
        e.stopPropagation();
        showInfo("You don't need to save!");
        editor.addNode(new NothingNode());
        break;
      case "KeyC":
        e.preventDefault();
        e.stopPropagation();
        copy();
        break;
      case "KeyV":
        e.preventDefault();
        e.stopPropagation();
        paste();
        break;
      case "KeyD":
        e.preventDefault();
        e.stopPropagation();
        duplicate();
        break;
      default:
    }
  });

  document.addEventListener("dblclick", (e) => {
    e.preventDefault();
    e.stopPropagation();
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
          if (data.payload instanceof NumberControl) {
            return NumberComponent;
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

  editor.addPipe((context) => {
    if (context.type === "connectioncreated") {
      if (shouldUpdate) {
        checkForUnsafeConnections(
          context.data as ClassicPreset.Connection<Nodes, Nodes>
        ).then(update); // TODO: FIX IT!;
      }
    }
    if (context.type === "nodecreated") {
      setTimeout(() => context.data.updateSize(area), 0);
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

  await deserialize(props.keyedGraph?.code ?? "");

  await logCode();
}

function update() {
  if (!shouldUpdate) return;
  engine.reset();
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
  const allNodes = graph.nodes().filter((n) => !n.parent);

  if (allNodes.length <= 0) return;

  let visited: string[] = [];
  /**
   * Order of code assembling
   *  1. Write all custom function blocks outside of main (sampleObject) method
   *  2. Write template functions (Heart, Sphere, Plane, Cylinder) outside of main method
   *  3. Write code into main method in correct order (e.g. all needed variables for a line are declared before that line)
   */
  let fullCode =
    HeartWGSL +
    "\n" +
    SphereWGSL +
    "\n" +
    PlaneWGSL +
    "\n" +
    CylinderWGSL +
    "\n" +
    CubeWGSL +
    "\n" +
    NoiseFunction +
    "\n\nfn sampleObject(input2: vec2f) -> vec3f {\n" +
    (await orderedCode(allNodes, visited, "\t"));

  fullCode += "\n}";
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

async function deserialize(json: string, parent?: string) {
  shouldUpdate = false;
  if (json === "") {
    json = BasicGraph;
    emit("save", json);
  }
  const sg = graphFromJSON(json);
  const nodes = new Map<string, Nodes>();

  for (let snObj of sg.graph) {
    const sn = toSerializedNode(snObj);
    const node = serializedNodeToNode(sn);
    nodes.set(sn.uuid, node);
    node.deserialize(sn);
    await editor.addNode(node);
    node.id = sn.uuid;
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
      // node.parent = idMap.get(sn.parent);
      node.parent = sn.parent;
      scopes
        .update(node.parent ?? node.id)
        .catch((reason) => showError("Could not update parent", reason));
    }
    for (let input of sn.inputs) {
      if (input.type === "node") {
        await editor.addConnection(
          new ClassicPreset.Connection(
            editor.getNodes().filter((n) => n.id === input.value)[0],
            input.keyFrom,
            editor.getNodes().filter((n) => n.id === sn.uuid)[0],
            input.keyTo
          )
        );
      }
    }
    node?.updateSize(area);
  }

  shouldUpdate = true;
  await editor.addNode(new NothingNode());
  await history.clear();
  // await rearrange();
}

function serializedNodeToNode(sn: SerializedNode): Nodes {
  let node: Nodes;
  switch (sn.nodeType) {
    case "Number":
      node = new NumberNode(
        (id) => genericUpdate(id, editor, area, engine, update),
        (cont) => genericUpdateControl(cont, area)
      );
      break;
    case "Math":
      node = new MathOpNode(
        "+",
        (id) => genericUpdate(id, editor, area, engine, update),
        (cont) => genericUpdateControl(cont, area)
      );
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
        (id) => genericUpdate(id, editor, area, engine, update),
        (cont) => sliderUpdateConrol(cont, area, props.engine),
        (id) => callGenericUpdate(id, editor, area, engine, update)
      );
      break;
    case "MathFunction":
      node = new MathFunctionNode(
        "",
        "",
        (id) => genericUpdate(id, editor, area, engine, update),
        (cont) => sliderUpdateConrol(cont, area, props.engine),
        (id) => callGenericUpdate(id, editor, area, engine, update)
      );
      break;
    default:
      return new NothingNode();
  }
  node.id = sn.uuid;
  return node;
}
</script>
<template>
  <div class="flex w-full h-full border border-gray-500">
    <NodesDock
      :display-nodes="uiNodes"
      :editor="editor"
      style="width: 25%"
    ></NodesDock>
    <div class="flex flex-1 relative">
      <div
        class="flex flex-1"
        ref="container"
        @dragover="
          (ev) => {
            ev.preventDefault();
            if (ev.dataTransfer) ev.dataTransfer.dropEffect = 'copy';
          }
        "
        @drop="
          (ev) => {
            ev.preventDefault();
            console.log(ev);
            if (ev.dataTransfer == null) return;
            const node = JSON.parse(
              ev.dataTransfer.getData('text/plain')
            ) as UINode;
            let toCreate: Nodes | null = null;
            uiNodes.forEach((value: Map<string, UINode>) => {
              if (value && value.has(node.name)) {
                const uiNode = value.get(node.name);
                toCreate = uiNode ? uiNode.get() : null;
              }
            });
            if (toCreate) {
              area.area.setPointerFrom(ev);
              addNodeAtPosition(
                toCreate,
                area.area.pointer.x,
                area.area.pointer.y
              );
            }
          }
        "
      ></div>
      <div class="absolute top-2 right-2">
        <n-button
          quaternary
          circle
          type="primary"
          @click="emit('code')"
          class="text-xl"
        >
          <template #icon>
            <n-icon size="32px"><mdi-code /></n-icon>
          </template>
        </n-button>
      </div>
    </div>
  </div>
</template>
