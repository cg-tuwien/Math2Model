import { ClassicPreset } from "rete";
import {
  applyOperator,
  idToVariableName,
  NodeReturn,
  nodeToVariableDeclaration,
  opToName,
  reteSocket,
  VPNode,
} from "@/vpnodes/basic/nodes";
import { type SerializedNode } from "@/vpnodes/serialization/node";
import type { Control } from "rete/_types/presets/classic";
import { NumberControl } from "../controls/number";
import type { Nodes } from "../nodes-list";

export class NumberNode extends VPNode {
  private valueControl: NumberControl;
  private nameControl: ClassicPreset.InputControl<"text", string>;
  constructor(
    private update?: (node: string) => void,
    private updateControl?: (control: Control) => void
  ) {
    super("Number");

    this.nameControl = new ClassicPreset.InputControl("text", {
      initial: idToVariableName(this.id),
    });
    this.addControl("variable name", this.nameControl);
    this.valueControl = new NumberControl(
      0.0,
      0.1,
      "value",
      () => {
        if (this.update) this.update(this.id);
      },
      (value) => {
        if (this.updateControl) this.updateControl(this.valueControl);
      }
    );
    this.addControl("value", this.valueControl);
    this.addOutput("value", new ClassicPreset.Output(reteSocket));

    this.updateSize();
  }

  data(): { value: NodeReturn } {
    const control: NumberControl | null = this.valueControl;
    let result = {
      value: {
        value: parseFloat(control?.value?.toFixed(20) ?? "0.0"),
        code:
          "var " +
          this.nameControl.value +
          " = " +
          (control?.value?.toFixed(20) ?? "0.0") +
          ";",
        refId: this.nameControl.value,
      },
    };

    // if (this.update) this.update(this.id);

    return result;
  }

  serialize(sn: SerializedNode): SerializedNode {
    sn.nodeType = "Number";
    const control: NumberControl | null = this.valueControl;

    if (control) {
      sn.inputs = [
        { type: "number", value: control.value ?? 0, key: "value" },
        {
          type: "text",
          value: this.nameControl.value ?? idToVariableName(this.id),
          key: "name",
        },
      ];
    }

    return super.serialize(sn);
  }

  deserialize(sn: SerializedNode) {
    for (let input of sn.inputs) {
      if (input.type === "number" && input.key === "value") {
        this.valueControl.value = input.value;
      }
      if (input.type === "text" && input.key === "name") {
        this.nameControl.value = input.value;
      }
    }

    super.deserialize(sn);
  }

  clone(): Nodes | void {
    const nn = new NumberNode(this.update, this.updateControl);
    nn.valueControl.value = this.valueControl.value;
    return nn;
  }
}

export class MathOpNode extends VPNode {
  private leftControl: NumberControl;
  private rightControl: NumberControl;
  constructor(
    private operator: "+" | "-" | "/" | "*" | "%" | "max" | "min",
    private update?: (node: string) => void,
    private updateControl?: (control: Control) => void
  ) {
    super(opToName(operator));

    this.leftControl = new NumberControl(
      0.0,
      0.1,
      "left operand",
      () => {
        if (this.update) this.update(this.id);
      },
      (value) => {
        if (this.updateControl) this.updateControl(this.leftControl);
      }
    );
    this.rightControl = new NumberControl(
      0.0,
      0.1,
      "right operand",
      () => {
        if (this.update) this.update(this.id);
      },
      (value) => {
        if (this.updateControl) this.updateControl(this.rightControl);
      }
    );
    this.addInput(
      "left",
      new ClassicPreset.Input(reteSocket, "First Operand/any")
    );
    this.addInput(
      "right",
      new ClassicPreset.Input(reteSocket, "Second Operand/any")
    );

    this.addControl("left", this.leftControl);
    this.addControl("right", this.rightControl);
    this.addOutput("value", new ClassicPreset.Output(reteSocket, "Result/any"));
    this.extraWidth = 10;
    this.extraHeight = 25;

    this.updateSize();
  }

  data(inputs: { left?: NodeReturn; right?: NodeReturn }): {
    value: NodeReturn;
  } {
    const { left, right } = inputs;

    let leftValue = this.leftControl.value ?? 0.0;
    let leftId = "";
    if (left) {
      leftValue = left.value;
      leftId = left.refId ? left.refId : "";

      if (this.hasControl("left")) this.removeControl("left");
    } else {
      if (!this.hasControl("left")) this.addControl("left", this.leftControl);
    }

    let rightValue = this.rightControl.value ?? 0.0;

    if (rightValue === 0 && this.operator === "/") rightValue = 1.0;

    let rightId = "";
    if (right) {
      rightValue = right.value;
      rightId = right.refId ? right.refId : "";

      if (this.hasControl("right")) this.removeControl("right");
    } else {
      if (!this.hasControl("right"))
        this.addControl("right", this.rightControl);
    }

    const value = applyOperator(leftValue, rightValue, this.operator);
    let code =
      nodeToVariableDeclaration(this) +
      " = " +
      (leftId === "" ? leftValue.toFixed(20).toString() : leftId) +
      ` ${this.operator} ` +
      (rightId === "" ? rightValue.toFixed(20).toString() : rightId) +
      ";";

    if (this.operator === "max" || this.operator === "min") {
      code =
        nodeToVariableDeclaration(this) +
        " = " +
        `${this.operator}(${leftId === "" ? leftValue.toFixed(20).toString() : leftId}, ${rightId === "" ? rightValue.toFixed(20).toString() : rightId});`;
    }

    // const control: ClassicPreset.InputControl<"number", number> | undefined =
    //   this.controls?.result as ClassicPreset.InputControl<"number", number>;
    // control?.setValue(value);

    // if (this.update) this.update(this.id);

    return {
      value: {
        value: value,
        code: code,
        refId: idToVariableName(this.id),
      },
    };
  }

  serialize(sn: SerializedNode): SerializedNode {
    sn.nodeType = "Math";

    if (this.hasControl("left")) {
      sn.inputs.push({
        type: "number",
        value: this.leftControl.value ?? 0,
        key: "left",
      });
    }
    if (this.hasControl("right")) {
      sn.inputs.push({
        type: "number",
        value: this.rightControl.value ?? 0,
        key: "right",
      });
    }

    sn.extraStringInformation = [{ key: "op", value: this.operator }];

    return super.serialize(sn);
  }

  deserialize(sn: SerializedNode) {
    for (let info of sn.inputs) {
      if (info.type === "number" && info.key === "left")
        this.leftControl.value = info.value;
      if (info.type === "number" && info.key === "right")
        this.rightControl.value = info.value;
    }

    if (sn.extraStringInformation) {
      for (let info of sn.extraStringInformation) {
        if (info.key === "op") {
          this.operator = info.value as "+" | "-" | "/" | "*" | "%";
          this.label = opToName(this.operator);
        }
      }
    }

    super.deserialize(sn);
  }

  clone(): Nodes | void {
    const mn = new MathOpNode(this.operator, this.update, this.updateControl);
    mn.leftControl.value = this.leftControl.value;
    mn.rightControl.value = this.rightControl.value;
    return mn;
  }
}
