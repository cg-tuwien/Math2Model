import {
    Vector3,
    Quaternion,
} from "@babylonjs/core";
export class ObjectData {
    public name: string = "";
    public code: string = "";
    public position: Vector3 = Vector3.One();
    public rotation: Quaternion = Quaternion.Identity();
}