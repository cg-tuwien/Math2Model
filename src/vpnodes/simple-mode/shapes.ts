import Heart from "@/../parametric-renderer-core/graphs/Heart.graph.wgsl?raw";
import Sphere from "@/../parametric-renderer-core/graphs/Sphere.graph.wgsl?raw";
import Plane from "@/../parametric-renderer-core/graphs/BasicGraphShader.graph.wgsl?raw";
import Cylinder from "@/../parametric-renderer-core/graphs/Cylinder.graph.wgsl?raw";
import Cube from "@/../parametric-renderer-core/graphs/Cube.graph.wgsl?raw";
import {
  idToVariableName,
  NodeReturn,
  nodeToVariableDeclaration,
  reteSocket,
  VPNode,
} from "@/vpnodes/basic/nodes";
import { ClassicPreset } from "rete";
import { vec3 } from "webgpu-matrix";
import { type SerializedNode } from "@/vpnodes/serialization/node";
import type { Nodes } from "../nodes-list";

export class ShapeNode extends VPNode {
  constructor(
    protected name: string,
    protected code: string
  ) {
    super(name);

    this.addInput(
      "param",
      new ClassicPreset.Input(reteSocket, "input[u, v]/vec2f")
    );
    this.addOutput(
      "value",
      new ClassicPreset.Output(reteSocket, "output[x, y, z]/vec3f")
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

  clone(): Nodes | void {
    return new ShapeNode(this.name, this.code);
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

export function newCubeShape() {
  return new ShapeNode("Cube", Cube);
}

function nameToCode(name: string) {
  switch (name) {
    case "Heart":
      return Heart;
    case "Sphere":
      return Sphere;
    case "Plane":
      return Plane;
    case "Cylinder":
      return Cylinder;
  }
  return "";
}
