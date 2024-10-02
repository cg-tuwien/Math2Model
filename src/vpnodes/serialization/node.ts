export class SerializedNode {
  public size: [number, number] = [0, 0];
  public uuid: string = "";
  public inputs: NodeInput[] = [];
  public nodeType: NodeType = "Number";
  parent?: string;
  extraStringInformation?: ExtraStringInformation[];
  extraNumberInformation?: ExtraNumberInformation[];

  getTextOrNumberInput(key: string) {
    // console.log("Searching for", key, "in", JSON.stringify(this.inputs));
    for (let input of this.inputs) {
      if (
        (input.type === "number" || input.type === "text") &&
        input.key === key
      )
        return input;
    }
    return undefined;
  }
}

export function toSerializedNode(obj: {
  size: [number, number];
  uuid: string;
  inputs: NodeInput[];
  nodeType: NodeType;
  parent?: string;
  extraStringInformation?: ExtraStringInformation[];
  extraNumberInformation?: ExtraNumberInformation[];
}) {
  const sn = new SerializedNode();
  sn.size = obj.size;
  sn.uuid = obj.uuid;
  sn.inputs = obj.inputs;
  sn.nodeType = obj.nodeType;
  sn.parent = obj.parent;
  sn.extraStringInformation = obj.extraStringInformation;
  sn.extraNumberInformation = obj.extraNumberInformation;
  return sn;
}

type ExtraStringInformation = {
  key: string;
  value: string;
};

type ExtraNumberInformation = {
  key: string;
  value: number;
};

type NodeInput =
  | {
      type: "node";
      value: string;
      keyFrom: string;
      keyTo: string;
    }
  | {
      type: "number";
      value: number;
      key: string;
    }
  | {
      type: "text";
      value: string;
      key: string;
    };

type NodeType =
  | "Number"
  | "Math"
  | "Vector"
  | "Separate"
  | "Join"
  | "FunctionCall"
  | "Return"
  | "VariableOut"
  | "VariableIn"
  | "Initialize"
  | "FunctionScope"
  | "CustomFunction"
  | "CallCustomFunction"
  | "LogicScope"
  | "Condition"
  | "Shape"
  | "Combine";
