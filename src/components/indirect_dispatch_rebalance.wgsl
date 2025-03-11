
struct DispatchIndirectArgs { // From https://docs.rs/wgpu/latest/wgpu/util/struct.DispatchIndirectArgs.html
  x: atomic<u32>,
  y: atomic<u32>,
  z: u32,
};

@group(0) @binding(0) var<storage, read_write> dispatch_next : DispatchIndirectArgs;

@compute @workgroup_size(1, 1, 1)
fn main(@builtin(workgroup_id) workgroup_id: vec3<u32>,
    @builtin(local_invocation_id) local_invocation_id: vec3<u32>) {
        if(local_invocation_id.x == 0u && workgroup_id.x == 0u)
        {
            var x = atomicLoad(&dispatch_next.x);
            while(x >= 65536u)
            {
                atomicAdd(&dispatch_next.y,1u);
                x-=65536u;
            }
            atomicMin(&dispatch_next.x,0xFFFFu);
        }
}