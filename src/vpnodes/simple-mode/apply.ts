import {
  idToVariableName,
  NodeReturn,
  nodeToVariableDeclaration,
  reteSocket,
  VPNode,
} from "@/vpnodes/basic/nodes";
import { SliderControl } from "@/vpnodes/controls/slider";
import { ClassicPreset } from "rete";
import { vec2, vec3 } from "webgpu-matrix";
import { type SerializedNode } from "@/vpnodes/serialization/node";
import {
  typeToValueCode,
  typeToValueStringCode,
  valueToType,
} from "@/vpnodes/basic/functions";
import type { Nodes } from "../nodes-list";

export class CombineNode extends VPNode {
  private cfControl: SliderControl;
  constructor(
    private update: (id: string) => void,
    private updateControl: (c: SliderControl) => void,
    private noDebounceUpdate: (id: string) => void
  ) {
    super("Combine Shapes");

    this.cfControl = new SliderControl(
      0.5,
      1.0,
      0.0,
      0.01,
      "Combine Factor",
      false,
      () => {
        this.update(this.id);
      },
      (value) => {
        this.updateControl(this.cfControl);
      },
      () => {
        this.noDebounceUpdate(this.id);
      }
    );

    this.addInput(
      "param1",
      new ClassicPreset.Input(reteSocket, "shape 1/vec3f")
    );
    this.addInput(
      "param2",
      new ClassicPreset.Input(reteSocket, "shape 2/vec3f")
    );
    this.addInput(
      "param3",
      new ClassicPreset.Input(reteSocket, "combine factor/f32")
    );
    this.addControl("cfactor", this.cfControl);
    this.addOutput(
      "value",
      new ClassicPreset.Output(reteSocket, "output[x, y, z]/vec3f")
    );
  }

  data(inputs: {
    param1: NodeReturn[];
    param2: NodeReturn[];
    param3: NodeReturn[];
  }): {
    value: NodeReturn;
  } {
    const { param1, param2, param3 } = inputs;
    const p1 = param1 ? param1[0].refId : "vec3f(0.0, 0.0, 0.0)";
    const p2 = param2 ? param2[0].refId : "vec3f(0.0, 0.0, 0.0)";
    const p3 = param3 ? param3[0].refId : this.cfControl.sliderValue;
    if (param3 && this.hasControl("cfactor")) {
      this.removeControl("cfactor");
    } else if (!param3 && !this.hasControl("cfactor")) {
      this.addControl("cfactor", this.cfControl);
    }

    // if (this.update) this.update(this.id);

    return {
      value: {
        value: vec3.zero(),
        code: `${nodeToVariableDeclaration(this)} = mix(${p1}, ${p2}, ${p3});`,
        refId: idToVariableName(this.id),
      },
    };
  }

  serialize(sn: SerializedNode): SerializedNode {
    sn.nodeType = "Combine";
    sn.extraNumberInformation = [
      { key: "cf", value: this.cfControl.value ?? 0 },
    ];
    return super.serialize(sn);
  }

  deserialize(sn: SerializedNode) {
    super.deserialize(sn);
    if (sn.extraNumberInformation) {
      for (let info of sn.extraNumberInformation) {
        if (info.key == "cf") {
          this.cfControl.value = info.value;
        }
      }
    }
  }

  clone(): Nodes | void {
    const cn = new CombineNode(
      this.update,
      this.updateControl,
      this.noDebounceUpdate
    );
    cn.cfControl.value = this.cfControl.value;
    return cn;
  }
}

export class MathFunctionNode extends VPNode {
  private variableControls: Map<string, SliderControl> = new Map();
  constructor(
    private name: string,
    private func: string,
    private update: (id: string) => void,
    private updateControl: (c: SliderControl) => void,
    private noDebounceUpdate: (id: string) => void,
    private isApply: boolean = true,
    private inputType: "f32" | "vec2f" | "vec3f" | "vec4f" | "any" = "any",
    private outputType: "f32" | "vec2f" | "vec3f" | "vec4f" | "any" = "any"
  ) {
    super(`${isApply ? "Apply" : "Calculate"} ${name} Function`);

    this.setup();
  }

