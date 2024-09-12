import { ClassicPreset } from "rete";
import { Area, AreaPlugin } from "rete-area-plugin";
import { type SerializedNode } from "@/vpnodes/serialization/node";

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
  left: number,
  right: number,
  op: "+" | "-" | "/" | "*" | "%",
): number {
  let result = left;

  if (op === "+") result += right;
  else if (op === "-") result -= right;
  else if (op === "/") result /= right === 0 ? 1 : right;
  else if (op === "*") result *= right;
  else result %= right;

  return result;
}

export class NothingNode extends ClassicPreset.Node {
  width = 0;
  height = 0;
  parent?: string;

  constructor() {
    super("");
  }

  data(): {} {
    return {};
  }

  updateSize(area?: AreaPlugin<any, any>) {}
  serialize(sn: SerializedNode) {
    return sn;
  }

  deserialize(sn: SerializedNode) {}
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
    this.height =
      140 +
      (this.extraHeight ?? 0) +
      (20 + (this.extraHeightSockets ?? 0)) *
        (Object.keys(this.inputs).length + Object.keys(this.outputs).length) +
      (30 + (this.extraHeightControls ?? 0)) *
        Object.keys(this.controls).length;
    if (area) area.update("node", this.id);
  }

  serialize(sn: SerializedNode) {
    sn.size = [this.width, this.height];
    sn.parent = this.parent;
    sn.uuid = this.id;
    return sn;
  }

  deserialize(sn: SerializedNode) {}
}

export class NodeReturn {
  constructor(
    public value: any,
    public code: string,
    public refId?: string,
  ) {}
}

export class ReturnNode extends VPNode {
  constructor(
    public def: any,
    customName?: string,
  ) {
    super(customName ?? "Return");

    this.addInput(
      "returnIn",
      new ClassicPreset.Input(reteSocket, "Return Value"),
    );
  }

  data(inputs: { returnIn: NodeReturn[] }): { value: NodeReturn } {
    const result = {
      value: {
        value: this.def,
        code: "return " + this.def.toString() + ";",
      },
    };
    const { returnIn } = inputs;

    if (returnIn) {
      result.value.value = returnIn[0].value;
      result.value.code =
        "return " + (returnIn[0].refId ?? returnIn[0].value.toString()) + ";";
    }

    return result;
  }

  serialize(sn: SerializedNode) {
    sn.nodeType = "Return";
    return super.serialize(sn);
  }
}

export class VariableOutNode extends VPNode {
  constructor(
    public value: any,
    private code: any,
    public ref?: string,
  ) {
    super(ref ?? "Variable");

    this.addOutput("value", new ClassicPreset.Output(reteSocket, "out"));
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
    super.deserialize(sn);
  }
}

export class VariableInNode extends VPNode {
  constructor(public ref: string) {
    super(ref);

    this.addInput("value", new ClassicPreset.Input(reteSocket, "set"));
  }

  data(inputs: { value: NodeReturn[] }): { value: NodeReturn } {
    const { value } = inputs;

    return {
      value: {
        value: value ? value[0].value : 0,
        code: `${this.ref} = ${value ? value[0].refId ?? value[0].value : this.ref};`,
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
    private numParams?: 0 | 1 | 2 | 3 | 4,
  ) {
    super(functionName ?? "Function Call");

    if (!functionName) {
      this.addControl(
        "functionName",
        new ClassicPreset.InputControl("text", { initial: "abs" }),
      );

      this.addControl(
        "numParams",
        new ClassicPreset.InputControl("number", { initial: 1 }),
      );
    }

    for (let i = 1; i < (numParams ?? 1) + 1; i++) {
      this.addInput(
        "param" + i.toString(),
        new ClassicPreset.Input(reteSocket, "Param " + i.toString()),
      );
    }

    this.addOutput("value", new ClassicPreset.Output(reteSocket, "Result"));
  }

  data(inputs: {
    param1: NodeReturn[];
    param2: NodeReturn[];
    param3: NodeReturn[];
    param4: NodeReturn[];
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
          (param1 ? param1[0].refId ?? param1[0].value : "0.0") + ", ";
      }
      if (this.numParams >= 2) {
        result.value.code +=
          (param2 ? param2[0].refId ?? param2[0].value : "0.0") + ", ";
      }
      if (this.numParams >= 3) {
        result.value.code +=
          (param3 ? param3[0].refId ?? param3[0].value : "0.0") + ", ";
      }
      if (this.numParams === 4) {
        result.value.code +=
          (param4 ? param4[0].refId ?? param4[0].value : "0.0") + ", ";
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
      new ClassicPreset.InputControl("text", { initial: "vec3f(0, 0, 0)" }),
    );
    this.addOutput("value", new ClassicPreset.Output(reteSocket, "Value"));
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
          ? valueCont.value ?? "vec3f(0, 0, 0)"
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
