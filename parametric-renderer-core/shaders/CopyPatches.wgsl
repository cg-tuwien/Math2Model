//#include "./Common.wgsl"
// AUTOGEN 8da34041f064de8dc1cd234d8f42053c996d4b74ec15cf08a817af40f8985abc
struct Patch {
  min: vec2<f32>,
  max: vec2<f32>,
};

// Is currently needed, see https://github.com/gpuweb/gpuweb/discussions/4438
struct PatchesRead {
  readStart: u32,
  readEnd: u32,
  write: u32, // Same size and alignment as atomic<u32>. Should be legal, right?
  patchesLength: u32,
  patches : array<Patch>,
};

struct PatchesReadWrite {
  readStart: u32,
  readEnd: u32,
  write: atomic<u32>,
  patchesLength: u32,
  patches : array<Patch>,
};

struct RenderBuffer {
  instanceCount: atomic<u32>,
  patchesLength: u32,
  patches: array<Patch>,
};
// END OF AUTOGEN

struct IndirectDrawBuffer {
    indexOrVertexCount: u32,
    instanceCount: u32,
    firstIndexOrVertex: u32,
    tmp1: u32,
    tmp2: u32,
};

@group(0) @binding(0) var<storage, read_write> indirectDrawBuffer : IndirectDrawBuffer;
// TODO: Make this readonly https://github.com/gpuweb/gpuweb/discussions/4438
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
    indirectDrawBuffer.instanceCount = renderBufferStart + (end - start);
  }
}