  private setup() {
    this.label = `${this.isApply ? "Apply" : "Calculate"} ${this.name} Function`;

    if (this.hasInput("param")) this.removeInput("param");
    if (this.hasOutput("value")) this.removeOutput("value");

    this.addInput(
      "param",
      new ClassicPreset.Input(reteSocket, `param / ${this.inputType}`)
    );

    this.addOutput(
      "value",
      new ClassicPreset.Output(reteSocket, `result / ${this.outputType}`)
    );

    const vars = this.func.split("{");
    for (let variable of vars) {
      const expr = variable.split("}")[0].split(",");
      if (expr.length < 6) continue;
      if (this.hasControl(expr[0])) continue;
      const control = new SliderControl(
        parseFloat(expr[1]),
        parseFloat(expr[2]),
        parseFloat(expr[3]),
        parseFloat(expr[4]),
        expr[0],
        true,
        () => this.update(this.id),
        (value) => this.updateControl(control),
        () => this.noDebounceUpdate(this.id)
      );
      this.variableControls.set(
        "{" + variable.split("}")[0] + "}/" + expr[5],
        control
      );
      this.addControl(expr[0], control);
    }

    this.extraHeightControls = 40;
    this.updateSize();
  }

  data(inputs: { param: NodeReturn[] }): { value: NodeReturn } {
    let funcCall = this.func;
    let { param } = inputs;

    funcCall = funcCall.replaceAll(
      "input2",
      String(
        param
          ? (param[0].refId ?? param[0].value)
          : this.inputType == "any"
            ? "input2"
            : typeToValueCode(this.inputType)
      )
    );
    for (let key of this.variableControls.keys()) {
      const k = key.split("/");
      const control = this.variableControls.get(key);
      if (k[1] === "same") {
        funcCall = funcCall.replaceAll(
          k[0],
          typeToValueStringCode(
            valueToType(param ? param[0].value : vec2.create(0.0, 0.0)),
            control?.sliderValue ?? "0.0",
            control?.sliderValue ?? "0.0",
            control?.sliderValue ?? "0.0",
            control?.sliderValue ?? "0.0"
          )
        );
      } else {
        funcCall = funcCall.replaceAll(
          k[0],
          typeToValueStringCode(
            k[1],
            control?.sliderValue ?? "0.0",
            control?.sliderValue ?? "0.0",
            control?.sliderValue ?? "0.0",
            control?.sliderValue ?? "0.0"
          )
        );
      }
    }

    var code = `${this.isApply ? nodeToVariableDeclaration(this) + "_1" : nodeToVariableDeclaration(this)} = ${funcCall};`;
    if (this.isApply)
      code += `\n\t${nodeToVariableDeclaration(this)} = vec3f(${param ? param[0].refId + ".x" : 0.0}, ${idToVariableName(this.id)}_1.x * ${idToVariableName(this.id)}_1.y, ${param ? param[0].refId + ".z" : 0.0});`;

    return {
      value: {
        value: param ? param[0].value : 0.0,
        code: code,
        refId: idToVariableName(this.id),
      },
    };
  }

  serialize(sn: SerializedNode): SerializedNode {
    sn.nodeType = "MathFunction";
    sn.extraStringInformation = [
      { key: "name", value: this.name },
      { key: "func", value: this.func },
      { key: "inputType", value: this.inputType },
      { key: "outputType", value: this.outputType },
    ];
    sn.extraNumberInformation = [];
    for (let key of this.variableControls.keys()) {
      sn.extraNumberInformation.push({
        key: key,
        value: this.variableControls.get(key)?.value ?? 0.0,
      });
    }
    sn.inputs.push({
      key: "isApply",
      value: this.isApply ? 1 : 0,
      type: "number",
    });
    return super.serialize(sn);
  }

  deserialize(sn: SerializedNode) {
    if (sn.extraStringInformation) {
      for (let info of sn.extraStringInformation) {
        if (info.key === "name") {
          this.name = info.value;
        }
        if (info.key === "func") {
          this.func = info.value;
        }
        if (info.key === "inputType") {
          this.inputType = info.value as
            | "f32"
            | "vec2f"
            | "vec3f"
            | "vec4f"
            | "any";
        }
        if (info.key === "outputType") {
          this.outputType = info.value as
            | "f32"
            | "vec2f"
            | "vec3f"
            | "vec4f"
            | "any";
        }
      }
    }
    for (let input of sn.inputs) {
      if (input.type === "number" && input.key === "isApply") {
        this.isApply = input.value == 1;
      }
    }
    this.setup();
    if (sn.extraNumberInformation) {
      for (let info of sn.extraNumberInformation) {
        if (this.variableControls.has(info.key)) {
          const cont = this.variableControls.get(info.key);
          if (cont) cont.value = info.value;
        }
      }
    }
    super.deserialize(sn);
  }

  clone(): Nodes | void {
    const mfn = new MathFunctionNode(
      this.name,
      this.func,
      this.update,
      this.updateControl,
      this.noDebounceUpdate,
      this.isApply,
      this.inputType,
      this.outputType
    );
    for (let key of this.variableControls.keys()) {
      const k = key.split("/");
      const control = this.variableControls.get(key);
      const otherControl = mfn.variableControls.get(key);
      if (otherControl && control) {
        otherControl.value = control.value;
      }
    }
    return mfn;
  }
}
