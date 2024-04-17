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