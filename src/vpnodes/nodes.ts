import { ClassicPreset } from "rete";
import type { Vec2 } from "webgpu-matrix/dist/1.x/vec2";
import { vec2, vec3 } from "webgpu-matrix";
import type { Vec3 } from "webgpu-matrix/dist/1.x/vec3";

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
    this.addInput("left", new ClassicPreset.Input(reteSocket, "X"));
    this.addInput("right", new ClassicPreset.Input(reteSocket, "Y"));

    this.addControl(
      "sum",
      new ClassicPreset.InputControl("number", {
        readonly: true,
        initial: 0,
      }),
    );
    this.addOutput("value", new ClassicPreset.Output(reteSocket, "Result"));
  }

  data(inputs: { left?: number[]; right?: number[] }): { value: number } {
    const { left, right } = inputs;
    const value = (left ? left[0] : 0) + (right ? right[0] : 0);
    console.log(left);
    console.log(right);

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
      }),
    );
    this.addControl(
      "y",
      new ClassicPreset.InputControl("number", {
        initial: 0,
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

  data(): { vec: Vec2 } {
    const xCont: ClassicPreset.InputControl<"number", number> | undefined = this
      .controls?.x as ClassicPreset.InputControl<"number", number>;
    const yCont: ClassicPreset.InputControl<"number", number> | undefined = this
      .controls?.y as ClassicPreset.InputControl<"number", number>;
    const vecCont: ClassicPreset.InputControl<"text", string> | undefined = this
      .controls?.vector as ClassicPreset.InputControl<"text", string>;

    vecCont?.setValue(
      "(" +
        (xCont?.value?.toString() ?? "0") +
        ", " +
        (yCont?.value?.toString() ?? "0") +
        ")",
    );
    if (this.update) this.update(vecCont);

    return { vec: vec2.create(xCont?.value ?? 0, yCont?.value ?? 0) };
  }
}

export class Seperate2Node extends ClassicPreset.Node {
  constructor(
    private update?: (control: ClassicPreset.InputControl<"text">) => void,
  ) {
    super("Seperate2");

    this.addInput("vector", new ClassicPreset.Input(reteSocket, "Vector"));

    this.addOutput("x", new ClassicPreset.Output(reteSocket, "X"));
    this.addOutput("y", new ClassicPreset.Output(reteSocket, "Y"));
  }

  data(inputs: { vector?: Vec2 }): { x: number; y: number } {
    const vector = inputs.vector;
    console.log(vec2.len(vector ?? vec2.zero()));
    return {
      x: vector ? vector[0][0] : 0,
      y: vector ? vector[0][1] : 0,
    };
  }
}

export class Join2Node extends ClassicPreset.Node {
  constructor(
    private update?: (control: ClassicPreset.InputControl<"text">) => void,
  ) {
    super("Join2");

    this.addInput("x", new ClassicPreset.Input(reteSocket, "X"));
    this.addInput("y", new ClassicPreset.Input(reteSocket, "Y"));

    this.addOutput("vector", new ClassicPreset.Output(reteSocket, "Vector2"));
  }

  data(inputs: { x?: number; y?: number }): { vector: Vec2 } {
    const { x, y } = inputs;
    return {
      vector: vec2.create(x ?? 0, y ?? 0),
    };
  }
}

export class Vector3Node extends ClassicPreset.Node {
  constructor(
    private update?: (control: ClassicPreset.InputControl<"text">) => void,
  ) {
    super("Vector3");

    this.addControl(
      "x",
      new ClassicPreset.InputControl("number", {
        initial: 0,
      }),
    );
    this.addControl(
      "y",
      new ClassicPreset.InputControl("number", {
        initial: 0,
      }),
    );
    this.addControl(
      "z",
      new ClassicPreset.InputControl("number", {
        initial: 0,
      }),
    );

    this.addControl(
      "vector",
      new ClassicPreset.InputControl("text", {
        readonly: true,
        initial: "(0, 0, 0)",
      }),
    );

    this.addOutput("vec", new ClassicPreset.Output(reteSocket));
  }

  data(): { vec: Vec3 } {
    const xCont: ClassicPreset.InputControl<"number", number> | undefined = this
      .controls?.x as ClassicPreset.InputControl<"number", number>;
    const yCont: ClassicPreset.InputControl<"number", number> | undefined = this
      .controls?.y as ClassicPreset.InputControl<"number", number>;
    const zCont: ClassicPreset.InputControl<"number", number> | undefined = this
      .controls?.z as ClassicPreset.InputControl<"number", number>;
    const vecCont: ClassicPreset.InputControl<"text", string> | undefined = this
      .controls?.vector as ClassicPreset.InputControl<"text", string>;

    vecCont?.setValue(
      "(" +
        (xCont?.value?.toString() ?? "0") +
        ", " +
        (yCont?.value?.toString() ?? "0") +
        ", " +
        (zCont?.value?.toString() ?? "0") +
        ")",
    );
    if (this.update) this.update(vecCont);

    return {
      vec: vec3.create(xCont?.value ?? 0, yCont?.value ?? 0, zCont?.value ?? 0),
    };
  }
}
