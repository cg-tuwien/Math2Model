import {
  Constants,
  Matrix,
  StorageBuffer,
  UniformBuffer,
  type WebGPUEngine,
} from "@babylonjs/core";
import { assert } from "@stefnotch/typestef/assert";
import { shallowRef, watch, watchEffect, type Ref, type ShallowRef } from "vue";

const dummyValue = Symbol("dummyValue");
export function shallowEffectRef<T>(
  effect: (oldValue: NoInfer<T> | null) => T
): ShallowRef<T> {
  const ref = shallowRef<T>(dummyValue as any);
  watchEffect(() => {
    const oldValue = ref.value;
    ref.value = effect(oldValue === dummyValue ? null : oldValue);
  });
  assert(ref.value !== dummyValue);
  return ref;
}

// From https://stackoverflow.com/a/70005195/3492994
export function concatArrayBuffers(views: ArrayBufferView[]) {
  let length = 0;
  for (const v of views) length += v.byteLength;

  let buf = new Uint8Array(length);
  let offset = 0;
  for (const v of views) {
    const uint8view = new Uint8Array(v.buffer, v.byteOffset, v.byteLength);
    buf.set(uint8view, offset);
    offset += uint8view.byteLength;
  }

  return buf;
}

export class SmartStorageBuffer {
  buffer: StorageBuffer;
  initialBuffer: StorageBuffer;
  initialByteSize: number;
  constructor(
    engine: WebGPUEngine,
    initialData: {
      buffers: ArrayBufferView[];
      runtimeArray: number;
    },
    creationFlags: number,
    label: string
  ) {
    const data = concatArrayBuffers(initialData.buffers);

    this.initialByteSize = data.byteLength;
    this.buffer = new StorageBuffer(
      engine,
      data.byteLength + initialData.runtimeArray,
      creationFlags | Constants.BUFFER_CREATIONFLAG_WRITE,
      label
    );
    this.buffer.update(data);
    this.initialBuffer = new StorageBuffer(
      engine,
      data.byteLength,
      Constants.BUFFER_CREATIONFLAG_READ | Constants.BUFFER_CREATIONFLAG_WRITE,
      label
    );
    this.initialBuffer.update(data);
  }

  reset(engine: WebGPUEngine) {
    engine._renderEncoder.copyBufferToBuffer(
      this.initialBuffer.getBuffer().underlyingResource,
      0,
      this.buffer.getBuffer().underlyingResource,
      0,
      this.initialByteSize
    );
  }

  dispose() {
    this.buffer.dispose();
    this.initialBuffer.dispose();
  }
}

export type UniformBufferEntry =
  | {
      name: string;
      type: "float";
      getValue: () => number;
    }
  | {
      name: string;
      type: "vec2";
      getValue: () => [number, number];
    }
  | {
      name: string;
      type: "vec4";
      getValue: () => [number, number, number, number];
    }
  | {
      name: string;
      type: "mat4";
      getValue: () => Matrix;
    };

export function useUniformBuffer(
  engine: Ref<WebGPUEngine>,
  onRender: (callback: () => void) => void,
  entries: UniformBufferEntry[]
) {
  const buffer = shallowEffectRef<UniformBuffer>((oldBuffer) => {
    oldBuffer?.dispose();
    const buffer = new UniformBuffer(engine.value);
    for (const entry of entries) {
      if (entry.type === "float") {
        buffer.addUniform(entry.name, 1);
      } else if (entry.type === "vec2") {
        buffer.addUniform(entry.name, 2);
      } else if (entry.type === "vec4") {
        buffer.addUniform(entry.name, 4);
      } else if (entry.type === "mat4") {
        buffer.addUniform(entry.name, 4 * 4);
      }
    }
    return buffer;
  });

  for (const entry of entries) {
    if (entry.type === "float") {
      onRender(() => {
        buffer.value.updateFloat(entry.name, entry.getValue());
      });
    } else if (entry.type === "vec2") {
      onRender(() => {
        const [x, y] = entry.getValue();
        buffer.value.updateFloat2(entry.name, x, y);
      });
    } else if (entry.type === "vec4") {
      onRender(() => {
        const [x, y, z, w] = entry.getValue();
        buffer.value.updateFloat4(entry.name, x, y, z, w);
      });
    } else if (entry.type === "mat4") {
      onRender(() => {
        buffer.value.updateMatrix(entry.name, entry.getValue());
      });
    }
  }

  return {
    buffer,
    [Symbol.dispose]: () => {
      buffer.value.dispose();
    },
  };
}
