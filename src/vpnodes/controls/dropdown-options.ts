import type { CustomFunctionNode } from "@/vpnodes/basic/functions";
import type { DropdownControl } from "@/vpnodes/controls/dropdown";

export const typeOptions = [
  { label: "int", value: "i32" },
  { label: "float", value: "f32" },
  { label: "vec2", value: "vec2f" },
  { label: "vec3", value: "vec3f" },
  { label: "vec4", value: "vec4f" },
];

export const customFunctions: { label: string; value: CustomFunctionNode }[] =
  [];

const listeners: ((change: { label: string; value: string }[]) => void)[] = [];

export function hasCustomFunction(func: CustomFunctionNode) {
  for (let f of customFunctions) {
    if (f.label === func.label) return true;
  }
  return false;
}

export function addCustomFunction(func: CustomFunctionNode) {
  if (hasCustomFunction(func)) return;
  customFunctions.push({
    label: func.label,
    value: func,
  });
  listeners.forEach((fn) => fn(getCustomFunctionOptions()));
}

export function removeCustomFunction(func: CustomFunctionNode) {
  let index = -1;
  for (let i = 0; i < customFunctions.length; i++) {
    if (customFunctions[i].label === func.label) {
      index = i;
      break;
    }
  }
  if (index > -1) customFunctions.splice(index, 1);
  listeners.forEach((fn) => fn(getCustomFunctionOptions()));
}

export function getCustomFunction(label: string) {
  const f = customFunctions.find((f) => f.label === label);
  if (f) return f.value;
  return undefined;
}

export function getCustomFunctionOptions() {
  return customFunctions.map((f) => {
    return { label: f.label, value: f.label };
  });
}

export function notify() {
  listeners.forEach((fn) => fn(getCustomFunctionOptions()));
}

export function subscribe(
  listener: (change: { label: string; value: string }[]) => void,
) {
  listeners.push(listener);
}

export function unsubscribe(
  listener: (change: { label: string; value: string }[]) => void,
) {
  let index = -1;
  for (let i = 0; i < listeners.length; i++) {
    if (listeners[i] === listener) {
      index = i;
      break;
    }
  }
  if (index > -1) listeners.splice(index, 1);
}
