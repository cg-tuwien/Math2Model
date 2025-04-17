import { ClassicPreset } from "rete";
import { vec2, vec3, vec4 } from "webgpu-matrix";
import {
  idToVariableName,
  NodeReturn,
  nodeToVariableDeclaration,
  reteSocket,
  VPNode,
} from "@/vpnodes/basic/nodes";
import { type SerializedNode } from "@/vpnodes/serialization/node";
import type { Nodes } from "../nodes-list";

export class VectorNode extends VPNode {
  private xControl: ClassicPreset.InputControl<"number", number>;
  private yControl: ClassicPreset.InputControl<"number", number>;
  private zControl: ClassicPreset.InputControl<"number", number>;
  private wControl: ClassicPreset.InputControl<"number", number>;
  constructor(
    private n: 2 | 3 | 4,
    private update?: (node: ClassicPreset.Node) => void
  ) {
    super("Vector" + n.toString());

    this.xControl = new ClassicPreset.InputControl("number", {
      initial: 0,
    });
    this.yControl = new ClassicPreset.InputControl("number", {
      initial: 0,
    });
    this.zControl = new ClassicPreset.InputControl("number", {
      initial: 0,
    });
    this.wControl = new ClassicPreset.InputControl("number", {
      initial: 0,
    });

    this.addControl("x", this.xControl);
    this.addInput("x", new ClassicPreset.Input(reteSocket, "X"));

    this.addControl("y", this.yControl);
    this.addInput("y", new ClassicPreset.Input(reteSocket, "Y"));

    if (n >= 3) {
      this.addControl("z", this.zControl);
      this.addInput("z", new ClassicPreset.Input(reteSocket, "Z"));
    }

    if (n === 4) {
      this.addControl("w", this.wControl);
      this.addInput("w", new ClassicPreset.Input(reteSocket, "W"));
    }

    this.addOutput("value", new ClassicPreset.Output(reteSocket));

    this.updateSize();
  }

  data(inputs: {
    x?: NodeReturn;
    y?: NodeReturn;
    z?: NodeReturn;
    w?: NodeReturn;
  }): { value: NodeReturn } {
    let xVal = this.xControl?.value ?? 0;
    let yVal = this.yControl?.value ?? 0;
    let zVal = this.zControl?.value ?? 0;
    let wVal = this.wControl?.value ?? 0;
    let xRef = "";
    let yRef = "";
    let zRef = "";
    let wRef = "";

    const { x, y, z, w } = inputs;

    if (x) {
      xVal = x.value ?? xVal;
      xRef = x.refId ?? "";
      if (this.hasControl("x")) this.removeControl("x");
    } else {
      if (!this.hasControl("x")) this.addControl("x", this.xControl);
    }
    if (y) {
      yVal = y.value ?? yVal;
      yRef = y.refId ?? "";
      if (this.hasControl("y")) this.removeControl("y");
    } else {
      if (!this.hasControl("y")) this.addControl("y", this.xControl);
    }
    if (z) {
      zVal = z.value ?? zVal;
      zRef = z.refId ?? "";
      if (this.hasControl("z")) this.removeControl("z");
    } else {
      if (!this.hasControl("z") && this.n >= 3)
        this.addControl("z", this.xControl);
    }
    if (w) {
      wVal = w.value ?? wVal;
      wRef = w.refId ?? "";
      if (this.hasControl("w")) this.removeControl("w");
    } else {
      if (!this.hasControl("w") && this.n === 4)
        this.addControl("w", this.xControl);
    }

    let vecResult = {
      value: vec2.create(xVal ?? 0, yVal ?? 0),
      code:
        nodeToVariableDeclaration(this) +
        " = " +
        "vec2f(" +
        (xRef == "" ? xVal.toString() : xRef) +
        ", " +
        (yRef == "" ? yVal.toString() : yRef) +
        ");",
      refId: idToVariableName(this.id),
    };
    if (this.n >= 3) {
      vecResult.value = vec3.create(xVal ?? 0, yVal ?? 0, zVal ?? 0);
      vecResult.code = `${nodeToVariableDeclaration(this)} = vec3f(${xRef == "" ? xVal : xRef}, ${yRef == "" ? yVal : yRef}, ${zRef == "" ? zVal : zRef});`;
    }

    if (this.n === 4) {
      vecResult.value = vec4.create(xVal ?? 0, yVal ?? 0, zVal ?? 0, wVal ?? 0);
      vecResult.code = `${nodeToVariableDeclaration(this)} = vec4f(${xRef == "" ? xVal : xRef}, ${yRef == "" ? yVal : yRef}, ${zRef == "" ? zVal : zRef}, ${wRef == "" ? wVal : wRef});`;
    }

    if (this.update) this.update(this);

    return {
      value: vecResult,
    };
  }

