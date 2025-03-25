@group(0) @binding(0) var<storage, read_write> dispatch_next : DispatchIndirectArgs;
@group(0) @binding(1) var<storage, read> patches_from_buffer : PatchesRead;
@group(0) @binding(2) var<storage, read_write> patches_to_buffer : Patches;
@group(0) @binding(3) var<uniform> force_render: ForceRenderFlag;
@group(0) @binding(4) var<storage> debug: array<f32>;
//@group(1) @binding(0) var<storage, read_write> dispatch_this : DispatchIndirectArgs;

@group(1) @binding(0) var<uniform> target_instance_id: u32;

struct PatchesRead { // Is currently needed, see https://github.com/gpuweb/gpuweb/discussions/4438
  patches_length: u32, // Same size and alignment as atomic<u32>. Should be legal, right?
  patches_capacity: u32,
  patches: array<EncodedPatch>,
};
struct RenderBuffer {
  patches_length: atomic<u32>,
  patches_capacity: u32,
  patches: array<EncodedPatch>,
};
struct RenderBufferRead {
  patches_length: u32,
  patches_capacity: u32,
  patches: array<EncodedPatch>,
};
struct DispatchIndirectArgs { // From https://docs.rs/wgpu/latest/wgpu/util/struct.DispatchIndirectArgs.html
  x: atomic<u32>,
  y: u32,
  z: u32,
};

struct Patch {
  min: vec2<f32>,
  max: vec2<f32>,
  instance: u32
};
struct Patches {
  patches_length: atomic<u32>,
  patches_capacity: u32,
  patches: array<EncodedPatch>,
};


struct ForceRenderFlag {
  flag: u32 // if flag == 0 { false } else { true }
}
struct EncodedPatch {
  u: u32,
  v: u32,
  instance: u32
};


@compute @workgroup_size(1, 1, 1)
fn main(@builtin(workgroup_id) workgroup_id: vec3<u32>, @builtin(local_invocation_id) local_invocation_id: vec3<u32>) 
{    
    atomicStore(&patches_to_buffer.patches_length,1u);
    let myPatch = patches_from_buffer.patches[workgroup_id.x];
    if(myPatch.instance == target_instance_id)//target_instance_id)
    {
        patches_to_buffer.patches[0] = myPatch;
    }
}
