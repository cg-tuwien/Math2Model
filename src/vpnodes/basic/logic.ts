import { ClassicPreset } from "rete";
import {
  VPNode,
  NodeReturn,
  reteSocket,
  VariableInNode,
  VariableOutNode,
} from "@/vpnodes/basic/nodes";
import { type Nodes } from "@/components/visual-programming/CodeGraph.vue";
import { type SerializedNode } from "@/vpnodes/serialization/node";

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

export class LogicScopeNode extends BlockNode {
  private varOutNode: VariableOutNode = new VariableOutNode(0, "");
  private varInNode: VariableInNode = new VariableInNode("");
  constructor(
    private name: string,
    private update?: (node: ClassicPreset.Node) => void,
    private addNode?: (node: Nodes) => void,
    private removeNode?: (node: Nodes) => void,
  ) {
    super(name);

    this.addInput("context", new ClassicPreset.Input(reteSocket, "Context"));
    this.addInput(
      "reference",
      new ClassicPreset.Input(reteSocket, "Outside Variable"),
    );

    this.addOutput(
      "value",
      new ClassicPreset.Output(reteSocket, "Outside Variable Result"),
    );

    this.varOutNode.parent = this.id;
    this.varInNode.parent = this.id;
  }

  data(input: { context: NodeReturn[]; reference: NodeReturn[] }): {
    value: NodeReturn;
  } {
    const { context, reference } = input;
    let result = {
      value: {
        value: 0,
        code: "",
        refId: "",
      },
    };
    if (!context) return result;

    if (
      reference &&
      this.addNode &&
      this.varOutNode.ref === "" &&
      this.varInNode.ref === ""
    ) {
      this.varOutNode.ref = reference[0].refId;
      this.varOutNode.value = reference[0].value;
      this.varInNode.ref = reference[0].refId ?? "";

      this.addNode(this.varOutNode);
      this.addNode(this.varInNode);
    }

    if (!reference) {
      if (
        this.varOutNode.ref !== "" &&
        this.varInNode.ref !== "" &&
        this.removeNode
      ) {
        this.removeNode(this.varOutNode);
        this.removeNode(this.varInNode);
      }
      this.varOutNode.ref = "";
      this.varInNode.ref = "";
      result.value.code = "}";
      return result;
    }

    result.value.value = reference[0].value ?? 0;
    result.value.code = `}`;
    result.value.refId = reference[0].refId ?? "";

    return result;
  }

  serialize(sn: SerializedNode): SerializedNode {
    sn.nodeType = "LogicScope";
    sn.extraStringInformation = [{ key: "name", value: this.name }];
    return super.serialize(sn);
  }

  deserialize(sn: SerializedNode) {
    if (sn.extraStringInformation) {
      for (let info of sn.extraStringInformation) {
        if (info.key === "name") {
          this.name = info.value;
          this.label = info.value;
        }
      }
    }

    super.deserialize(sn);
  }
}

export class ConditionNode extends VPNode {
  constructor(
    private name: string,
    private operator: "==" | "!=" | ">" | "<" | ">=" | "<=",
  ) {
    super(name);

    this.addInput("left", new ClassicPreset.Input(reteSocket, "Left"));
    this.addInput("right", new ClassicPreset.Input(reteSocket, "Right"));

    this.addOutput("true", new ClassicPreset.Output(reteSocket, "True"));
    this.addOutput("false", new ClassicPreset.Output(reteSocket, "False"));

    //this.updateSize();
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
        code: `if(${left ? left[0].refId ?? left[0].value : 0} ${this.operator} ${right ? right[0].refId ?? right[0].value : 0}) {`,
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

  serialize(sn: SerializedNode): SerializedNode {
    sn.nodeType = "Condition";

    sn.extraStringInformation = [
      { key: "name", value: this.name },
      { key: "op", value: this.operator },
    ];

    return super.serialize(sn);
  }

  deserialize(sn: SerializedNode) {
    if (sn.extraStringInformation) {
      for (let info of sn.extraStringInformation) {
        if (info.key === "name") {
          this.name = info.value;
          this.label = info.value;
        }
        if (info.key === "op") {
          this.operator = info.value as "==" | ">=" | "<=" | ">" | "<" | "!=";
        }
      }
    }

    super.deserialize(sn);
  }
}
