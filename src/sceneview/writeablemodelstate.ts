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
  return aggregrateValues(models, {
    id: "",
    name: "",
    code: makeFilePath(""),
    position: ReadonlyVector3.zero,
    rotation: ReadonlyEulerAngles.identity,
    scale: 1,
    material: {
      color: ReadonlyVector3.zero,
      roughness: 0,
      metallic: 0,
      emissive: ReadonlyVector3.zero,
    },
  });
}

type AggregatableValue =
  | string
  | FilePath
  | number
  | ReadonlyVector3
  | ReadonlyEulerAngles
  | {
      [key: string]: AggregatableValue;
    };

function aggregrateValues<T extends AggregatableValue>(
  values: T[],
  defaultValue: T
): T {
  if (typeof defaultValue === "number" || typeof defaultValue === "string") {
    let base = values.length > 0 ? values[0] : defaultValue;
    for (let i = 1; i < values.length; i++) {
      if (values[i] !== base) {
        return defaultValue;
      }
    }
    return base;
  } else if (defaultValue instanceof ReadonlyVector3) {
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
  } else if (defaultValue instanceof ReadonlyEulerAngles) {
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
  } else if (typeof defaultValue === "object") {
    const valuesTyped = values as { [key: string]: AggregatableValue }[];
    return Object.fromEntries(
      Object.entries(defaultValue).map(([key, value]) => {
        return [
          key,
          aggregrateValues(
            valuesTyped.map((v) => v[key]),
            value
          ),
        ];
      })
    ) as T;
  } else {
    assertUnreachable(defaultValue);
  }
}
