// This is the simplest design, where each virtual model has its own set of resources.

struct RenderStep {
    indirect_draw_buffers: TypedBuffer<copy_patches::DrawIndexedBuffers>,
    bind_group_0: shader::bind_groups::BindGroup0,
    bind_group_1: Vec<shader::bind_groups::BindGroup1>,
}
struct ComputePatchesStep {
    compute_patches_input_buffer: TypedBuffer<compute_patches::InputBuffer>,
    patches_buffer: [TypedBuffer<compute_patches::Patches>; 2],
    render_buffer: Vec<TypedBuffer<compute_patches::RenderBuffer>>,
    indirect_compute_buffer: TypedBuffer<compute_patches::DispatchIndirectArgs>,

    bind_group_0: compute_patches::bind_groups::BindGroup0,
    bind_group_1: compute_patches::bind_groups::BindGroup1,
    bind_group_2: [compute_patches::bind_groups::BindGroup2; 2],
}
struct CopyPatchesStep {
    bind_group_0: copy_patches::bind_groups::BindGroup0,
}

struct VirtualModel {}
