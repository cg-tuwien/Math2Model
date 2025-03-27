import {
  idToVariableName,
  NodeReturn,
  nodeToVariableDeclaration,
  reteSocket,
  ReturnNode,
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
import { vec2, vec3, vec4 } from "webgpu-matrix";
import { type SerializedNode } from "@/vpnodes/serialization/node";
import type { Nodes } from "../nodes-list";

export class FunctionScopeNode extends BlockNode {
  public retNode: ReturnNode = new ReturnNode(0);
  public paramNodes: VariableOutNode[] = [];
  constructor(
    private name: string,
    private update?: (node: ClassicPreset.Node) => void
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

  serialize(sn: SerializedNode): SerializedNode {
    sn.nodeType = "FunctionScope";
    sn.extraStringInformation = [{ key: "name", value: this.name }];
    return super.serialize(sn);
  }

  deserialize(sn: SerializedNode) {
    if (sn.extraStringInformation) {
      for (let info of sn.extraStringInformation) {
        if (info.key === "name") {
          this.name = info.value;
          this.label = this.name;
        }
      }
    }
    super.deserialize(sn);
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
      keyB: string
    ) => void,
    private addNodeSelf?: boolean
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
        removeCustomFunction(this);

        for (let i = 0; i < 9; i++) {
          if (i < value) {
            if (this.hasControl("arg" + i.toString())) continue;
            this.addControl(
              this.paramControls[i].key,
              this.paramControls[i].cont
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
        addCustomFunction(this);
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
      }
    );
    this.addControl("name", this.nameControl);
    this.addControl("n", this.nControl);

    this.addControl("ret", this.retControl);

    this.addOutput("value", new ClassicPreset.Output(reteSocket, "Scope"));

    if (this.addNodeSelf && this.addNode && this.addConnection) {
      this.functionScope = new FunctionScopeNode("Function Scope", this.update);
      this.addNode(this.functionScope);
      this.addConnection(this, "value", this.functionScope, "context");
      this.functionScope.retNode.def = typeToValueCode(
        this.retControl.selected ?? "i32"
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
          }
        ),
      });

      const paramNode = new VariableOutNode(0, "", this.paramControls[i].key);
      this.functionScope.paramNodes.push(paramNode);
    }

    addCustomFunction(this);
  }

  setParamName(i: number, name: string): void {
    if (i >= 0 && i < this.functionScope.paramNodes.length) {
      this.functionScope.paramNodes[i].ref = name;
      this.paramControls[i].cont.label = name + " type";
      this.paramControls[i].key = name;
      if (this.update) this.update(this);
    }
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

  serialize(sn: SerializedNode): SerializedNode {
    sn.nodeType = "CustomFunction";

    sn.inputs = [
      { key: "name", value: this.nameControl.value ?? "", type: "text" },
      { key: "n", value: this.nControl.value ?? 0, type: "number" },
      { key: "ret", value: this.retControl.selected ?? "i32", type: "text" },
    ];

    sn.extraStringInformation = [];
    for (let i = 0; i < 9; i++) {
      //console.log(
      //  "Serializing arg",
      //  i,
      //  "selected",
      //  this.paramControls[i].cont.selected,
      //);
      sn.inputs.push({
        key: "arg" + i.toString(),
        value: this.paramControls[i].cont.selected ?? "i32",
        type: "text",
      });
      sn.extraStringInformation.push({
        key: "arg" + i.toString(),
        value: this.paramControls[i].key,
      });
    }

    return super.serialize(sn);
  }

  deserialize(sn: SerializedNode) {
    for (let info of sn.inputs) {
      if (info.type === "text" && info.key === "name") {
        removeCustomFunction(this);
        this.nameControl.value = info.value;
        this.label = info.value;
        addCustomFunction(this);
      }
      if (info.type === "number" && info.key === "n")
        this.nControl.setValue(info.value);
      if (info.type === "text" && info.key === "ret")
        this.retControl.selected = info.value;
    }

    for (let i = 0; i < 9; i++) {
      const inp = sn.getTextOrNumberInput(this.paramControls[i].key);
      this.paramControls[i].cont.selected = inp ? inp.value.toString() : "i32";

      if (sn.extraStringInformation) {
        for (let info of sn.extraStringInformation) {
          if (info.key === this.paramControls[i].key)
            this.setParamName(i, info.value);
        }
      }
    }

    super.deserialize(sn);
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
        new ClassicPreset.Input(reteSocket, "arg" + i.toString())
      );
    }

    this.funcControl = new DropdownControl(
      getCustomFunctionOptions(),
      undefined,
      "Select custom function",
      "Function to call",
      (select) => this.updateInputs(select)
    );

    subscribe((change) => {
      this.funcControl.setOptions(change);
      this.updateInputs(this.funcControl.selected ?? change[0].label);
    });

    this.addControl("func", this.funcControl);
    this.addOutput(
      "value",
      new ClassicPreset.Output(reteSocket, "Function return")
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
            func.paramControls[i].cont.selected ?? "i32"
          );
        }

        if (i < func.nControl.value - 1) result.value.code += ", ";
      }
    }
    result.value.code += ");";

    return result;
  }

  serialize(sn: SerializedNode): SerializedNode {
    sn.nodeType = "CallCustomFunction";

    sn.inputs = [
      { key: "function", value: this.funcControl.selected ?? "", type: "text" },
      { key: "argN", value: this.argN, type: "number" },
    ];

    return super.serialize(sn);
  }

  deserialize(sn: SerializedNode) {
    for (let info of sn.inputs) {
      if (info.type === "text" && info.key === "function")
        this.funcControl.selected = info.value;
      if (info.type === "number" && info.key === "argN") this.argN = info.value;
    }

    super.deserialize(sn);
  }
}

