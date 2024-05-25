////#include "./Common.wgsl"
//// AUTOGEN 1c0b2e57e3dd026763bb346cee213a10f0c740a7aa85a23af4416803018482e7
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
  _patches_length: u32, // Not to be used, CopyPatches will never write to this
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

@group(0) @binding(0) var<storage, read_write> indirect_draw_buffer : DrawIndexedIndirectArgs;
@group(0) @binding(1) var<storage, read> patches_from_buffer : PatchesRead;
@group(0) @binding(2) var<storage, read_write> render_buffer : RenderBuffer;

@compute @workgroup_size(1, 1, 1)
fn main(@builtin(global_invocation_id) global_id : vec3<u32>) {
  let render_buffer_start = atomicLoad(&render_buffer.patches_length);
  let patch_index = global_id.x;
  let write_index = render_buffer_start + patch_index;

  // TODO: Frustum culling

  if (patch_index < patches_from_buffer.patches_length 
      && write_index < render_buffer.patches_capacity) {
    // Notice how we know where to put everything, so no need for synchronization with atomics
    render_buffer.patches[write_index] = patches_from_buffer.patches[patch_index];
  }

  if(global_id.x == 0u && global_id.y == 0u && global_id.z == 0u) {
    var final_patches_length = render_buffer_start + patches_from_buffer.patches_length;

    if(final_patches_length > render_buffer.patches_capacity) {
      final_patches_length = render_buffer.patches_capacity;
      // TODO: write to an atomic when we run out of space
    }

    // The vertex shader will never read render_buffer.patches_length. It's not allowed to.
    // So we don't need to update render_buffer.patches_length. Not that we could, thanks to storageBarriers being useless.
    indirect_draw_buffer.instance_count = final_patches_length;
  }
}