  serialize(sn: SerializedNode) {
    sn.nodeType = "Vector";
    if (this.hasControl("x")) {
      sn.inputs = [
        { type: "number", value: this.xControl.value ?? 0, key: "x" },
      ];
    } else {
      sn.inputs = [];
    }

    if (this.hasControl("y")) {
      sn.inputs.push({
        type: "number",
        value: this.yControl.value ?? 0,
        key: "y",
      });
    }
    if (this.hasControl("z")) {
      sn.inputs.push({
        type: "number",
        value: this.zControl.value ?? 0,
        key: "z",
      });
    }
    if (this.hasControl("w")) {
      sn.inputs.push({
        type: "number",
        value: this.wControl.value ?? 0,
        key: "w",
      });
    }

    sn.extraNumberInformation = [{ key: "n", value: this.n }];

    return super.serialize(sn);
  }

  deserialize(sn: SerializedNode) {
    for (let info of sn.inputs) {
      if (info.type === "number" && info.key === "x")
        this.xControl.value = info.value;
      if (info.type === "number" && info.key === "y")
        this.yControl.value = info.value;
      if (info.type === "number" && info.key === "z")
        this.zControl.value = info.value;
      if (info.type === "number" && info.key === "w")
        this.wControl.value = info.value;
    }

    if (sn.extraNumberInformation) {
      for (let info of sn.extraNumberInformation) {
        if (info.key === "n") {
          this.n = info.value as 2 | 3 | 4;
          this.label = "Vector" + info.value.toString();
        }
      }
      if (this.n <= 3) {
        this.removeControl("w");
        this.removeInput("w");
      }
      if (this.n == 2) {
        this.removeControl("z");
        this.removeInput("z");
      }
    }

    super.deserialize(sn);
  }
}

export class SeparateNode extends VPNode {
  private zOutput: ClassicPreset.Output<ClassicPreset.Socket>;
  private wOutput: ClassicPreset.Output<ClassicPreset.Socket>;
  constructor(private update?: (node: SeparateNode) => void) {
    super("Separate");

    this.addInput("vector", new ClassicPreset.Input(reteSocket, "Vector"));

    this.addOutput("x", new ClassicPreset.Output(reteSocket, "X"));
    this.addOutput("y", new ClassicPreset.Output(reteSocket, "Y"));

    this.zOutput = new ClassicPreset.Output(reteSocket, "Z");
    this.wOutput = new ClassicPreset.Output(reteSocket, "W");

    // this.addOutput("z", new ClassicPreset.Output(reteSocket, "Z"));
    // this.addOutput("w", new ClassicPreset.Output(reteSocket, "W"));

    this.updateSize();
  }

  data(inputs: { vector?: NodeReturn }): { x: NodeReturn; y: NodeReturn } {
    const vector = inputs.vector;

    let result = {
      x: {
        value: vector ? vector.value[0] : 0,
        code: `${nodeToVariableDeclaration(this)}_1 = ${vector ? vector.refId + "[0]" : "0"};`,
        refId: idToVariableName(this.id) + "_1",
      },
      y: {
        value: vector ? vector.value[1] : 0,
        code: `${nodeToVariableDeclaration(this)}_2 = ${vector ? vector.refId + "[1]" : "0"};`,
        refId: idToVariableName(this.id) + "_2",
      },
      z: {
        value: 0,
        code: "",
        refId: idToVariableName(this.id) + "_3",
      },
      w: {
        value: 0,
        code: "",
        refId: idToVariableName(this.id) + "_4",
      },
    };

    if (vector && vector.value.length >= 3) {
      result.z.value = vector.value[2];
      result.z.code = `${nodeToVariableDeclaration(this)}_3 = ${vector ? vector.refId + "[2]" : "0"};`;
      if (!this.hasOutput("z")) {
        this.addOutput("z", this.zOutput);

        if (this.update) {
          this.update(this);
        }
      }
    } else {
      if (this.hasOutput("z")) this.removeOutput("z");

      if (this.update) this.update(this);
    }

    if (vector && vector.value.length >= 4) {
      result.w.value = vector.value[3];
      result.w.code = `${nodeToVariableDeclaration(this)}_4 = ${vector ? vector.refId + "[3]" : "0"};`;
      if (!this.hasOutput("w")) {
        this.addOutput("w", this.wOutput);

        if (this.update) {
          this.update(this);
        }
      }
    } else {
      if (this.hasOutput("w")) this.removeOutput("w");

      if (this.update) this.update(this);
    }

    return result;
  }

