import { ClassicPreset } from "rete";
import type { Vec2 } from "webgpu-matrix/dist/1.x/vec2";
import { vec2, vec3 } from "webgpu-matrix";
import type { Vec3 } from "webgpu-matrix/dist/1.x/vec3";

export const reteSocket = new ClassicPreset.Socket("socket");

export function nodeToVariableDeclaration(node: ClassicPreset.Node) {
  return "var " + idToVariableName(node.id);
}

export function idToVariableName(id: string): string {
  return `ref_${id.substring(0, 5)}`;
}

function opToName(op: "+" | "-" | "/" | "*" | "%"): string {
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

function applyOperator(
  left: number,
  right: number,
  op: "+" | "-" | "/" | "*" | "%",
): number {
  let result = left;

  if (op === "+") result += right;
  else if (op === "-") result -= right;
  else if (op === "/") result /= right;
  else if (op === "*") result *= right;
  else result %= right;

  return result;
}

export class NodeReturn {
  constructor(
    public value: any,
    public code: string,
    public refId?: string,
  ) {}
}

export class NumberNode extends ClassicPreset.Node {
  constructor() {
    super("Number");

    this.addControl(
      "value",
      new ClassicPreset.InputControl("number", { initial: 0 }),
    );
    this.addOutput("value", new ClassicPreset.Output(reteSocket));
  }

  data(): { value: NodeReturn } {
    const control: ClassicPreset.InputControl<"number", number> | undefined =
      this.controls?.value as ClassicPreset.InputControl<"number", number>;
    return {
      value: {
        value: control?.value ?? 0,
        code:
          nodeToVariableDeclaration(this) +
          " = " +
          (control?.value?.toString() ?? "0") +
          ";",
        refId: idToVariableName(this.id),
      },
    };
  }
}

export class MathOpNode extends ClassicPreset.Node {
  constructor(
    private operator: "+" | "-" | "/" | "*" | "%",
    private update?: (control: ClassicPreset.InputControl<"number">) => void,
  ) {
    super(opToName(operator));
    this.addInput("left", new ClassicPreset.Input(reteSocket, "X"));
    this.addInput("right", new ClassicPreset.Input(reteSocket, "Y"));

    this.addControl(
      "result",
      new ClassicPreset.InputControl("number", {
        readonly: true,
        initial: 0,
      }),
    );
    this.addOutput("value", new ClassicPreset.Output(reteSocket, "Result"));
  }

  data(inputs: {
    left?: NodeReturn[] | number[];
    right?: NodeReturn[] | number[];
  }): { value: NodeReturn } {
    const { left, right } = inputs;

    console.log(left);
    console.log(right);

    let leftValue = 0;
    let leftId = "";
    if (left) {
      if (typeof left[0] != "number") {
        leftValue = left[0].value;
        leftId = left[0].refId ? left[0].refId : "";
      } else {
        leftValue = left[0];
      }
    }

    let rightValue = 0;
    let rightId = "";
    if (right) {
      if (typeof right[0] != "number") {
        rightValue = right[0].value;
        rightId = right[0].refId ? right[0].refId : "";
      } else {
        rightValue = right[0];
      }
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

    if (this.update) this.update(control);

    return {
      value: {
        value: value,
        code: code,
        refId: idToVariableName(this.id),
      },
    };
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

    this.addOutput("value", new ClassicPreset.Output(reteSocket));
  }

  data(): { value: NodeReturn } {
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

    return {
      value: {
        value: vec2.create(xCont?.value ?? 0, yCont?.value ?? 0),
        code:
          nodeToVariableDeclaration(this) +
          " = " +
          "vec2(" +
          (xCont?.value?.toString() ?? "0") +
          ", " +
          (yCont?.value?.toString() ?? "0") +
          ");",
        refId: idToVariableName(this.id),
      },
    };
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

  data(inputs: { vector?: NodeReturn[] }): { x: NodeReturn; y: NodeReturn } {
    const vector = inputs.vector;
    console.log(vector ? vector[0].value : "Hello");
    return {
      x: {
        value: vector ? vector[0].value[0] : 0,
        code: `${nodeToVariableDeclaration(this)}_1 = ${vector ? vector[0].refId + "[0]" : "0"};`,
        refId: idToVariableName(this.id) + "_1",
      },
      y: {
        value: vector ? vector[0].value[1] : 0,
        code: `${nodeToVariableDeclaration(this)}_2 = ${vector ? vector[0].refId + "[1]" : "0"};`,
        refId: idToVariableName(this.id) + "_2",
      },
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

    this.addOutput("value", new ClassicPreset.Output(reteSocket, "Vector2"));
  }

  data(inputs: { x?: NodeReturn[]; y?: NodeReturn[] }): { value: NodeReturn } {
    const { x, y } = inputs;
    return {
      value: {
        value: [vec2.create(x ? x[0].value : 0, y ? y[0].value : 0)],
        code:
          nodeToVariableDeclaration(this) +
          " = " +
          `vec2(${x ? x[0].refId : 0}, ${y ? y[0].refId : 0});`,
        refId: idToVariableName(this.id),
      },
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

    this.addOutput("value", new ClassicPreset.Output(reteSocket));
  }

  data(): { value: NodeReturn } {
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
      value: {
        value: vec3.create(
          xCont?.value ?? 0,
          yCont?.value ?? 0,
          zCont?.value ?? 0,
        ),
        code:
          nodeToVariableDeclaration(this) +
          " = " +
          `vec3(${xCont?.value?.toString() ?? 0}, ${yCont?.value?.toString() ?? 0}, ${zCont?.value?.toString() ?? 0});`,
        refId: idToVariableName(this.id),
      },
    };
  }
}
