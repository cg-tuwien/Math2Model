import { ClassicPreset } from "rete";
import { Vector2 } from "@babylonjs/core";

export const reteSocket = new ClassicPreset.Socket("socket");

export class NumberNode extends ClassicPreset.Node {
  constructor() {
    super("Number");

    this.addControl(
      "value",
      new ClassicPreset.InputControl("number", { initial: 0 }),
    );
    this.addOutput("value", new ClassicPreset.Output(reteSocket));
  }

  data(): { value: number } {
    const control: ClassicPreset.InputControl<"number", number> | undefined =
      this.controls?.value as ClassicPreset.InputControl<"number", number>;
    return {
      value: control?.value ?? 0,
    };
  }
}

export class AddNode extends ClassicPreset.Node {
  constructor(
    private update?: (control: ClassicPreset.InputControl<"number">) => void,
  ) {
    super("Add");
    this.addInput("left", new ClassicPreset.Input(reteSocket, "Left"));
    this.addInput("right", new ClassicPreset.Input(reteSocket, "Right"));

    this.addControl(
      "sum",
      new ClassicPreset.InputControl("number", {
        readonly: true,
        initial: 0,
      }),
    );
    this.addOutput("value", new ClassicPreset.Output(reteSocket, "Number"));
  }

  data(inputs: { left?: number[]; right?: number[] }): { value: number } {
    const { left, right } = inputs;
    const value = (left ? left[0] : 0) + (right ? right[0] : 0);

    const control: ClassicPreset.InputControl<"number", number> | undefined =
      this.controls?.sum as ClassicPreset.InputControl<"number", number>;
    console.log(control);
    control?.setValue(value);

    if (this.update) this.update(control);

    return { value };
  }
}

export class Vector2Node extends ClassicPreset.Node {
  constructor(
    private update?: (control: ClassicPreset.InputControl<"text">) => void,
  ) {
    super("Vector2");

    this.addControl(
      "x",
      new ClassicPreset.InputControl("number", {
        initial: 0,
        change: this.chgX,
      }),
    );
    this.addControl(
      "y",
      new ClassicPreset.InputControl("number", {
        initial: 0,
        change: this.chgY,
      }),
    );

    this.addControl(
      "vector",
      new ClassicPreset.InputControl("text", {
        readonly: true,
        initial: "(0, 0)",
      }),
    );

    this.addOutput("vec", new ClassicPreset.Output(reteSocket));
  }

  data(): { x: number; y: number } {
    const xCont: ClassicPreset.InputControl<"number", number> | undefined = this
      .controls?.x as ClassicPreset.InputControl<"number", number>;
    const yCont: ClassicPreset.InputControl<"number", number> | undefined = this
      .controls?.y as ClassicPreset.InputControl<"number", number>;

    return { x: xCont?.value ?? 0, y: yCont?.value ?? 0 };
  }

  chgX(value: number): void {
    console.log("Update X: " + value.toString() + this.controls);
    const vecCont: ClassicPreset.InputControl<"text", string> | undefined = this
      .controls?.vector as ClassicPreset.InputControl<"text", string>;
    const yCont: ClassicPreset.InputControl<"number", number> | undefined = this
      .controls?.y as ClassicPreset.InputControl<"number", number>;
    console.log(vecCont);
    console.log(yCont);

    vecCont?.setValue(
      "(" + value.toString() + ", " + yCont?.value?.toString() ?? "0" + ")",
    );
    console.log("New vector: " + vecCont?.value?.toString() ?? "(0, 0)");
    if (this.update) this.update(vecCont);
  }

  chgY(value: number): void {
    const vecCont: ClassicPreset.InputControl<"text", string> | undefined = this
      .controls?.vector as ClassicPreset.InputControl<"text", string>;
    const xCont: ClassicPreset.InputControl<"number", number> | undefined = this
      .controls?.y as ClassicPreset.InputControl<"number", number>;

    vecCont?.setValue(
      "(" + xCont?.value?.toString() ?? "0" + ", " + value.toString() + ")",
    );
    if (this.update) this.update(vecCont);
  }
}
