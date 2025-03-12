import WaveSine from "~icons/mdi/sine-wave";
import WaveSawTool from "~icons/mdi/sawtooth-wave";
import WaveSquare from "~icons/mdi/square-wave";
import ArrowsSplit from "~icons/mdi/set-split";
import MathFunction from "~icons/mdi/function-variant";
import ArrowsJoin from "~icons/mdi/arrow-collapse-right";
import { MathOpNode, NumberNode } from "./basic/math";
import {
  FunctionCallNode,
  InitializeNode,
  NothingNode,
  ReturnNode,
  VariableInNode,
  VariableOutNode,
} from "./basic/nodes";
import { SeparateNode, JoinNode, VectorNode } from "./basic/vector";
import { CombineNode, MathFunctionNode } from "./simple-mode/apply";
import {
  newHeartShape,
  newSphereShape,
  newPlaneShape,
  newCylinderShape,
  ShapeNode,
  newCubeShape,
} from "./simple-mode/shapes";
import type { UINode } from "./ui/uinode";
import { ClassicPreset, type GetSchemes, type NodeEditor } from "rete";

import JoinFullRound from "~icons/mdi/full-outer-join";
import CategoryOutlined from "~icons/mdi/category-outline";
import Scale from "~icons/mdi/resize";
import Heart24Regular from "~icons/mdi/cards-heart-outline";
import Circle24Regular from "~icons/mdi/circle-outline";
import RectangleLandscape24Regular from "~icons/mdi/rectangle-outline";
import type { AreaPlugin } from "rete-area-plugin";
import type { VueArea2D } from "rete-vue-plugin";
import type { ContextMenuExtra } from "rete-context-menu-plugin";
import type {
  CustomFunctionNode,
  CallCustomFunctionNode,
  FunctionScopeNode,
} from "./basic/functions";
import type { ConditionNode, LogicScopeNode } from "./basic/logic";

export type AreaExtra = VueArea2D<Schemes> | ContextMenuExtra;

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
  | FunctionScopeNode
  | ShapeNode
  | MathFunctionNode;

export class Connection<
  A extends Nodes,
  B extends Nodes,
> extends ClassicPreset.Connection<A, B> {}

export type Conns =
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
  | Connection<InitializeNode, LogicScopeNode>
  | Connection<ConditionNode, LogicScopeNode>
  | Connection<CustomFunctionNode, FunctionScopeNode>;

export type Schemes = GetSchemes<Nodes, Conns>;

