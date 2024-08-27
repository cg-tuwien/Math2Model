import { ClassicPreset, type NodeEditor } from "rete";
import {
  idToVariableName,
  NodeReturn,
  reteSocket,
  VPNode,
} from "@/vpnodes/nodes";

function applyLogic(
  left: any,
  right: any,
  op: "==" | "!=" | ">" | "<" | ">=" | "<=",
): boolean {
  switch (op) {
    case "==":
      return left == right;
    case "!=":
      return left != right;
    case ">":
      return left > right;
    case "<":
      return left < right;
    case ">=":
      return left >= right;
    case "<=":
      return left <= right;
  }
}

export class BlockNode extends VPNode {}

export class ScopeNode extends BlockNode {
  constructor(
    name: string,
    private update?: (node: ClassicPreset.Node) => void,
  ) {
    super(name);

    this.addInput("context", new ClassicPreset.Input(reteSocket, "Context"));

    // this.addOutput("value", new ClassicPreset.Output(reteSocket, "Next"));
  }

  data(input: { context: NodeReturn[] }): { value: NodeReturn } {
    const { context } = input;
    let result = {
      value: {
        value: 0,
        code: "",
        refId: "",
      },
    };
    if (!context) return result;

    result.value.value = context[0].value;
    result.value.code = "}";
    result.value.refId = context[0].refId ?? "";

    return result;
  }
}

export class ConditionNode extends VPNode {
  constructor(
    name: string,
    private operator: "==" | "!=" | ">" | "<" | ">=" | "<=",
    trueScope: ScopeNode,
    falseScope: ScopeNode,
    editor: NodeEditor<any>,
  ) {
    super(name);

    this.addInput("left", new ClassicPreset.Input(reteSocket, "Left"));
    this.addInput("right", new ClassicPreset.Input(reteSocket, "Right"));

    this.addOutput("true", new ClassicPreset.Output(reteSocket, "True"));
    this.addOutput("false", new ClassicPreset.Output(reteSocket, "False"));

    editor.addConnection(
      new ClassicPreset.Connection(this, "true", trueScope, "context"),
    );
    editor.addConnection(
      new ClassicPreset.Connection(this, "false", falseScope, "context"),
    );

    this.updateSize();
  }

  data(input: { left?: NodeReturn[]; right?: NodeReturn[] }): {
    true: NodeReturn;
    false: NodeReturn;
  } {
    const { left, right } = input;

    return {
      true: {
        value: applyLogic(
          left ? left[0].value : 0,
          right ? right[0].value : 0,
          this.operator,
        ),
        code: `if(${left ? left[0].refId : 0} ${this.operator} ${right ? right[0].value : 0}) {`,
      },
      false: {
        value: applyLogic(
          left ? left[0].value : 0,
          right ? right[0].value : 0,
          this.operator,
        ),
        code: `else {`,
      },
    };
  }
}

export class TemplateNode extends VPNode {}
