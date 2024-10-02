import Heart from "@/../parametric-renderer-core/graphs/Heart.graph.wgsl?raw";
import Sphere from "@/../parametric-renderer-core/graphs/Sphere.graph.wgsl?raw";
import Plane from "@/../parametric-renderer-core/graphs/BasicGraphShader.graph.wgsl?raw";
import Cylinder from "@/../parametric-renderer-core/graphs/Cylinder.graph.wgsl?raw";
import {
  idToVariableName,
  NodeReturn,
  nodeToVariableDeclaration,
  reteSocket,
  VPNode,
} from "@/vpnodes/basic/nodes";
import { ClassicPreset } from "rete";
import { vec2, vec3 } from "webgpu-matrix";
import { typeToValue } from "@/vpnodes/basic/functions";
import { type SerializedNode } from "@/vpnodes/serialization/node";
import { SliderControl } from "@/vpnodes/controls/slider";

export class ShapeNode extends VPNode {
  constructor(
    protected name: string,
    protected code: string,
  ) {
    super(name);

    this.addInput(
      "param",
      new ClassicPreset.Input(reteSocket, "input[u, v]/vec2f"),
    );
    this.addOutput(
      "value",
      new ClassicPreset.Output(reteSocket, "output[x, y, z]/vec3f"),
    );
  }

  data(inputs: { param: NodeReturn[] }): {
    function: NodeReturn;
    value: NodeReturn;
  } {
    const { param } = inputs;
    return {
      function: {
        value: 0,
        code: this.code,
      },
      value: {
        value: vec3.zero(),
        code: `${nodeToVariableDeclaration(this)} = ${this.name}(${param ? (param[0].refId ?? param[0].value) : "vec2f(0.0, 0.0)"});`,
        refId: idToVariableName(this.id),
      },
    };
  }

  serialize(sn: SerializedNode): SerializedNode {
    sn.nodeType = "Shape";
    sn.extraStringInformation = [{ key: "name", value: this.name }];
    return super.serialize(sn);
  }

  deserialize(sn: SerializedNode) {
    super.deserialize(sn);
    if (sn.extraStringInformation) {
      for (let info of sn.extraStringInformation) {
        if (info.key == "name") {
          this.name = info.value;
          this.label = this.name;
          this.code = nameToCode(info.value);
        }
      }
    }
  }
}

export function newHeartShape() {
  return new ShapeNode("Heart", Heart);
}

export function newSphereShape() {
  return new ShapeNode("Sphere", Sphere);
}

export function newPlaneShape() {
  return new ShapeNode("Plane", Plane);
}

export function newCylinderShape() {
  return new ShapeNode("Cylinder", Cylinder);
}

function nameToCode(name: "Heart" | "Sphere" | "Plane") {
  switch (name) {
    case "Heart":
      return Heart;
    case "Sphere":
      return Sphere;
    case "Plane":
      return Plane;
  }
  return "";
}

export class CombineNode extends VPNode {
  private cfControl: SliderControl;
  constructor(
    private update: (id: string) => void,
    private updateControl: (c: ClassicPreset.Control) => void,
  ) {
    super("Combine Shapes");

    this.cfControl = new SliderControl(
      0.5,
      1.0,
      0.0,
      0.01,
      "Combine Factor",
      (value) => {
        this.update(this.id);
      },
      (value) => {
        this.updateControl(this.cfControl);
      },
    );

    this.addInput(
      "param1",
      new ClassicPreset.Input(reteSocket, "shape 1/vec3f"),
    );
    this.addInput(
      "param2",
      new ClassicPreset.Input(reteSocket, "shape 2/vec3f"),
    );
    this.addControl("cfactor", this.cfControl);
    this.addOutput(
      "value",
      new ClassicPreset.Output(reteSocket, "output[x, y, z]/vec3f"),
    );
  }

  data(inputs: { param1: NodeReturn[]; param2: NodeReturn[] }): {
    value: NodeReturn;
  } {
    const { param1, param2 } = inputs;
    const p1 = param1 ? param1[0].refId : "vec3f(0.0, 0.0, 0.0)";
    const p2 = param2 ? param2[0].refId : "vec3f(0.0, 0.0, 0.0)";
    console.log(this.cfControl.value);
    return {
      value: {
        value: vec3.zero(),
        code: `${nodeToVariableDeclaration(this)} = mix(${p1}, ${p2}, ${this.cfControl.value});`,
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
}
