//#include "./Common.wgsl"
// AUTOGEN d1af44c46a1cbb7b88eb3a40a108148e105bbe3d63aab3143845fbb6b5bb0256
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
  x: atomic<u32>,
  y: u32,
  z: u32,
} 
fn ceil_div(a: u32, b: u32) -> u32 { return (a + b - 1u) / b; }
// END OF AUTOGEN

struct InputBuffer {
    model_view_projection: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> input_buffer : InputBuffer;
@group(0) @binding(1) var<storage, read> patches_from_buffer : PatchesRead;
@group(0) @binding(2) var<storage, read_write> patches_to_buffer : Patches;
@group(0) @binding(3) var<storage, read_write> render_buffer : RenderBuffer;
@group(0) @binding(4) var<storage, read_write> dispatch_next : DispatchIndirectArgs;

//#include "./HeartSphere.wgsl"
// AUTOGEN e752278f38b5cff0b524b4eac45aa8fe29236e32e79fa3d6bca5a871d21478e8
fn evaluateImage(input2: vec2f) -> vec3f {
    let pos = vec3(input2.x, 0.0, 2. * input2.y) * 3.14159265359;

    let x = sin(pos.x) * cos(pos.z);
    let y = sin(pos.x) * sin(pos.z);
    let z = cos(pos.x);

    let x2 = sin(pos.x) * (15. * sin(pos.z) - 4. * sin(3. * pos.z));
    let y2 = 8. * cos(pos.x);
    let z2 = sin(pos.x) * (15. * cos(pos.z) - 5. * cos(2. * pos.z) - 2. * cos(3. * pos.z) - cos(2. * pos.z));

    let sphere = vec3(x, y, z) * 3.0;
    let heart = vec3(x2, y2, z2) * 0.2;

    let p = vec3(mix(sphere, heart, 0.) * 1.);

    return p;
}
// END OF AUTOGEN

fn triangle_area(a: vec3f, b: vec3f, c: vec3f) -> f32 {
  return 0.5 * length(cross(b - a, c - a));
}

const WORKGROUP_SIZE = 64u;

// assume a single work group
@compute @workgroup_size(WORKGROUP_SIZE, 1, 1)
fn main(@builtin(global_invocation_id) global_id : vec3<u32>) {
  // TODO: Pls benchmark this compared to the previous one
  let patch_index = global_id.x;
  var final_patches_length = 0u;
  if (patch_index < patches_from_buffer.patches_length) {
    let quad = patches_from_buffer.patches[patch_index];

    let corners = array<vec3f, 4>(
      evaluateImage(vec2f(quad.min.x, quad.min.y)),
      evaluateImage(vec2f(quad.max.x, quad.min.y)),
      evaluateImage(vec2f(quad.max.x, quad.max.y)),
      evaluateImage(vec2f(quad.min.x, quad.max.y))
    );
    
    let corners_clip_space = array<vec4f, 4>(
      (input_buffer.model_view_projection * vec4f(corners[0], 1.0)),
      (input_buffer.model_view_projection * vec4f(corners[1], 1.0)),
      (input_buffer.model_view_projection * vec4f(corners[2], 1.0)),
      (input_buffer.model_view_projection * vec4f(corners[3], 1.0))
    );
    // TODO: Clipping (aka discard if outside of the frustum)
    // Answer: No clipping. We're only doing culling, cause clipping would be a pointless overkill
    // Culling is done by checking if all samples are outside of exactly one of the frustum planes :)
    let corners_screen_space = array<vec2f, 4>(
      corners_clip_space[0].xy / corners_clip_space[0].w,
      corners_clip_space[1].xy / corners_clip_space[1].w,
      corners_clip_space[2].xy / corners_clip_space[2].w,
      corners_clip_space[3].xy / corners_clip_space[3].w
    );
    let corners_for_area = array<vec3f, 4>(
      vec3f(corners_screen_space[0], 0.0),
      vec3f(corners_screen_space[1], 0.0),
      vec3f(corners_screen_space[2], 0.0),
      vec3f(corners_screen_space[3], 0.0)
    );
    var area = triangle_area(corners_for_area[0], corners_for_area[1], corners_for_area[2]) + 
    triangle_area(corners_for_area[0], corners_for_area[2], corners_for_area[3]);
    
    let size_threshold = 0.05; // TODO: Will depend on the screen resolution (and we need different thresholds for X and Y)

    // Bonus check for degenerate cases
    // TODO: Do not check in screen space. Instead, check if the function seems to be chaotic or something. (See research paper.)
    let center_spot = input_buffer.model_view_projection * vec4f(evaluateImage(mix(quad.min, quad.max, 0.5)), 1.0);
    if (distance(center_spot.xyz / center_spot.w, corners_clip_space[0].xyz / corners_clip_space[0].w) > 0.1) { // In screen space + depth
      area = 2 * size_threshold;
    }

    if (area < size_threshold * size_threshold) {
      // TODO: Write to different render buffers
      // Super duper 1x1 pixel patches => planes with 4 vertices
      // Slightly larger patches => plane with more vertices
      // etc.

      // TODO: Should we do instancing, or should we directly generate vertices here?
      // Done, please render
      let write_index = min(atomicAdd(&render_buffer.patches_length, 1u), render_buffer.patches_capacity - 1u);
      render_buffer.patches[write_index] = quad;
    } else {
      // Split in four
      // TODO: Properly handle overflow
      final_patches_length = atomicAdd(&patches_to_buffer.patches_length, 4u);
      let write_index = min(final_patches_length, patches_to_buffer.patches_capacity - 4u);

      let center = mix(quad.min, quad.max, 0.5);
      patches_to_buffer.patches[write_index + 0] = Patch(quad.min, center);
      patches_to_buffer.patches[write_index + 1] = Patch(vec2f(center.x, quad.min.y), vec2f(quad.max.x, center.y));
      patches_to_buffer.patches[write_index + 2] = Patch(center, quad.max);
      patches_to_buffer.patches[write_index + 3] = Patch(vec2f(quad.min.x, center.y), vec2f(center.x, quad.max.y));
    }
  }

  atomicMax(&dispatch_next.x, ceil_div(final_patches_length, WORKGROUP_SIZE));
  // Well this is wrong
  // See https://stackoverflow.com/questions/72035548/what-does-storagebarrier-in-webgpu-actually-do
  // 🍎🍏 was nice. Yay.
  // storageBarrier(); // Wait for all threads to finish reading (?) before continuing
}