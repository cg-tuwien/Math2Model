import { ClassicPreset } from "rete";
import { Area, AreaPlugin } from "rete-area-plugin";
import { type SerializedNode } from "@/vpnodes/serialization/node";

import { vec2, vec3, vec4 } from "webgpu-matrix";
import type { Nodes } from "../nodes-list";

export const reteSocket = new ClassicPreset.Socket("socket");

export function nodeToVariableDeclaration(node: ClassicPreset.Node) {
  return "var " + idToVariableName(node.id);
}

export function idToVariableName(id: string): string {
  return `ref_${id.substring(0, 5)}`;
}

export function opToName(op: "+" | "-" | "/" | "*" | "%"): string {
  return op === "+"
    ? "Add"
    : op === "-"
      ? "Subtract"
      : op === "/"
        ? "Divide"
        : op === "*"
          ? "Multiply"
          : "Modulo";
}

export function applyOperator(
  left: any,
  right: any,
  op: "+" | "-" | "/" | "*" | "%"
): any {
  let result = left;

  if (valueToType(left) !== "f32") {
    return left;
  }

  if (valueToType(right) !== "f32") {
    return right;
  }

  if (op === "+") result += right;
  else if (op === "-") result -= right;
  else if (op === "/") result /= right === 0 ? 1 : right;
  else if (op === "*") result *= right;
  else result %= right;

  return result;
}

export class VPNode extends ClassicPreset.Node {
  width = 180;
  height = 140;
  parent?: string;
  extraWidth?: number;
  extraHeight?: number;
  extraHeightSockets?: number;
  extraHeightControls?: number;

  updateSize(area?: AreaPlugin<any, any>) {
    this.width = 180 + (this.extraWidth ?? 0);
    this.height = 140 + (this.extraHeight ?? 0);
    if (this.inputs && this.outputs)
      this.height +=
        (20 + (this.extraHeightSockets ?? 0)) *
        (Object.keys(this.inputs).length + Object.keys(this.outputs).length);
    if (this.controls)
      this.height +=
        (30 + (this.extraHeightControls ?? 0)) *
        Object.keys(this.controls).length;
    if (area) area.update("node", this.id);
  }

  serialize(sn: SerializedNode) {
    sn.parent = this.parent;
    sn.uuid = this.id;
    return sn;
  }

  deserialize(sn: SerializedNode) {}

  clone(): Nodes | void {}

  data(input: Partial<NodeData>): NodeData {
    return {};
  }
}

export class NodeReturn {
  constructor(
    public value: any,
    public code: string,
    public refId?: string
  ) {}
}

export type NodeData = { [key: string]: NodeReturn };

export class ReturnNode extends VPNode {
  constructor(
    public def: any,
    private customName?: string
  ) {
    super(customName ?? "Return");

    this.addInput(
      "returnIn",
      new ClassicPreset.Input(reteSocket, "Return Value")
    );
  }

  data(inputs: { returnIn?: NodeReturn }): { value: NodeReturn } {
    const result = {
      value: {
        value: this.def,
        code: "return " + this.def.toString() + ";",
      },
    };
    const { returnIn } = inputs;

    if (returnIn) {
      result.value.value = returnIn.value;
      result.value.code =
        "return " + (returnIn.refId ?? returnIn.value.toString()) + ";";
    }

    return result;
  }

  serialize(sn: SerializedNode) {
    sn.nodeType = "Return";
    sn.inputs.push({ key: "def", value: this.def, type: "text" });
    sn.inputs.push({
      key: "customName",
      value: this.customName ?? "Return",
      type: "text",
    });
    return super.serialize(sn);
  }

  deserialize(sn: SerializedNode) {
    for (let input of sn.inputs) {
      if (input.type === "text" && input.key === "def") this.def = input.value;
      if (input.type === "text" && input.key === "customName") {
        this.label = input.value;
      }
    }
    super.deserialize(sn);
  }
}

export class VariableOutNode extends VPNode {
  constructor(
    public value: any,
    public code: any,
    public ref?: string
  ) {
    super(ref ?? "Variable");

    this.addOutput("value", new ClassicPreset.Output(reteSocket, "out"));
    if (this.ref && this.ref.length > 15) this.extraWidth = this.ref.length * 5;
    this.updateSize();
  }

