struct EncodedPatch {
  u: u32,
  v: u32,
  instance: u32
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
var<private> instance_id: u32;

/// Copies the render buffer sizes to the indirect dispatching of the vertices stage
@compute @workgroup_size(1, 1, 1)
fn main() {
  
  const limit = 8192u;
  var x = render_buffer_2.patches_length;
  while(x >= limit)
  {
      dispatch_next.y+=1u;
      x-=limit;
  }
  dispatch_next.x = min(render_buffer_2.patches_length,0xFFFFu);
}