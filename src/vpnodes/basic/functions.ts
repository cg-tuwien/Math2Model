import {
  idToVariableName,
  InitializeNode,
  NodeReturn,
  nodeToVariableDeclaration,
  reteSocket,
  ReturnNode,
  VariableInNode,
  VariableOutNode,
  VPNode,
} from "@/vpnodes/basic/nodes";
import { ClassicPreset } from "rete";
import { DropdownControl } from "@/vpnodes/controls/dropdown";
import {
  addCustomFunction,
  getCustomFunction,
  getCustomFunctionOptions,
  removeCustomFunction,
  subscribe,
  typeOptions,
} from "@/vpnodes/controls/dropdown-options";
import { BlockNode } from "@/vpnodes/basic/logic";
import type { Nodes } from "@/components/visual-programming/CodeGraph.vue";

export class FunctionScopeNode extends BlockNode {
  public retNode: ReturnNode = new ReturnNode(0);
  public paramNodes: InitializeNode[] = [];
  constructor(
    name: string,
    private update?: (node: ClassicPreset.Node) => void,
  ) {
    super(name);

    this.addInput("context", new ClassicPreset.Input(reteSocket, "Context"));
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

    if (!reference) {
      result.value.code = "}";
      return result;
    }

    result.value.value = reference[0].value ?? 0;
    result.value.code = `}`;
    result.value.refId = reference[0].refId ?? "";

    return result;
  }
}

export class CustomFunctionNode extends VPNode {
  private nameControl: ClassicPreset.InputControl<"text", string>;
  private nControl: ClassicPreset.InputControl<"number", number>;
  private retControl: DropdownControl;
  private paramControls: { key: string; cont: DropdownControl }[] = [];
  private functionScope = new FunctionScopeNode("Function Scope");

  constructor(
    private update?: (node: ClassicPreset.Node) => void,
    private addNode?: (node: Nodes) => void,
    private removeNode?: (node: Nodes) => void,
    private addConnection?: (
      nodeA: Nodes,
      keyA: string,
      nodeB: Nodes,
      keyB: string,
    ) => void,
  ) {
    super("Custom Function");
    this.label = "func" + idToVariableName(this.id);
    this.nameControl = new ClassicPreset.InputControl("text", {
      initial: "func" + idToVariableName(this.id),
      change: (value) => {
        removeCustomFunction(this);
        this.label = value;
        addCustomFunction(this);
      },
    });

    this.nControl = new ClassicPreset.InputControl("number", {
      initial: 0,
      change: (value) => {
        removeCustomFunction(this);
        value = Math.min(9, value);
        value = Math.max(0, value);
        this.nControl.value = value;
        while (this.paramControls.length > 0) {
          const param = this.paramControls.shift();
          if (param) this.removeControl(param.key);

          if (this.functionScope && this.removeNode) {
            const paramNode = this.functionScope.paramNodes.shift();
            if (paramNode) this.removeNode(paramNode);
          }
        }

        for (let i = 0; i < value; i++) {
          const drpdwnControl = new DropdownControl(
            typeOptions,
            "i32",
            "Parameter type",
            "arg" + i.toString() + " type",
          );
          this.addControl("arg" + i.toString(), drpdwnControl);
          this.paramControls.push({
            key: "arg" + i.toString(),
            cont: drpdwnControl,
          });

          if (this.functionScope && this.addNode) {
            const paramNode = new VariableOutNode(0, "", "arg" + i.toString());
            paramNode.parent = this.functionScope.id;
            this.addNode(paramNode);
            this.functionScope.paramNodes.push(paramNode);
            this.addNode(this.functionScope.retNode);
            this.functionScope.retNode.parent = this.functionScope.id;
          }
        }
        if (this.update) this.update(this);
        this.extraHeightControls = 2.5 * value;
        this.updateSize();
        addCustomFunction(this);
      },
    });

    this.retControl = new DropdownControl(
      typeOptions,
      "",
      "Return type",
      "return type",
      (s) => {
        this.functionScope.retNode.def = typeToValueCode(s);
        if (this.update) this.update(this);
      },
    );
    this.addControl("name", this.nameControl);
    this.addControl("n", this.nControl);

    this.addControl("ret", this.retControl);

    this.addOutput("value", new ClassicPreset.Output(reteSocket, "Scope"));

    if (this.addNode && this.addConnection) {
      this.functionScope = new FunctionScopeNode("Function Scope", this.update);
      this.addNode(this.functionScope);
      this.addConnection(this, "value", this.functionScope, "context");
      this.functionScope.retNode.def = typeToValueCode(
        this.retControl.selected ?? "i32",
      );
    }

    addCustomFunction(this);
  }

  data(): { value: NodeReturn } {
    const result = {
      value: {
        value: this.nControl.value ?? 0,
        code: "fn " + this.nameControl.value + "(",
        refId: this.nameControl.value,
      },
    };

    if (this.nControl.value && this.nControl.value >= 1) {
      result.value.code += this.paramControls
        .map((value) => `${value.key}: ${value.cont.selected ?? "i32"}`)
        .join(", ");
    }

    result.value.code += ") -> " + this.retControl.selected + ` {`;

    return result;
  }
}

export class CallCustomFunctionNode extends VPNode {
  private funcControl: DropdownControl;
  constructor() {
    super("Call Custom Function");
    this.funcControl = new DropdownControl(
      getCustomFunctionOptions(),
      undefined,
      "Select custom function",
      "Function to call",
    );

    subscribe((change) => this.funcControl.setOptions(change));

    this.addControl("func", this.funcControl);
  }

  data(): { value: NodeReturn } {
    const func = this.funcControl.selected
      ? getCustomFunction(this.funcControl.selected)
      : undefined;
    if (!func) return { value: { value: 0, code: "" } };
    return {
      value: {
        value: 0,
        code: `${nodeToVariableDeclaration(this)} = ${func.label}();`,
        refId: idToVariableName(this.id),
      },
    };
  }
}

export function typeToValueCode(type: string): string {
  switch (type) {
    case "i32":
      return "0";
    case "f32":
      return "0.0";
    case "vec2f":
      return "vec2f(0.0, 0.0)";
    case "vec3f":
      return "vec3f(0.0, 0.0, 0.0)";
    case "vec4f":
      return "vec4f(0.0, 0.0, 0.0, 0.0)";
    default:
      return "";
  }
}
