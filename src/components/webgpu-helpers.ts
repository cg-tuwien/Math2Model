export function createHelpers(device: GPUDevice) {
  function concatArrayBuffers(views: ArrayBufferView[]): Uint8Array {
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

  function createBufferWith(
    contents: Uint8Array,
    usage: GPUBufferUsageFlags,
    size?: number
  ) {
    const buffer = device.createBuffer({
      size: size ?? contents.byteLength,
      usage,
      mappedAtCreation: true,
    });
    new Uint8Array(buffer.getMappedRange()).set(contents);
    buffer.unmap();
    return buffer;
  }

  return {
    concatArrayBuffers,
    createBufferWith,
  };
}