  data(): { value: NodeReturn } {
    return {
      value: {
        value: this.value,
        code: this.code,
        refId: this.ref,
      },
    };
  }

  serialize(sn: SerializedNode) {
    sn.nodeType = "VariableOut";
    sn.extraNumberInformation = [{ key: "value", value: this.value }];
    sn.extraStringInformation = [{ key: "code", value: this.code }];
    if (this.ref)
      sn.extraStringInformation.push({ key: "ref", value: this.ref });
    return super.serialize(sn);
  }

  deserialize(sn: SerializedNode) {
    if (sn.extraNumberInformation) {
      for (let info of sn.extraNumberInformation) {
        if (info.key === "value") this.value = info.value;
      }
    }
    if (sn.extraStringInformation) {
      for (let info of sn.extraStringInformation) {
        if (info.key === "code") this.code = info.value;
        if (info.key === "ref") {
          this.ref = info.value;
          this.label = info.value;
        }
      }
    }
    if (this.ref && this.ref.length > 15) this.extraWidth = this.ref.length * 5;
    super.deserialize(sn);
  }
}

export class VariableInNode extends VPNode {
  constructor(public ref: string) {
    super(ref);

    this.addInput("value", new ClassicPreset.Input(reteSocket, "set"));
    this.updateSize();
  }

  data(inputs: { value?: NodeReturn }): { value: NodeReturn } {
    const { value } = inputs;

    return {
      value: {
        value: value ? value.value : 0,
        code: `${this.ref} = ${value ? (value.refId ?? value.value) : this.ref};`,
        refId: this.ref,
      },
    };
  }

  serialize(sn: SerializedNode) {
    sn.nodeType = "VariableIn";
    sn.extraStringInformation = [{ key: "ref", value: this.ref }];
    return super.serialize(sn);
  }

  deserialize(sn: SerializedNode) {
    if (sn.extraStringInformation) {
      for (let info of sn.extraStringInformation) {
        if (info.key === "ref") this.ref = info.value;
      }
    }
    super.deserialize(sn);
  }
}

export class FunctionCallNode extends VPNode {
  constructor(
    private functionName?: string,
    private numParams?: 0 | 1 | 2 | 3 | 4
  ) {
    super(functionName ?? "Function Call");

    if (!functionName) {
      this.addControl(
        "functionName",
        new ClassicPreset.InputControl("text", { initial: "abs" })
      );

      this.addControl(
        "numParams",
        new ClassicPreset.InputControl("number", { initial: 1 })
      );
    }

    for (let i = 1; i < (numParams ?? 0) + 1; i++) {
      this.addInput(
        "param" + i.toString(),
        new ClassicPreset.Input(reteSocket, "Param " + i.toString())
      );
    }

    this.addOutput("value", new ClassicPreset.Output(reteSocket, "Result"));
    this.updateSize();
  }

  data(inputs: {
    param1?: NodeReturn;
    param2?: NodeReturn;
    param3?: NodeReturn;
    param4?: NodeReturn;
  }): { value: NodeReturn } {
    const result = {
      value: {
        value: 0,
        code: nodeToVariableDeclaration(this) + " = " + this.functionName + "(",
        refId: idToVariableName(this.id),
      },
    };
    const { param1, param2, param3, param4 } = inputs;

    if (this.numParams) {
      if (this.numParams >= 1) {
        result.value.code +=
          (param1 ? (param1.refId ?? param1.value) : "0.0") + ", ";
      }
      if (this.numParams >= 2) {
        result.value.code +=
          (param2 ? (param2.refId ?? param2.value) : "0.0") + ", ";
      }
      if (this.numParams >= 3) {
        result.value.code +=
          (param3 ? (param3.refId ?? param3.value) : "0.0") + ", ";
      }
      if (this.numParams === 4) {
        result.value.code +=
          (param4 ? (param4.refId ?? param4.value) : "0.0") + ", ";
      }
    }

    result.value.code =
      result.value.code.substring(0, result.value.code.length - 2) + ");";

    return result;
  }

  serialize(sn: SerializedNode) {
    sn.nodeType = "FunctionCall";
    sn.extraStringInformation = [
      { key: "function", value: this.functionName ?? "" },
    ];
    sn.extraNumberInformation = [
      { key: "nParams", value: this.numParams ?? 0 },
    ];
    return super.serialize(sn);
  }