  serialize(sn: SerializedNode): SerializedNode {
    sn.nodeType = "Separate";
    return super.serialize(sn);
  }

  deserialize(sn: SerializedNode) {
    super.deserialize(sn);
    this.addOutput("z", this.zOutput);
    this.addOutput("w", this.wOutput);
  }

  clone(): Nodes | void {
    return new SeparateNode(this.update);
  }
}

export class JoinNode extends VPNode {
  private zInput: ClassicPreset.Input<ClassicPreset.Socket>;
  private wInput: ClassicPreset.Input<ClassicPreset.Socket>;
  constructor(private update?: (node: ClassicPreset.Node) => void) {
    super("Join");

    this.zInput = new ClassicPreset.Input(reteSocket, "Z");
    this.wInput = new ClassicPreset.Input(reteSocket, "W");

    this.addInput("x", new ClassicPreset.Input(reteSocket, "X"));
    this.addInput("y", new ClassicPreset.Input(reteSocket, "Y"));

    this.addOutput("value", new ClassicPreset.Output(reteSocket, "Vector"));

    this.updateSize();
  }

  data(inputs: {
    x?: NodeReturn;
    y?: NodeReturn;
    z?: NodeReturn;
    w?: NodeReturn;
  }): { value: NodeReturn } {
    const { x, y, z, w } = inputs;
    let xVal = x ? x.value : 0;
    let yVal = y ? y.value : 0;
    let zVal = z ? z.value : 0;
    let wVal = w ? w.value : 0;
    let xRef = x ? (x.refId ?? "") : "";
    let yRef = y ? (y.refId ?? "") : "";
    let zRef = z ? (z.refId ?? "") : "";
    let wRef = w ? (w.refId ?? "") : "";

    let result = {
      value: {
        value: vec2.create(xVal, yVal),
        code: `${nodeToVariableDeclaration(this)} = vec2f(${xRef == "" ? xVal : xRef}, ${yRef == "" ? yVal : yRef}`,
        refId: idToVariableName(this.id),
      },
    };

    if (x && y) {
      if (!this.hasInput("z")) this.addInput("z", this.zInput);
    } else {
      if (this.hasInput("z") && !z) this.removeInput("z");
    }

    if (x && y && z) {
      if (!this.hasInput("w")) this.addInput("w", this.wInput);
    } else {
      if (this.hasInput("w") && !w) this.removeInput("w");
    }
    if (this.update) this.update(this);

    if (z) {
      result.value.code = result.value.code.replace("vec2", "vec3");
      result.value.code += ", " + (zRef == "" ? zVal : zRef);
      result.value.value = vec3.create(xVal, yVal, zVal);
    }

    if (w) {
      result.value.code = result.value.code.replace("vec2", "vec4");
      result.value.code = result.value.code.replace("vec3", "vec4");
      result.value.code += ", " + (wRef == "" ? wVal : wRef);
      result.value.value = vec4.create(xVal, yVal, zVal, wVal);
    }

    result.value.code += ");";

    return result;
  }

  serialize(sn: SerializedNode): SerializedNode {
    sn.nodeType = "Join";
    return super.serialize(sn);
  }

  deserialize(sn: SerializedNode) {
    super.deserialize(sn);
    this.addInput("z", this.zInput);
    this.addInput("w", this.wInput);
  }

  clone(): Nodes | void {
    return new JoinNode(this.update);
  }
}
