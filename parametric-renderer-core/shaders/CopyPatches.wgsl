//#include "./Common.wgsl"
// AUTOGEN 8f314de05189acdd25bff27345cda3548c9b99c7fa5df3ad72fc2781340b0546
struct Patch {
  min: vec2<f32>,
  max: vec2<f32>,
};
struct Patches {
  readStart: u32,
  readEnd: u32,
  write: atomic<u32>,
  patchesLength: u32,
  patches : array<Patch>,
};
struct PatchesRead { // Is currently needed, see https://github.com/gpuweb/gpuweb/discussions/4438
  readStart: u32,
  readEnd: u32,
  write: u32, // Same size and alignment as atomic<u32>. Should be legal, right?
  patchesLength: u32,
  patches : array<Patch>,
};
struct RenderBuffer {
  instanceCount: atomic<u32>,
  patchesLength: u32,
  patches: array<Patch>,
};
struct RenderBufferRead {
  instanceCount: u32, // Same size as atomic<u32>
  patchesLength: u32,
  patches: array<Patch>,
};
// END OF AUTOGEN

// From https://docs.rs/wgpu/latest/wgpu/util/struct.DrawIndexedIndirectArgs.html
struct DrawIndexedIndirectArgs  {
  index_count: u32,
  instance_count: u32,
  first_index: u32,
  base_vertex: i32,
  first_instance: u32,
};

@group(0) @binding(0) var<storage, read_write> indirectDrawBuffer : DrawIndexedIndirectArgs;
@group(0) @binding(2) var<storage, read> patchesBuffer : PatchesRead;
@group(0) @binding(3) var<storage, read_write> renderBuffer : RenderBuffer;

fn ceilDiv(a: u32, b: u32) -> u32 {
  return (a + b - 1u) / b;
}

@compute @workgroup_size(64, 1, 1)
fn main(@builtin(global_invocation_id) global_id : vec3<u32>) {
  let renderBufferStart = atomicLoad(&renderBuffer.instanceCount);

  let start = patchesBuffer.readStart;
  var end = patchesBuffer.write;
  end = min(end, min(end - start, renderBuffer.patchesLength) + start);

  // Split into 64 segments (workgroup size)
  let segmentSize = ceilDiv(end - start, 64u);
  let segmentStart = start + segmentSize * global_id.x;
  let segmentEnd = min(segmentStart + segmentSize, end);

  // Copy patches to render buffer
  // Notice how we know where to put everything, so no need for synchronization with atomics
  for (var i = segmentStart; i < segmentEnd; i = i + 1u) {
    let quad = patchesBuffer.patches[i];
    renderBuffer.patches[renderBufferStart + (i - start)] = quad;
  }

  if(global_id.x == 0u && global_id.y == 0u && global_id.z == 0u) {
    indirectDrawBuffer.instance_count = renderBufferStart + (end - start);
  }
}