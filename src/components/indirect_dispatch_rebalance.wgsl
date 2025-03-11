
struct DispatchIndirectArgs { // From https://docs.rs/wgpu/latest/wgpu/util/struct.DispatchIndirectArgs.html
  x: u32,
  y: u32,
  z: u32,
}

fn ceil_div(a: u32, b: u32) -> u32 { return (a + b - 1u) / b; }

@group(0) @binding(0) var<storage, read_write> dispatch_next : DispatchIndirectArgs;

@compute @workgroup_size(1, 1, 1)
fn main(@builtin(workgroup_id) workgroup_id: vec3<u32>,
        @builtin(local_invocation_id) local_invocation_id: vec3<u32>) {

    const limit = 8192u;

    if(local_invocation_id.x == 0u && workgroup_id.x == 0u)
    {
        var x = dispatch_next.x;
        if(x > limit) {
            dispatch_next.y = ceil_div(x, limit);
            dispatch_next.x = limit;
        }
    }
}

// Workgroup IDs span from (0,0,0) to (group_count_x - 1, group_count_y - 1, group_count_z - 1).
// (10, 1) = 0->9, 0
// (10, 2) = 0->9, 0->1