struct EncodedPatch {
  u: u32,
  v: u32,
};

struct RenderBufferRead {
  patches_length: u32,
  patches_capacity: u32,
  patches: array<EncodedPatch>,
};

struct DispatchIndirectArgs { // From https://docs.rs/wgpu/latest/wgpu/util/struct.DispatchIndirectArgs.html
  x: u32,
  y: u32,
  z: u32,
};

@group(0) @binding(0) var<storage, read> render_buffer_2 : RenderBufferRead;
@group(0) @binding(1) var<storage, read_write> dispatch_next: DispatchIndirectArgs;

/// Copies the render buffer sizes to the indirect dispatching of the vertices stage
@compute @workgroup_size(1, 1, 1)
fn main() {
  dispatch_next.x = render_buffer_2.patches_length;
}