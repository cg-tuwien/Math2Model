<script setup lang="ts">
import { AreaExtensions, AreaPlugin } from "rete-area-plugin";
import { Presets as VuePresets, VuePlugin } from "rete-vue-plugin";
import { ClassicPreset, NodeEditor, type NodeId } from "rete";
import { computed, type DeepReadonly, onUnmounted, ref, watch } from "vue";
import {
  ConnectionPlugin,
  Presets as ConnectionPresets,
} from "rete-connection-plugin";
import { ContextMenuPlugin } from "rete-context-menu-plugin";
import {
  FunctionCallNode,
  InitializeNode,
  InstanceCountNode,
  ReturnNode,
  VariableInNode,
  VariableOutNode,
  type NodeData,
} from "@/vpnodes/basic/nodes";
import { structures } from "rete-structures";
import { Presets as ScopesPresets, ScopesPlugin } from "rete-scopes-plugin";
import { LogicScopeNode } from "@/vpnodes/basic/logic";
import { useThrottleFn, watchArray } from "@vueuse/core";
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
import type { VirtualModelState } from "@/scenes/scene-state";
import { assert } from "@stefnotch/typestef/assert";

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
  models: DeepReadonly<VirtualModelState>[];
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

const instanceCounts = computed(() => props.models.map((m) => m.instanceCount));
watchArray(instanceCounts, () => {
  console.log("Models changed, updating...");
  update();
});

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
const nodes = useUiNodes(editor, area, props.engine, update, props.models);
let uiNodes: Map<string, Map<string, UINode>> = nodes.uiNodes;
let shouldUpdate = true;

watch(container, (container) => {
  container?.append(area.container);
});

createEditor();

onUnmounted(() => {
  document.removeEventListener("keydown", keyListener);
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
}

function keyListener(e: KeyboardEvent) {
  if (e.code === "Delete") del();
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
      break;
    case "KeyC":
      copy();
      break;
    case "KeyV":
      paste();
      break;
    case "KeyD":
      e.preventDefault();
      e.stopPropagation();
      duplicate();
      break;
    default:
  }
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
}

