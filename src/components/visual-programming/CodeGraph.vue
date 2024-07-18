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
  AddNode,
  Vector2Node,
  Seperate2Node,
  Join2Node,
  Vector3Node,
} from "@/vpnodes/nodes";
import { DataflowEngine } from "rete-engine";
import { DockPlugin, DockPresets } from "rete-dock-plugin";

const container = ref<HTMLElement | null>(null);

onMounted(() => {
  if (container.value === null) return;

  createEditor();
});

type Nodes =
  | NumberNode
  | AddNode
  | Vector2Node
  | Seperate2Node
  | Join2Node
  | Vector3Node;
class Connection<
  A extends Nodes,
  B extends Nodes,
> extends ClassicPreset.Connection<A, B> {}

type Conns =
  | Connection<NumberNode, AddNode>
  | Connection<AddNode, AddNode>
  | Connection<Vector2Node, Seperate2Node>
  | Connection<Seperate2Node, AddNode>
  | Connection<Seperate2Node, Join2Node>
  | Connection<Join2Node, Seperate2Node>
  | Connection<NumberNode, Join2Node>
  | Connection<Vector3Node, Seperate2Node>;

async function createEditor() {
  type Schemes = GetSchemes<Nodes, Conns>;

  const editor = new NodeEditor<Schemes>();
  const engine = new DataflowEngine<Schemes>();

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
      ["Add", () => new AddNode((c) => area.update("control", c.id))],
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
  dock.add(() => new AddNode((c) => area.update("control", c.id)));
  dock.add(() => new Vector2Node((c) => area.update("control", c.id)));
  dock.add(() => new Vector3Node((c) => area.update("control", c.id)));
  dock.add(() => new Seperate2Node((c) => area.update("control", c.id)));
  dock.add(() => new Join2Node((c) => area.update("control", c.id)));

  editor.addPipe((context) => {
    if (["connectioncreated", "connectionremoved"].includes(context.type)) {
      engine.reset();

      editor
        .getNodes()
        .filter((n) => n instanceof AddNode)
        .forEach((n) => engine.fetch(n.id));
    }
    return context;
  });
}
</script>

<template>
  <div class="rete" ref="container"></div>
</template>

<style scoped></style>
