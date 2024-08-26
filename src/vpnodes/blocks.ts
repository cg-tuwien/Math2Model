import { ClassicPreset } from "rete";
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

export class BlockNode extends ClassicPreset.Node {
  children?: ClassicPreset.Node[];
}

export class ScopeNode extends BlockNode {
  private lineno = 1;
  constructor(
    name: string,
    private update?: (node: ClassicPreset.Node) => void,
  ) {
    super(name);

    this.addInput("context", new ClassicPreset.Input(reteSocket, "Context"));

    this.addOutput(
      "line" + this.lineno.toString(),
      new ClassicPreset.Output(reteSocket, "Lines"),
    );
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

    if (this.outputs["line" + this.lineno.toString()]) {
      this.lineno++;
      this.addOutput(
        "line" + this.lineno.toString(),
        new ClassicPreset.Output(reteSocket, "Line " + this.lineno.toString()),
      );
      if (this.update) this.update(this);
    }

    result.value.value = context[0].value;
    result.value.code = "}";
    result.value.refId = context[0].refId ?? "";

    return result;
  }
}

export class ConditionNode extends BlockNode {
  constructor(
    name: string,
    private operator: "==" | "!=" | ">" | "<" | ">=" | "<=",
  ) {
    super(name);

    this.addInput("left", new ClassicPreset.Input(reteSocket, "Left"));
    this.addInput("right", new ClassicPreset.Input(reteSocket, "Right"));

    this.addOutput("true", new ClassicPreset.Output(reteSocket, "True"));
    this.addOutput("false", new ClassicPreset.Output(reteSocket, "False"));
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
