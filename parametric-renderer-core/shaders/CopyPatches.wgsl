////#include "./Common.wgsl"
//// AUTOGEN 32cdfe4ef9c02e259d48c302aae42e898b22c7302626909847da08fae73f6cac
struct Patch {
  min: vec2<f32>,
  max: vec2<f32>,
};
struct Patches {
  patches_length: atomic<u32>,
  patches_capacity: u32,
  patches : array<Patch>,
};
struct PatchesRead { // Is currently needed, see https://github.com/gpuweb/gpuweb/discussions/4438
  patches_length: u32, // Same size and alignment as atomic<u32>. Should be legal, right?
  patches_capacity: u32,
  patches : array<Patch>,
};
struct RenderBuffer {
  patches_length: atomic<u32>,
  patches_capacity: u32,
  patches: array<Patch>,
};
struct RenderBufferRead {
  patches_length: u32,
  patches_capacity: u32,
  patches: array<Patch>,
};
struct DispatchIndirectArgs { // From https://docs.rs/wgpu/latest/wgpu/util/struct.DispatchIndirectArgs.html
  x: atomic<u32>,
  y: u32,
  z: u32,
};
fn ceil_div(a: u32, b: u32) -> u32 { return (a + b - 1u) / b; }
fn assert(condition: bool) {
  // TODO: Implement this
}
//// END OF AUTOGEN

// From https://docs.rs/wgpu/latest/wgpu/util/struct.DrawIndexedIndirectArgs.html
struct DrawIndexedIndirectArgs  {
  index_count: u32,
  instance_count: u32,
  first_index: u32,
  base_vertex: i32,
  first_instance: u32,
};

struct DrawIndexedBuffers {
  indirect_draw_2: DrawIndexedIndirectArgs,
  indirect_draw_4: DrawIndexedIndirectArgs,
  indirect_draw_8: DrawIndexedIndirectArgs,
  indirect_draw_16: DrawIndexedIndirectArgs,
  indirect_draw_32: DrawIndexedIndirectArgs,
};

@group(0) @binding(0) var<storage, read> render_buffer_2 : RenderBufferRead;
@group(0) @binding(1) var<storage, read> render_buffer_4 : RenderBufferRead;
@group(0) @binding(2) var<storage, read> render_buffer_8 : RenderBufferRead;
@group(0) @binding(3) var<storage, read> render_buffer_16 : RenderBufferRead;
@group(0) @binding(4) var<storage, read> render_buffer_32 : RenderBufferRead;

@group(0) @binding(5) var<storage, read_write> indirect_draw: DrawIndexedBuffers;

/// Copies the render buffer sizes to indirect draws
@compute @workgroup_size(1, 1, 1)
fn main(@builtin(global_invocation_id) global_id : vec3<u32>) {
  indirect_draw.indirect_draw_2.instance_count = render_buffer_2.patches_length;
  indirect_draw.indirect_draw_4.instance_count = render_buffer_4.patches_length;
  indirect_draw.indirect_draw_8.instance_count = render_buffer_8.patches_length;
  indirect_draw.indirect_draw_16.instance_count = render_buffer_16.patches_length;
  indirect_draw.indirect_draw_32.instance_count = render_buffer_32.patches_length;
}