  deserialize(sn: SerializedNode) {
    if (sn.extraNumberInformation) {
      for (let info of sn.extraNumberInformation) {
        if (info.key === "nParams") {
          if (info.value === 0) {
            this.numParams = 0;
          } else if (info.value === 1) {
            this.numParams = 1;
          } else if (info.value === 2) {
            this.numParams = 2;
          } else if (info.value === 3) {
            this.numParams = 3;
          } else if (info.value === 4) {
            this.numParams = 4;
          } else {
            this.numParams = undefined;
          }
        }
      }

      for (let i = 1; i < (this.numParams ?? 0) + 1; i++) {
        if (!this.hasInput("param" + i.toString())) {
          this.addInput(
            "param" + i.toString(),
            new ClassicPreset.Input(reteSocket, "Param " + i.toString())
          );
        }
      }
    }
    if (sn.extraStringInformation) {
      for (let info of sn.extraStringInformation) {
        if (info.key === "function") {
          this.functionName = info.value;
          this.label = info.value;

          if (this.hasControl("functionName")) {
            this.removeControl("functionName");
          }

          if (this.hasControl("numParams")) {
            this.removeControl("numParams");
          }
        }
      }
    }
    super.deserialize(sn);
  }
}

export class InitializeNode extends VPNode {
  constructor() {
    super("Initialize");

    this.addControl(
      "value",
      new ClassicPreset.InputControl("text", { initial: "vec3f(0, 0, 0)" })
    );
    this.addOutput("value", new ClassicPreset.Output(reteSocket, "Value"));
    this.updateSize();
  }

  data(): { value: NodeReturn } {
    const valueCont: ClassicPreset.InputControl<"text", string> = this.controls
      .value as ClassicPreset.InputControl<"text", string>;
    if (!valueCont) return { value: { value: 0, code: "" } };

    return {
      value: {
        value: 0,
        code: `${nodeToVariableDeclaration(this)} = ${valueCont.value};`,
        refId: idToVariableName(this.id),
      },
    };
  }

  serialize(sn: SerializedNode) {
    sn.nodeType = "Initialize";
    const valueCont: ClassicPreset.InputControl<"text", string> = this.controls
      .value as ClassicPreset.InputControl<"text", string>;
    sn.inputs = [
      {
        type: "text",
        value: valueCont
          ? (valueCont.value ?? "vec3f(0, 0, 0)")
          : "vec3f(0, 0, 0)",
        key: "value",
      },
    ];
    return super.serialize(sn);
  }

  deserialize(sn: SerializedNode) {
    const valueCont: ClassicPreset.InputControl<"text", string> = this.controls
      .value as ClassicPreset.InputControl<"text", string>;
    if (valueCont && sn.inputs) {
      for (let info of sn.inputs) {
        if (info.type === "text" && info.key === "value")
          valueCont.setValue(info.value);
      }
    }
    super.deserialize(sn);
  }
}

export class InstanceCountNode extends VPNode {
  constructor(
    public modelId: string,
    public name?: string
  ) {
    super(name ?? "Instance Count");

    this.addOutput("value", new ClassicPreset.Output(reteSocket, "out"));
    if (this.name && this.name.length > 15)
      this.extraWidth = this.name.length * 5;
    this.updateSize();
  }

  data(): { value: NodeReturn } {
    return {
      value: {
        value: 0.0,
        code: nodeToVariableDeclaration(this) + " = ",
        refId: idToVariableName(this.id),
      },
    };
  }

  serialize(sn: SerializedNode) {
    sn.nodeType = "InstanceCount";
    sn.extraStringInformation = [{ key: "modelId", value: this.modelId }];
    if (this.name)
      sn.extraStringInformation.push({ key: "name", value: this.name });
    return super.serialize(sn);
  }

  deserialize(sn: SerializedNode) {
    if (sn.extraStringInformation) {
      for (let info of sn.extraStringInformation) {
        if (info.key === "modelId") this.modelId = info.value;
        if (info.key === "name") {
          this.name = info.value;
          this.label = info.value;
        }
      }
    }
    if (this.name && this.name.length > 15)
      this.extraWidth = this.name.length * 5;
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
