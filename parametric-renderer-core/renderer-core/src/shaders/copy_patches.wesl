import package::common::RenderBufferRead;

// From https://docs.rs/wgpu/latest/wgpu/util/struct.DrawIndexedIndirectArgs.html
struct DrawIndexedIndirectArgs {
  index_count: u32,
  instance_count: u32,
  first_index: u32,
  base_vertex: i32,
  first_instance: u32,
}

@group(0) @binding(0) var<storage, read> render_buffer_2 : RenderBufferRead;
@group(0) @binding(1) var<storage, read> render_buffer_4 : RenderBufferRead;
@group(0) @binding(2) var<storage, read> render_buffer_8 : RenderBufferRead;
@group(0) @binding(3) var<storage, read> render_buffer_16 : RenderBufferRead;
@group(0) @binding(4) var<storage, read> render_buffer_32 : RenderBufferRead;

@group(0) @binding(5) var<storage, read_write> indirect_draw: array<DrawIndexedIndirectArgs, 5>;

/// Copies the render buffer sizes to indirect draws
@compute @workgroup_size(1, 1, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    indirect_draw[0].instance_count = render_buffer_2.patches_length;
    indirect_draw[1].instance_count = render_buffer_4.patches_length;
    indirect_draw[2].instance_count = render_buffer_8.patches_length;
    indirect_draw[3].instance_count = render_buffer_16.patches_length;
    indirect_draw[4].instance_count = render_buffer_32.patches_length;
}
