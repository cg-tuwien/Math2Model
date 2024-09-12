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
  notify,
  removeCustomFunction,
  subscribe,
  typeOptions,
} from "@/vpnodes/controls/dropdown-options";
import { BlockNode } from "@/vpnodes/basic/logic";
import type { Nodes } from "@/components/visual-programming/CodeGraph.vue";
import { vec2, vec3, vec4 } from "webgpu-matrix";

export class FunctionScopeNode extends BlockNode {
  public retNode: ReturnNode = new ReturnNode(0);
  public paramNodes: VariableOutNode[] = [];
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
  public nameControl: ClassicPreset.InputControl<"text", string>;
  public nControl: ClassicPreset.InputControl<"number", number>;
  public retControl: DropdownControl;
  public paramControls: { key: string; cont: DropdownControl }[] = [];
  public functionScope = new FunctionScopeNode("Function Scope");

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
        value = Math.min(9, value);
        value = Math.max(0, value);
        this.nControl.value = value;

        for (let i = 0; i < 9; i++) {
          if (i < value) {
            if (this.hasControl("arg" + i.toString())) continue;
            this.addControl(
              this.paramControls[i].key,
              this.paramControls[i].cont,
            );
            this.functionScope.paramNodes[i].parent = this.functionScope.id;
            if (this.addNode) this.addNode(this.functionScope.paramNodes[i]);
          } else {
            if (!this.hasControl("arg" + i.toString())) continue;
            this.removeControl("arg" + i.toString());
            if (this.removeNode)
              this.removeNode(this.functionScope.paramNodes[i]);
          }
        }

        if (this.update) this.update(this);
        this.extraHeightControls = 2.5 * value;
        this.updateSize();
        notify();
      },
    });

    this.retControl = new DropdownControl(
      typeOptions,
      "",
      "Return type",
      "return type",
      (s, l) => {
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

    for (let i = 0; i < 9; i++) {
      this.paramControls.push({
        key: "arg" + i.toString(),
        cont: new DropdownControl(
          typeOptions,
          "i32",
          "Parameter Type",
          "arg" + i.toString() + " type",
          (select, label) => {
            if (this.update) this.update(this);
            for (let i = 0; i < this.paramControls.length; i++) {
              if (this.paramControls[i].cont.label === label) {
                this.functionScope.paramNodes[i].value = typeToValue(select);
                return;
              }
            }
          },
        ),
      });

      const paramNode = new VariableOutNode(0, "", "arg" + i.toString());
      this.functionScope.paramNodes.push(paramNode);
    }

    addCustomFunction(this);
  }

  data(): { value: NodeReturn } {
    const result = {
      value: {
        value: typeToValue(this.retControl.selected ?? ""),
        code: "fn " + this.nameControl.value + "(",
        refId: this.nameControl.value,
      },
    };

    if (this.nControl.value && this.nControl.value >= 1) {
      result.value.code += this.paramControls
        .slice(0, this.nControl.value)
        .map((value) => `${value.key}: ${value.cont.selected ?? "i32"}`)
        .join(", ");
    }

    result.value.code += ") -> " + this.retControl.selected + ` {`;

    return result;
  }
}

export class CallCustomFunctionNode extends VPNode {
  private funcControl: DropdownControl;
  private argInputs: ClassicPreset.Input<any>[] = [];
  private argN: number = 0;
  constructor(private update?: (node: ClassicPreset.Node) => void) {
    super("Call Custom Function");

    for (let i = 0; i < 9; i++) {
      this.argInputs.push(
        new ClassicPreset.Input(reteSocket, "arg" + i.toString()),
      );
    }

    this.funcControl = new DropdownControl(
      getCustomFunctionOptions(),
      undefined,
      "Select custom function",
      "Function to call",
      (select) => this.updateInputs(select),
    );

    subscribe((change) => {
      this.funcControl.setOptions(change);
      this.updateInputs(this.funcControl.selected ?? change[0].label);
    });

    this.addControl("func", this.funcControl);
    this.addOutput(
      "value",
      new ClassicPreset.Output(reteSocket, "Function return"),
    );
    this.updateInputs(this.funcControl.selected ?? "");
  }

  updateInputs(this: CallCustomFunctionNode, select: string) {
    const func = getCustomFunction(select);
    if (!func) return;
    this.argN = func.nControl.value ?? this.argN;
    for (let i = 0; i < 9; i++) {
      if (i < this.argN) {
        if (this.hasInput("arg" + i.toString())) continue;
        this.addInput("arg" + i.toString(), this.argInputs[i]);
      } else {
        if (this.hasInput("arg" + i.toString()))
          this.removeInput("arg" + i.toString());
      }
    }
    if (this.update) this.update(this);
    this.extraHeightSockets = this.argN * 1.5;
    this.updateSize();
  }

  data(input: {
    arg0: NodeReturn[];
    arg1: NodeReturn[];
    arg2: NodeReturn[];
    arg3: NodeReturn[];
    arg4: NodeReturn[];
    arg5: NodeReturn[];
    arg6: NodeReturn[];
    arg7: NodeReturn[];
    arg8: NodeReturn[];
  }): { value: NodeReturn } {
    const func = this.funcControl.selected
      ? getCustomFunction(this.funcControl.selected)
      : undefined;
    const { arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8 } = input;
    const args = [arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8];
    if (!func) return { value: { value: 0, code: "" } };
    const result = {
      value: {
        value: typeToValue(func.retControl.selected ?? ""),
        code: `${nodeToVariableDeclaration(this)} = ${func.label}(`,
        refId: idToVariableName(this.id),
      },
    };

    if (func.nControl.value) {
      for (let i = 0; i < func.nControl.value; i++) {
        const arg = args[i];
        if (arg && arg.length > 0) {
          result.value.code += arg[0].refId ?? arg[0].value;
        } else {
          result.value.code += typeToValueCode(
            func.paramControls[i].cont.selected ?? "i32",
          );
        }

        if (i < func.nControl.value - 1) result.value.code += ", ";
      }
    }
    result.value.code += ");";

    return result;
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

export function typeToValue(type: string) {
  switch (type) {
    case "i32":
      return 0;
    case "f32":
      return 0.0;
    case "vec2f":
      return vec2.create(0.0, 0.0);
    case "vec3f":
      return vec3.create(0.0, 0.0, 0.0);
    case "vec4f":
      return vec4.create(0.0, 0.0, 0.0, 0.0);
    default:
      return undefined;
  }
}