export function useUiNodes(
  editor: NodeEditor<Schemes>,
  area: AreaPlugin<Schemes, AreaExtra>
) {
  async function addNode(node: Nodes) {
    await editor.addNode(node);
  }

  function createUINode(uiNode: UINode) {
    addNode(uiNode.get());
  }

  function makeMap(nodes: UINode[]): Map<string, UINode> {
    return new Map<string, UINode>(
      nodes.map((node) => [node.name, node] as const)
    );
  }

  // Could be shrunk down a bit I think
  const uiNodes: Map<string, Map<string, UINode>> = new Map([
    [
      "Shapes",
      makeMap([
        {
          name: "Heart",
          type: "SHAPE",
          prefix: "parametric",
          image: Heart24Regular,
          get: () => {
            //addNode(n);
            return newHeartShape();
          },
          create: createUINode,
          draggable: true,
        },
        {
          name: "Sphere",
          type: "SHAPE",
          prefix: "parametric",
          image: Circle24Regular,
          get: () => {
            //addNode(n);
            return newSphereShape();
          },
          create: createUINode,
          draggable: true,
        },
        {
          name: "Plane",
          type: "SHAPE",
          prefix: "parametric",
          image: RectangleLandscape24Regular,
          get: () => {
            //addNode(n);
            return newPlaneShape();
          },
          create: createUINode,
          draggable: true,
        },
        {
          name: "Cylinder",
          type: "SHAPE",
          prefix: "parametric",
          image: CategoryOutlined,
          get: () => {
            return newCylinderShape();
          },
          create: createUINode,
          draggable: true,
        },
        {
          name: "Cube",
          type: "SHAPE",
          prefix: "parametric",
          image: RectangleLandscape24Regular,
          get: () => {
            return newCubeShape();
          },
          create: createUINode,
          draggable: true,
        },
      ]),
    ],
    [
      "Apply",
      new Map<string, UINode>([
        [
          "Combine",
          {
            name: "Combine",
            type: "APPLY",
            prefix: "",
            image: JoinFullRound,
            get: () => {
              //addNode(n);
              return new CombineNode(
                (id: string) => {
                  area.update("node", id);
                  editor.addNode(new NothingNode());
                },
                (c) => area.update("control", c.id)
              );
            },
            create: createUINode,
            draggable: true,
          },
        ],
        [
          "Scale",
          {
            name: "Scale",
            type: "APPLY",
            prefix: "",
            image: Scale,
            get: () => {
              return new MathFunctionNode(
                "Scale",
                "mat3x3(vec3f({scale x,1,100,-100,0.1,f32},0.0,0.0), vec3f(0.0,{scale y,1,100,-100,0.1,f32},0.0), vec3f(0.0,0.0,{scale z,1,100,-100,0.1,f32})) * input2",
                (id) => {
                  area.update("node", id);
                  editor.addNode(new NothingNode());
                },
                (c) => {
                  area.update("control", c.id);
                },
                false,
                "vec3f",
                "vec3f"
              );
            },
            create: createUINode,
            draggable: true,
          },
        ],
        [
          "Abs",
          {
            name: "Abs",
            type: "APPLY",
            prefix: "",
            image: Scale,
            get: () => {
              return new MathFunctionNode(
                "Abs",
                "abs(input2)",
                (id) => {
                  area.update("node", id);
                  editor.addNode(new NothingNode());
                },
                (c) => {
                  area.update("control", c.id);
                },
                false,
                "vec3f",
                "vec3f"
              );
            },
            create: createUINode,
            draggable: true,
          },
        ],
        [
          "Sine",
          {
            name: "Sine",
            type: "APPLY",
            prefix: "",
            image: WaveSine,
            get: () => {
              return new MathFunctionNode(
                "Sine",
                "sin({angular frequency,0.0,3.14159,-3.14159,0.1,f32} * input2 + {phase,0.0,3.14159,-3.14159,0.1,f32})",
                (id) => {
                  area.update("node", id);
                  editor.addNode(new NothingNode());
                },
                (c) => {
                  area.update("control", c.id);
                },
                false,
                "any",
                "any"
              );
            },
            create: createUINode,
            draggable: true,
          },
        ],
        [
          "Cosine",
          {
            name: "Cosine",
            type: "APPLY",
            prefix: "",
            image: WaveSine,
            get: () => {
              return new MathFunctionNode(
                "Cosine",
                "cos({angular frequency,0.0,3.14159,-3.14159,0.1,f32} * input2 + {phase,0.0,3.14159,-3.14159,0.1,f32})",
                (id) => {
                  area.update("node", id);
                  editor.addNode(new NothingNode());
                },
                (c) => {
                  area.update("control", c.id);
                },
                false,
                "any",
                "any"
              );
            },
            create: createUINode,
            draggable: true,
          },
        ],
        [
          "Arcus Sine",
          {
            name: "Arcus Sine",
            type: "APPLY",
            prefix: "",
            image: WaveSine,
            get: () => {
              return new MathFunctionNode(
                "asin",
                "asin({angular frequency,0.0,3.14159,-3.14159,0.1,f32} * input2 + {phase,0.0,3.14159,-3.14159,0.1,f32})",
                (id) => {
                  area.update("node", id);
                  editor.addNode(new NothingNode());
                },
                (c) => {
                  area.update("control", c.id);
                },
                false,
                "any",
                "any"
              );
            },
            create: createUINode,
            draggable: true,
          },
        ],
        [
          "Arcus Cosine",
          {
            name: "Arcus Cosine",
            type: "APPLY",
            prefix: "",
            image: WaveSine,
            get: () => {
              return new MathFunctionNode(
                "acos",
                "acos({angular frequency,0.0,3.14159,-3.14159,0.1,f32} * input2 + {phase,0.0,3.14159,-3.14159,0.1,f32})",
                (id) => {
                  area.update("node", id);
                  editor.addNode(new NothingNode());
                },
                (c) => {
                  area.update("control", c.id);
                },
                false,
                "any",
                "any"
              );
            },
            create: createUINode,
            draggable: true,
          },
        ],
        [
          "Sawtooth",
          {
            name: "Sawtooth",
            type: "APPLY",
            prefix: "",
            image: WaveSawTool,
            get: () => {
              return new MathFunctionNode(
                "Sawtooth",
                "(({sawtooth count,0,10,-10,0.1,f32} * input2) - floor({sawtooth count,0,10,-10,0.1,f32} * input2))",
                (id) => {
                  area.update("node", id);
                  editor.addNode(new NothingNode());
                },
                (c) => {
                  area.update("control", c.id);
                },
                false,
                "any",
                "any"
              );
            },
            create: createUINode,
            draggable: true,
          },
        ],
        [
          "POW",
          {
            name: "POW",
            type: "APPLY",
            prefix: "",
            image: WaveSine,
            get: () => {
              return new MathFunctionNode(
                "Pow",
                "pow(input2, {x1,0,10,-10,0.1,same})",
                (id) => {
                  area.update("node", id);
                  editor.addNode(new NothingNode());
                },
                (c) => {
                  area.update("control", c.id);
                },
                false,
                "any",
                "any"
              );
            },
            create: createUINode,
            draggable: true,
          },
        ],
        [
          "Square",
          {
            name: "Square",
            type: "APPLY",
            prefix: "",
            image: WaveSquare,
            get: () => {
              return new MathFunctionNode(
                "Square",
                "sign(sin(input2*{frequency,1,10,-10,0.1,f32}))",
                (id) => {
                  area.update("node", id);
                  editor.addNode(new NothingNode());
                },
                (c) => {
                  area.update("control", c.id);
                },
                false,
                "any",
                "any"
              );
            },
            create: createUINode,
            draggable: true,
          },
        ],
        [
          "Step",
          {
            name: "Step",
            type: "APPLY",
            prefix: "",
            image: WaveSquare,
            get: () => {
              return new MathFunctionNode(
                "Step",
                "step({edge,1,10,-10,1,same}, input2)",
                (id) => {
                  area.update("node", id);
                  editor.addNode(new NothingNode());
                },
                (c) => {
                  area.update("control", c.id);
                },
                false,
                "any",
                "any"
              );
            },
            create: createUINode,
            draggable: true,
          },
        ],
        [
          "Normalize",
          {
            name: "Normalize",
            type: "APPLY",
            prefix: "",
            image: WaveSquare,
            get: () => {
              return new MathFunctionNode(
                "Normalize",
                "normalize(input2)",
                (id) => {
                  area.update("node", id);
                  editor.addNode(new NothingNode());
                },
                (c) => {
                  area.update("control", c.id);
                },
                false,
                "any",
                "any"
              );
            },
            create: createUINode,
            draggable: true,
          },
        ],
        [
          "Perlin Noise",
          {
            name: "Perlin Noise",
            type: "APPLY",
            prefix: "",
            image: WaveSquare,
            get: () => {
              return new MathFunctionNode(
                "Perlin Noise",
                "cnoise(input2)",
                (id) => {
                  area.update("node", id);
                  editor.addNode(new NothingNode());
                },
                (c) => {
                  area.update("control", c.id);
                },
                false,
                "vec2f",
                "f32"
              );
            },
            create: createUINode,
            draggable: true,
          },
        ],
      ]),
    ],
    [
      "Parameters",
      new Map<string, UINode>([
        [
          "Split",
          {
            name: "Split",
            type: "ARRANGE",
            prefix: "vec->x,y,z",
            image: ArrowsSplit,
            get: () => {
              return new SeparateNode((n) => area.update("node", n.id));
            },
            create: createUINode,
            draggable: true,
          },
        ],
        [
          "Join",
          {
            name: "Join",
            type: "ARRANGE",
            prefix: "x,y,z->vec",
            image: ArrowsJoin,
            get: () => {
              return new JoinNode((n) => area.update("node", n.id));
            },
            create: createUINode,
            draggable: true,
          },
        ],
      ]),
    ],
    [
      "Maths",
      new Map<string, UINode>([
        [
          "Number",
          {
            name: "Number",
            type: "CALCULATE",
            prefix: "",
            image: MathFunction,
            get: () => {
              return new NumberNode((n) => area.update("node", n.id));
            },
            create: createUINode,
            draggable: true,
          },
        ],
        [
          "Add",
          {
            name: "Add",
            type: "CALCULATE",
            prefix: "",
            image: MathFunction,
            get: () => {
              return new MathOpNode("+", (node, control) => {
                area.update("node", node.id);
                area.update("control", control.id);
              });
            },
            create: createUINode,
            draggable: true,
          },
        ],
        [
          "Subtract",
          {
            name: "Subtract",
            type: "CALCULATE",
            prefix: "",
            image: MathFunction,
            get: () => {
              return new MathOpNode("-", (n, c) => {
                area.update("node", n.id);
                area.update("control", c.id);
              });
            },
            create: createUINode,
            draggable: true,
          },
        ],
        [
          "Multiply",
          {
            name: "Multiply",
            type: "CALCULATE",
            prefix: "",
            image: MathFunction,
            get: () => {
              return new MathOpNode("*", (node, control) => {
                area.update("node", node.id);
                area.update("control", control.id);
              });
            },
            create: createUINode,
            draggable: true,
          },
        ],
        [
          "Divide",
          {
            name: "Divide",
            type: "CALCULATE",
            prefix: "",
            image: MathFunction,
            get: () => {
              return new MathOpNode("/", (n, c) => {
                area.update("node", n.id);
                area.update("control", c.id);
              });
            },
            create: createUINode,
            draggable: true,
          },
        ],
        [
          "Modulo",
          {
            name: "Modulo",
            type: "CALCULATE",
            prefix: "",
            image: MathFunction,
            get: () => {
              return new MathOpNode("%", (n, c) => {
                area.update("node", n.id);
                area.update("control", c.id);
              });
            },
            create: createUINode,
            draggable: true,
          },
        ],
        [
          "Square Root",
          {
            name: "Square Root",
            type: "CALCULATE",
            prefix: "",
            image: MathFunction,
            get: () => {
              return new MathFunctionNode(
                "Square Root",
                "sqrt(input2)",
                (id) => {
                  area.update("node", id);
                  editor.addNode(new NothingNode());
                },
                (c) => {
                  area.update("control", c.id);
                },
                false,
                "any",
                "any"
              );
            },
            create: createUINode,
            draggable: true,
          },
        ],
        [
          "Floor",
          {
            name: "Floor",
            type: "CALCULATE",
            prefix: "",
            image: MathFunction,
            get: () => {
              return new MathFunctionNode(
                "Floor",
                "floor(input2)",
                (id) => {
                  area.update("node", id);
                  editor.addNode(new NothingNode());
                },
                (c) => {
                  area.update("control", c.id);
                },
                false,
                "any",
                "any"
              );
            },
            create: createUINode,
            draggable: true,
          },
        ],
        [
          "Ceil",
          {
            name: "Ceil",
            type: "CALCULATE",
            prefix: "",
            image: MathFunction,
            get: () => {
              return new MathFunctionNode(
                "Ceil",
                "ceil(input2)",
                (id) => {
                  area.update("node", id);
                  editor.addNode(new NothingNode());
                },
                (c) => {
                  area.update("control", c.id);
                },
                false,
                "any",
                "any"
              );
            },
            create: createUINode,
            draggable: true,
          },
        ],
        [
          "Exp",
          {
            name: "Exp",
            type: "CALCULATE",
            prefix: "",
            image: MathFunction,
            get: () => {
              return new MathFunctionNode(
                "Exp",
                "exp(input2)",
                (id) => {
                  area.update("node", id);
                  editor.addNode(new NothingNode());
                },
                (c) => {
                  area.update("control", c.id);
                },
                false,
                "any",
                "any"
              );
            },
            create: createUINode,
            draggable: true,
          },
        ],
      ]),
    ],
    [
      "Constants",
      new Map<string, UINode>([
        [
          "PI",
          {
            name: "PI",
            type: "CONSTANT",
            prefix: "",
            image: MathFunction,
            get: () => {
              return new VariableOutNode(0.0, "var PI = 3.14159265359;", "PI");
            },
            create: createUINode,
            draggable: true,
          },
        ],
        [
          "TWO PI",
          {
            name: "TWO PI",
            type: "CONSTANT",
            prefix: "",
            image: MathFunction,
            get: () => {
              return new VariableOutNode(
                0.0,
                "var TWO_PI = 3.14159265359 * 2.0;",
                "TWO_PI"
              );
            },
            create: createUINode,
            draggable: true,
          },
        ],
        [
          "HALF PI",
          {
            name: "HALF PI",
            type: "CONSTANT",
            prefix: "",
            image: MathFunction,
            get: () => {
              return new VariableOutNode(
                0.0,
                "var HALF_PI = 3.14159265359 / 2.0;",
                "HALF_PI"
              );
            },
            create: createUINode,
            draggable: true,
          },
        ],
        [
          "Elapsed Time",
          {
            name: "Elapsed Time",
            type: "CONSTANT",
            prefix: "",
            image: MathFunction,
            get: () => {
              return new VariableOutNode(0.0, "", "time.elapsed");
            },
            create: createUINode,
            draggable: true,
          },
        ],
        [
          "Delta Time",
          {
            name: "Delta Time",
            type: "CONSTANT",
            prefix: "",
            image: MathFunction,
            get: () => {
              return new VariableOutNode(0.0, "", "time.delta");
            },
            create: createUINode,
            draggable: true,
          },
        ],
        [
          "Frame",
          {
            name: "Frame",
            type: "CONSTANT",
            prefix: "",
            image: MathFunction,
            get: () => {
              return new VariableOutNode(
                0.0,
                "var frame = f32(time.frame);",
                "frame"
              );
            },
            create: createUINode,
            draggable: true,
          },
        ],
        [
          "Instance ID",
          {
            name: "Instance ID",
            type: "CONSTANT",
            prefix: "",
            image: MathFunction,
            get: () => {
              return new VariableOutNode(
                0.0,
                "var instanceId = f32(instance_id);",
                "instanceId"
              );
            },
            create: createUINode,
            draggable: true,
          },
        ],
      ]),
    ],
  ]);
  return { uiNodes };
}
