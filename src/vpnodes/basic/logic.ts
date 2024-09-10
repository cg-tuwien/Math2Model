import { ClassicPreset, type NodeEditor } from "rete";
import {
  VPNode,
  idToVariableName,
  NodeReturn,
  nodeToVariableDeclaration,
  reteSocket,
  VariableInNode,
  VariableOutNode,
} from "@/vpnodes/basic/nodes";
import { type Nodes } from "@/components/visual-programming/CodeGraph.vue";
import { ref } from "vue";
import { DropdownControl } from "@/vpnodes/controls/dropdown";
import {
  addCustomFunction,
  customFunctions,
  getCustomFunction,
  getCustomFunctionOptions,
  removeCustomFunction,
  subscribe,
  typeOptions,
} from "@/vpnodes/controls/dropdown-options";

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
  private varOutNode?: VariableOutNode;
  private varInNode?: VariableInNode;
  constructor(
    name: string,
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

    if (reference && this.addNode && !this.varOutNode && !this.varInNode) {
      this.varOutNode = new VariableOutNode(
        reference[0].value,
        "",
        reference[0].refId,
      );
      this.varInNode = new VariableInNode(reference[0].refId ?? "");

      this.varOutNode.parent = this.id;
      this.varInNode.parent = this.id;

      this.addNode(this.varOutNode);
      this.addNode(this.varInNode);
    }

    if (!reference) {
      if (this.varOutNode && this.varInNode && this.removeNode) {
        this.removeNode(this.varOutNode);
        this.removeNode(this.varInNode);
      }
      this.varOutNode = undefined;
      this.varInNode = undefined;
      result.value.code = "}";
      return result;
    }

    result.value.value = reference[0].value ?? 0;
    result.value.code = `}`;
    result.value.refId = reference[0].refId ?? "";

    return result;
  }
}

export class ConditionNode extends VPNode {
  constructor(
    name: string,
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
}