function createEditor() {
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
                paste();
              },
            },
          ],
        };
      }
      const deleteItem: Item = {
        label: "Delete (del)",
        key: "delete",
        async handler() {
          shouldUpdate = false;
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
            shouldUpdate = true;
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

  document.addEventListener("keydown", keyListener);

  area.addPipe(async (context) => {
    // prevent zooming with double click
    if (context.type === "zoom") {
      if (context.data.source === "dblclick") {
        return;
      }
    }
    return context;
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
          if (
            context.payload instanceof VariableOutNode ||
            context.payload instanceof InstanceCountNode
          ) {
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
}

function update() {
  if (!shouldUpdate) return;
  saveGraphThrottled();
}

function saveGraph() {
  serialize();
  logCode();
}
const saveGraphThrottled = useThrottleFn(saveGraph, 500, true, false);

/** Returns all connections. Can be queried with `.get(nodeId)[inputName]` */
function inputConnections() {
  const inputs = new Map<NodeId, { [key: string]: Conns }>();
  editor.getConnections().forEach((c) => {
    const obj = inputs.get(c.target);
    if (obj === undefined) {
      inputs.set(c.target, {
        [c.targetInput]: c,
      });
    } else {
      obj[c.targetInput] = c;
    }
  });
  return inputs;
}

function getNodesCode(node: Nodes, nodeData: NodeData, indent: string = "") {
  let fullCode = "";
  if (node instanceof SeparateNode) {
    fullCode += indent + nodeData.x.code + "\n";
    fullCode += indent + nodeData.y.code + "\n";
    fullCode += nodeData.z.code !== "" ? indent + nodeData.z.code + "\n" : "";
    fullCode += nodeData.w.code !== "" ? indent + nodeData.w.code + "\n" : "";
  } else if (node instanceof InstanceCountNode) {
    const icNode = node as InstanceCountNode;
    const instanceCount = props.models.find(
      (m) => m.id === icNode.modelId
    )?.instanceCount;
    fullCode += indent + nodeData.value.code + "f32(" + instanceCount + ");\n";
  } else {
    if (nodeData.value && nodeData.value.code !== "")
      fullCode += indent + nodeData.value.code + "\n";
  }

  return fullCode;
}

function logCode() {
  const returnNodes = editor.getNodes().filter((n) => n instanceof ReturnNode);

  if (returnNodes.length <= 0) return;

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
    orderedCode(returnNodes, "\t");

  fullCode += "\n}";
  emit("update", fullCode);
}

function orderedCode(startNodes: Nodes[], indent: string = "") {
  const allConnections = inputConnections();
  let fullCode = "";

  // Permanent mark
  const nodeOutput = new Map<NodeId, NodeData>();
  const temporaryMark = new Set<NodeId>();

  // https://en.wikipedia.org/wiki/Topological_sorting#Depth-first_search
  function visit(node: Nodes) {
    if (nodeOutput.has(node.id)) return;
    if (temporaryMark.has(node.id)) {
      showError("Graph is cyclic!");
      return;
    }

    temporaryMark.add(node.id);

    const connections = allConnections.get(node.id) ?? {};
    for (const connection of Object.values(connections)) {
      const child = editor.getNode(connection.source);
      assert(child, "invalid connection");
      visit(child);
    }

    const inputKeys = Object.keys(node.inputs);

    const inputData: Partial<NodeData> = Object.fromEntries(
      inputKeys.map((v) => {
        const connection = connections[v];
        if (connection) {
          return [
            v,
            nodeOutput.get(connection.source)?.[connection.sourceOutput],
          ];
        } else {
          return [v, undefined];
        }
      })
    );
    const nodeData = node.data(inputData);
    nodeOutput.set(node.id, nodeData);
    fullCode += getNodesCode(node, nodeData, indent);
  }

  startNodes.forEach((n) => visit(n));

  return fullCode;
}

function serialize() {
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

async function deserialize(json: string) {
  shouldUpdate = false;
  if (json === "") {
    json = BasicGraph;
  }
  const sg = graphFromJSON(json);
  const nodes = new Map<string, Nodes>();
  const promises = [];
  for (let snObj of sg.graph) {
    if (snObj.nodeType === "NothingNode") continue;
    const sn = toSerializedNode(snObj);
    const node = serializedNodeToNode(sn);
    nodes.set(sn.uuid, node);
    node.deserialize(sn);
    node.id = sn.uuid;
    promises.push(editor.addNode(node));
  }
  await Promise.all(promises);
  promises.length = 0;

  for (let snObj of sg.graph) {
    const sn = toSerializedNode(snObj);
    const node = nodes.get(sn.uuid);
    assert(node);
    if (sn.position) {
      promises.push(
        area.translate(node.id, { x: sn.position[0], y: sn.position[1] })
      );
    }
    for (let input of sn.inputs) {
      if (input.type === "node") {
        promises.push(
          editor.addConnection(
            new ClassicPreset.Connection(
              editor.getNodes().filter((n) => n.id === input.value)[0],
              input.keyFrom,
              editor.getNodes().filter((n) => n.id === sn.uuid)[0],
              input.keyTo
            )
          )
        );
      }
    }
    node?.updateSize(area);
  }

  await Promise.all(promises);
  promises.length = 0;

  shouldUpdate = true;
  history.clear();
}

function serializedNodeToNode(sn: SerializedNode): Nodes {
  let node: Nodes;
  switch (sn.nodeType) {
    case "Number":
      node = new NumberNode(
        (id) => genericUpdate(id, editor, area, update),
        (cont) => genericUpdateControl(cont, area)
      );
      break;
    case "Math":
      node = new MathOpNode(
        "+",
        (id) => genericUpdate(id, editor, area, update),
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
    case "Shape":
      node = new ShapeNode("", "");
      break;
    case "Combine":
      node = new CombineNode(
        (id) => genericUpdate(id, editor, area, update),
        (cont) => sliderUpdateConrol(cont, area, props.engine),
        (id) => callGenericUpdate(id, editor, area, update)
      );
      break;
    case "MathFunction":
      node = new MathFunctionNode(
        "",
        "",
        (id) => genericUpdate(id, editor, area, update),
        (cont) => sliderUpdateConrol(cont, area, props.engine),
        (id) => callGenericUpdate(id, editor, area, update)
      );
      break;
    case "InstanceCount":
      node = new InstanceCountNode("");
      break;
    default:
      console.error(sn);
      throw new Error("Invalid node");
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
