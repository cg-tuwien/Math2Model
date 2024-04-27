//#include "./Common.wgsl"
// AUTOGEN c5f9275279f235c07bee40b0b5dafdb91d723a1e24c4acb363c929645456a556
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
  patches_length: u32, // Same size as atomic<u32>
  patches_capacity: u32,
  patches: array<Patch>,
};
struct DispatchIndirectArgs { // From https://docs.rs/wgpu/latest/wgpu/util/struct.DispatchIndirectArgs.html
  x: u32,
  y: u32,
  z: u32,
} 
fn ceil_div(a: u32, b: u32) -> u32 { return (a + b - 1u) / b; }
// END OF AUTOGEN

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

// Needs to match the ComputePatches workgroup size
const WORKGROUP_SIZE = 64u;

@compute @workgroup_size(WORKGROUP_SIZE, 1, 1)
fn main(@builtin(global_invocation_id) global_id : vec3<u32>) {
  let render_buffer_start = atomicLoad(&render_buffer.patches_length);
  let patch_index = global_id.x;
  if (patch_index < patches_from_buffer.patches_length) {
    let quad = patches_from_buffer.patches[patch_index];
    
    // Notice how we know where to put everything, so no need for synchronization with atomics
    let write_index = render_buffer_start + patch_index;
    render_buffer.patches[write_index] = quad;
  }

  storageBarrier();
  if(global_id.x == 0u && global_id.y == 0u && global_id.z == 0u) {
    let final_patches_length = render_buffer_start + patches_from_buffer.patches_length;
    atomicStore(&render_buffer.patches_length, final_patches_length);
    indirect_draw_buffer.instance_count = final_patches_length;
  }
  // Maybe not necessary
  storageBarrier();
}