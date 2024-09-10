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

export class NumberNode extends VPNode {
  private valueControl: ClassicPreset.InputControl<"number", number>;
  private valueIn: ClassicPreset.Input<ClassicPreset.Socket>;
  constructor(private update?: (node: ClassicPreset.Node) => void) {
    super("Number");

    this.valueControl = new ClassicPreset.InputControl("number", {
      initial: 0.0,
    });
    this.valueIn = new ClassicPreset.Input(reteSocket, "in");
    // this.addControl("value", this.valueControl);
    this.addInput("value", this.valueIn);
    this.valueIn.addControl(this.valueControl);
    this.addOutput("value", new ClassicPreset.Output(reteSocket));

    this.updateSize();
  }

  data(inputs: { value?: NodeReturn[] }): { value: NodeReturn } {
    const control: ClassicPreset.InputControl<"number", number> | null = this
      .valueIn.control as ClassicPreset.InputControl<"number", number>;
    let result = {
      value: {
        value: parseFloat(control?.value?.toFixed(20) ?? "0.0"),
        code:
          nodeToVariableDeclaration(this) +
          " = " +
          (control?.value?.toFixed(20) ?? "0.0") +
          ";",
        refId: idToVariableName(this.id),
      },
    };
    if (inputs.value) {
      result.value.value = inputs.value[0].value ?? 0;
      result.value.code = `${nodeToVariableDeclaration(this)} = ${inputs.value[0].refId};`;
      if (this.valueIn.control) this.valueIn.removeControl();
    } else {
      if (!this.valueIn.control) this.valueIn.addControl(this.valueControl);
    }

    if (this.update) this.update(this);

    return result;
  }
}

export class MathOpNode extends VPNode {
  private leftControl: ClassicPreset.InputControl<"number", number>;
  private rightControl: ClassicPreset.InputControl<"number", number>;
  constructor(
    private operator: "+" | "-" | "/" | "*" | "%",
    private update?: (
      node: ClassicPreset.Node,
      control: ClassicPreset.InputControl<"number">,
    ) => void,
  ) {
    super(opToName(operator));

    this.leftControl = new ClassicPreset.InputControl<"number", number>(
      "number",
      {
        initial: 0,
      },
    );
    this.rightControl = new ClassicPreset.InputControl<"number", number>(
      "number",
      {
        initial: 0,
      },
    );
    this.addInput("left", new ClassicPreset.Input(reteSocket, "X"));
    this.addInput("right", new ClassicPreset.Input(reteSocket, "Y"));

    this.addControl(
      "result",
      new ClassicPreset.InputControl("number", {
        readonly: true,
        initial: 0,
      }),
    );

    this.addControl("left", this.leftControl);
    this.addControl("right", this.rightControl);
    this.addOutput("value", new ClassicPreset.Output(reteSocket, "Result"));

    this.updateSize();
  }

  data(inputs: {
    left?: NodeReturn[] | number[];
    right?: NodeReturn[] | number[];
  }): { value: NodeReturn } {
    const { left, right } = inputs;

    console.log(left);
    console.log(right);

    let leftValue = this.leftControl.value ?? 0.0;
    let leftId = "";
    if (left) {
      if (typeof left[0] != "number") {
        leftValue = parseFloat(left[0].value.toFixed(20));
        leftId = left[0].refId ? left[0].refId : "";
      } else {
        leftValue = parseFloat(left[0].toFixed(20));
      }

      if (this.hasControl("left")) this.removeControl("left");
    } else {
      if (!this.hasControl("left")) this.addControl("left", this.leftControl);
    }

    let rightValue = this.rightControl.value ?? 0.0;

    if (rightValue === 0 && this.operator === "/") rightValue = 1.0;

    let rightId = "";
    if (right) {
      if (typeof right[0] != "number") {
        rightValue = parseFloat(right[0].value.toFixed(20));
        rightId = right[0].refId ? right[0].refId : "";
      } else {
        rightValue = parseFloat(right[0].toFixed(20));
      }

      if (this.hasControl("right")) this.removeControl("right");
    } else {
      if (!this.hasControl("right"))
        this.addControl("right", this.rightControl);
    }

    console.log(leftValue);
    console.log(rightValue);

    const value = applyOperator(leftValue, rightValue, this.operator);
    const code =
      nodeToVariableDeclaration(this) +
      " = " +
      (leftId === "" ? leftValue.toString() : leftId) +
      ` ${this.operator} ` +
      (rightId === "" ? rightValue.toString() : rightId) +
      ";";
    console.log(value);

    const control: ClassicPreset.InputControl<"number", number> | undefined =
      this.controls?.result as ClassicPreset.InputControl<"number", number>;
    control?.setValue(value);

    if (this.update) this.update(this, control);

    return {
      value: {
        value: value,
        code: code,
        refId: idToVariableName(this.id),
      },
    };
  }
}