export function typeToValueCode(
  type: string,
  valueX?: number,
  valueY?: number,
  valueZ?: number,
  valueW?: number
): string {
  switch (type) {
    case "i32":
      return valueX?.toString() ?? "0";
    case "f32":
      return valueX?.toFixed(20) ?? "0.0";
    case "vec2f":
      return `vec2f(${valueX?.toFixed(20) ?? "0.0"}, ${valueY?.toFixed(20) ?? "0.0"})`;
    case "vec3f":
      return `vec3f(${valueX?.toFixed(20) ?? "0.0"}, ${valueY?.toFixed(20) ?? "0.0"}, ${valueZ?.toFixed(20) ?? "0.0"})`;
    case "vec4f":
      return `vec4f(${valueX?.toFixed(20) ?? "0.0"}, ${valueY?.toFixed(20) ?? "0.0"}, ${valueZ?.toFixed(20) ?? "0.0"}, ${valueW?.toFixed(20) ?? "0.0"})`;
    default:
      return "";
  }
}

export function typeToValueStringCode(
  type: string,
  valueX?: string,
  valueY?: string,
  valueZ?: string,
  valueW?: string
): string {
  switch (type) {
    case "i32":
      return valueX?.toString() ?? "0";
    case "f32":
      return valueX ?? "0.0";
    case "vec2f":
      return `vec2f(${valueX ?? "0.0"}, ${valueY ?? "0.0"})`;
    case "vec3f":
      return `vec3f(${valueX ?? "0.0"}, ${valueY ?? "0.0"}, ${valueZ ?? "0.0"})`;
    case "vec4f":
      return `vec4f(${valueX ?? "0.0"}, ${valueY ?? "0.0"}, ${valueZ ?? "0.0"}, ${valueW ?? "0.0"})`;
    default:
      return "";
  }
}

export function typeToValue(
  type: string,
  valueX?: number,
  valueY?: number,
  valueZ?: number,
  valueW?: number
) {
  switch (type) {
    case "i32":
      return valueX ?? 0;
    case "f32":
      return valueX ?? 0.0;
    case "vec2f":
      return vec2.create(valueX ?? 0.0, valueY ?? 0.0);
    case "vec3f":
      return vec3.create(valueX ?? 0.0, valueY ?? 0.0, valueZ ?? 0.0);
    case "vec4f":
      return vec4.create(
        valueX ?? 0.0,
        valueY ?? 0.0,
        valueZ ?? 0.0,
        valueW ?? 0.0
      );
    default:
      return undefined;
  }
}

export function valueToType(value: number | Float32Array): string {
  if (value instanceof Float32Array) {
    return value.length == 2 ? "vec2f" : value.length == 3 ? "vec3f" : "vec4f";
  } else {
    return "f32";
  }
}
