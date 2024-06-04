import { makeFilePath, type FilePath } from "@/filesystem/reactive-files";
import {
  ReadonlyEulerAngles,
  ReadonlyVector3,
  type VirtualModelState,
} from "@/scenes/VirtualScene";
import { assertUnreachable } from "@stefnotch/typestef/assert";

export function commonModelState(
  models: VirtualModelState[]
): VirtualModelState {
  const aggregrate = <T extends AggregatableValue>(
    getValue: (model: VirtualModelState) => T,
    defaultValue: T
  ) => {
    return aggregrateValues(models.map(getValue), defaultValue);
  };

  const output: VirtualModelState = {
    id: aggregrate((m) => m.id, ""),
    name: aggregrate((m) => m.name, ""),
    code: aggregrate((m) => m.code, makeFilePath("")),
    position: aggregrate((m) => m.position, ReadonlyVector3.zero),
    rotation: aggregrate((m) => m.rotation, ReadonlyEulerAngles.identity),
    scale: aggregrate((m) => m.scale, 1),
    material: {
      color: aggregrate((m) => m.material.color, ReadonlyVector3.zero),
      roughness: aggregrate((m) => m.material.roughness, 0),
      metallic: aggregrate((m) => m.material.metallic, 0),
      emissive: aggregrate((m) => m.material.emissive, ReadonlyVector3.zero),
    },
  };
  return output;
}

type AggregatableValue =
  | string
  | FilePath
  | number
  | ReadonlyVector3
  | ReadonlyEulerAngles;

function aggregrateValues<T extends AggregatableValue>(
  values: T[],
  defaultValue: T
): T {
  let base = values[0];
  if (typeof base === "number" || typeof base === "string") {
    for (let i = 1; i < values.length; i++) {
      if (values[i] !== base) {
        return defaultValue;
      }
    }
    return base;
  } else if (base instanceof ReadonlyVector3) {
    const valuesTyped = values as ReadonlyVector3[];
    let defaultValueTyped = defaultValue as ReadonlyVector3;
    return new ReadonlyVector3(
      aggregrateValues(
        valuesTyped.map((v) => v.x),
        defaultValueTyped.x
      ),
      aggregrateValues(
        valuesTyped.map((v) => v.y),
        defaultValueTyped.y
      ),
      aggregrateValues(
        valuesTyped.map((v) => v.z),
        defaultValueTyped.z
      )
    ) as T;
  } else if (base instanceof ReadonlyEulerAngles) {
    const valuesTyped = values as ReadonlyEulerAngles[];
    let defaultValueTyped = defaultValue as ReadonlyEulerAngles;
    return new ReadonlyEulerAngles(
      aggregrateValues(
        valuesTyped.map((v) => v.x),
        defaultValueTyped.x
      ),
      aggregrateValues(
        valuesTyped.map((v) => v.y),
        defaultValueTyped.y
      ),
      aggregrateValues(
        valuesTyped.map((v) => v.z),
        defaultValueTyped.z
      )
    ) as T;
  } else {
    assertUnreachable(base);
  }
}
