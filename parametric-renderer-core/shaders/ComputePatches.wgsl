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

////#include "./EvaluateImage.wgsl"
//// AUTOGEN 8960c43e220676f35c377822f4f6c2d82fd28b95f1106cfe67927c591c81d815
struct Time {
  elapsed: f32,
  delta: f32,
  frame: u32,
};
struct Screen {
  resolution: vec2<f32>,
  inv_resolution: vec2<f32>,
};
struct Mouse {
  pos: vec2<f32>,
  prev_pos: vec2<f32>,
  buttons: u32,
};
fn mouse_held(button: u32) -> bool {
  return (mouse.buttons & button) != 0u;
}
// Group 0 is for constants that change once per frame at most
@group(0) @binding(0) var<uniform> time : Time;
@group(0) @binding(1) var<uniform> screen : Screen;
@group(0) @binding(2) var<uniform> mouse : Mouse;

fn evaluateImage(input2: vec2f) -> vec3f {
    return vec3f(input2, 0.0);
}
//// END OF AUTOGEN


struct InputBuffer {
    model_view_projection: mat4x4<f32>,
    threshold_factor: f32
};

// Group 1 is for things that change once per model
// TODO: Read back the patches_length on the CPU to know when we're going out of bounds
// (And how far out of bounds)
@group(1) @binding(0) var<uniform> input_buffer : InputBuffer;
@group(1) @binding(1) var<storage, read_write> render_buffer_2 : RenderBuffer;
@group(1) @binding(2) var<storage, read_write> render_buffer_4 : RenderBuffer;
@group(1) @binding(3) var<storage, read_write> render_buffer_8 : RenderBuffer;
@group(1) @binding(4) var<storage, read_write> render_buffer_16 : RenderBuffer;
@group(1) @binding(5) var<storage, read_write> render_buffer_32 : RenderBuffer;
// Group 2 is for things that change multiple times per model
@group(2) @binding(0) var<storage, read_write> dispatch_next : DispatchIndirectArgs;
@group(2) @binding(1) var<storage, read> patches_from_buffer : PatchesRead;
@group(2) @binding(2) var<storage, read_write> patches_to_buffer : Patches;

fn triangle_area(a: vec3f, b: vec3f, c: vec3f) -> f32 {
  return 0.5 * length(cross(b - a, c - a));
}

// 8 samples in the X direction
const U_X = 8u;
// and repeat that 4 times
const U_Y = 4u;
const WORKGROUP_SIZE = U_X * U_Y;

// A vec2 with screen space coordinates
alias vec2Screen = vec2<f32>;

var<workgroup> u_samples: array<array<vec2Screen, U_X>, U_Y>;
var<workgroup> v_samples: array<array<vec2Screen, U_X>, U_Y>;
const U_LENGTHS_X = U_X - 1; // Last sample per row doesn't have a next sample
var<workgroup> u_lengths: array<array<f32, U_LENGTHS_X>, U_Y>;
var<workgroup> v_lengths: array<array<f32, U_LENGTHS_X>, U_Y>;

/// Split the patch and write it to the output buffers
fn split_patch(quad: Patch, u_length: array<f32, U_Y>, v_length: array<f32, U_Y>) {
  // We use threshold_32, because after that, we don't need to split anymore.
  // Instead, we need to compute the correct render buffer to write to.
  let threshold_32 = (32.0 * screen.inv_resolution) * input_buffer.threshold_factor;

  let split_top = u_length[0] > threshold_32.x || u_length[1] > threshold_32.x;
  let split_bottom = u_length[2] > threshold_32.x || u_length[3] > threshold_32.x;
  let split_left = v_length[0] > threshold_32.y || v_length[1] > threshold_32.y;
  let split_right = v_length[2] > threshold_32.y || v_length[3] > threshold_32.y;

  let patch_center = mix(quad.min, quad.max, 0.5);

  let patch_top = Patch(quad.min, vec2(quad.max.x, patch_center.y));
  let patch_bottom = Patch(vec2(quad.min.x, patch_center.y), quad.max);
  let patch_left = Patch(quad.min, vec2(patch_center.x, quad.max.y));
  let patch_right = Patch(vec2(patch_center.x, quad.min.y), quad.max);

  let patch_top_left = Patch(quad.min, patch_center);
  let patch_top_right = Patch(vec2(patch_center.x, quad.min.y), vec2(quad.max.x, patch_center.y));
  let patch_bottom_right = Patch(patch_center, quad.max);
  let patch_bottom_left = Patch(vec2(quad.min.x, patch_center.y), vec2(patch_center.x, quad.max.y));

  // TODO: @Stefan: Add D:/master/optimal-splits.xopp with the list
  let splits_bitflags = (u32(split_top) << 3) | (u32(split_bottom) << 2) | (u32(split_left) << 1) | u32(split_right);
  if (splits_bitflags == 0) {
    // 0000 => render it (only case where we don't split further)
    let max_u_length = max(max(u_length[0], u_length[1]), max(u_length[2], u_length[3]));
    let max_v_length = max(max(v_length[0], v_length[1]), max(v_length[2], v_length[3]));

    let threshold_16 = (16.0 * screen.inv_resolution) * input_buffer.threshold_factor;
    let threshold_8 = (8.0 * screen.inv_resolution) * input_buffer.threshold_factor;
    let threshold_4 = (4.0 * screen.inv_resolution) * input_buffer.threshold_factor;
    let threshold_2 = (2.0 * screen.inv_resolution) * input_buffer.threshold_factor;

    if (max_u_length > threshold_16.x || max_v_length > threshold_16.y) {
      let write_index = atomicAdd(&render_buffer_32.patches_length, 1u);
      if (write_index < render_buffer_32.patches_capacity) {
        render_buffer_32.patches[write_index] = quad;
      }
    } else if (max_u_length > threshold_8.x || max_v_length > threshold_8.y) {
      let write_index = atomicAdd(&render_buffer_16.patches_length, 1u);
      if (write_index < render_buffer_16.patches_capacity) {
        render_buffer_16.patches[write_index] = quad;
      }
    } else if (max_u_length > threshold_4.x || max_v_length > threshold_4.y) {
      let write_index = atomicAdd(&render_buffer_8.patches_length, 1u);
      if (write_index < render_buffer_8.patches_capacity) {
        render_buffer_8.patches[write_index] = quad;
      }
    } else if (max_u_length > threshold_2.x || max_v_length > threshold_2.y) {
      let write_index = atomicAdd(&render_buffer_4.patches_length, 1u);
      if (write_index < render_buffer_4.patches_capacity) {
        render_buffer_4.patches[write_index] = quad;
      }
    } else {
      let write_index = atomicAdd(&render_buffer_2.patches_length, 1u);
      if (write_index < render_buffer_2.patches_capacity) {
        render_buffer_2.patches[write_index] = quad;
      }
    }
  } else if (splits_bitflags == 8 || splits_bitflags == 4 || splits_bitflags == 12) {
    // 1000, 0100, 1100 => Split along the U axis
    let write_index = atomicAdd(&patches_to_buffer.patches_length, 2u);
    if write_index + 2 < patches_to_buffer.patches_capacity {
      atomicAdd(&dispatch_next.x, 2u);
      patches_to_buffer.patches[write_index + 0] = patch_left;
      patches_to_buffer.patches[write_index + 1] = patch_right;
    }
  } else if (splits_bitflags == 2 || splits_bitflags == 1 || splits_bitflags == 3) {
    // 0010, 0001, 0011 => Split along the V axis
    let write_index = atomicAdd(&patches_to_buffer.patches_length, 2u);
    if write_index + 2 < patches_to_buffer.patches_capacity {
      atomicAdd(&dispatch_next.x, 2u);
      patches_to_buffer.patches[write_index + 0] = patch_top;
      patches_to_buffer.patches[write_index + 1] = patch_bottom;
    }
  } else if(splits_bitflags == 14 || splits_bitflags == 10) {
    // 1110 => T-split
    // 1010 => Ambiguous T-split (1110 or 1011)
    let write_index = atomicAdd(&patches_to_buffer.patches_length, 3u);
    if write_index + 3 < patches_to_buffer.patches_capacity {
      atomicAdd(&dispatch_next.x, 3u);
      patches_to_buffer.patches[write_index + 0] = patch_right;
      patches_to_buffer.patches[write_index + 1] = patch_top_left;
      patches_to_buffer.patches[write_index + 2] = patch_bottom_left;
    }
  } else if(splits_bitflags == 13 || splits_bitflags == 5) {
    // 1101 => T-split
    // 0101 => Ambiguous T-split (1101 or 0111)
    let write_index = atomicAdd(&patches_to_buffer.patches_length, 3u);
    if write_index + 3 < patches_to_buffer.patches_capacity {
      atomicAdd(&dispatch_next.x, 3u);
      patches_to_buffer.patches[write_index + 0] = patch_left;
      patches_to_buffer.patches[write_index + 1] = patch_top_right;
      patches_to_buffer.patches[write_index + 2] = patch_bottom_right;
    }
  } else if(splits_bitflags == 11 || splits_bitflags == 9) {
    // 1011 => T-split
    // 1001 => Ambiguous T-split (1101 or 1011)
    let write_index = atomicAdd(&patches_to_buffer.patches_length, 3u);
    if write_index + 3 < patches_to_buffer.patches_capacity {
      atomicAdd(&dispatch_next.x, 3u);
      patches_to_buffer.patches[write_index + 0] = patch_top_left;
      patches_to_buffer.patches[write_index + 1] = patch_top_right;
      patches_to_buffer.patches[write_index + 2] = patch_bottom;
    }
  } else if(splits_bitflags == 7 || splits_bitflags == 6) {
    // 0111 => T-split
    // 0110 => Ambiguous T-split (1110 or 0111)
    let write_index = atomicAdd(&patches_to_buffer.patches_length, 3u);
    if write_index + 3 < patches_to_buffer.patches_capacity {
      atomicAdd(&dispatch_next.x, 3u);
      patches_to_buffer.patches[write_index + 0] = patch_top;
      patches_to_buffer.patches[write_index + 1] = patch_bottom_left;
      patches_to_buffer.patches[write_index + 2] = patch_bottom_right;
    }
  } else if(splits_bitflags == 15) {
    // 1111 => Full split
    let write_index = atomicAdd(&patches_to_buffer.patches_length, 4u);
    if write_index + 4 < patches_to_buffer.patches_capacity {
      atomicAdd(&dispatch_next.x, 4u);
      patches_to_buffer.patches[write_index + 0] = patch_top_left;
      patches_to_buffer.patches[write_index + 1] = patch_top_right;
      patches_to_buffer.patches[write_index + 2] = patch_bottom_right;
      patches_to_buffer.patches[write_index + 3] = patch_bottom_left;
    }
  }
}

// assume a single work group
@compute @workgroup_size(WORKGROUP_SIZE, 1, 1)
fn main(@builtin(workgroup_id) workgroup_id : vec3<u32>, 
        @builtin(local_invocation_id) local_invocation_id : vec3<u32>) {
  let patch_index: u32 = workgroup_id.x;
  let sample_index: u32 = local_invocation_id.x; // From 0 to 31 (WORKGROUP_SIZE - 1)
  assert(patch_index < patches_from_buffer.patches_length); // We dispatch one per patch, so this is always true.
  let quad = patches_from_buffer.patches[patch_index];
  let quad_size = quad.max - quad.min;

  let u_v_sample_index = vec2<u32>(sample_index % U_X, sample_index / U_X);
  
  // 4*8 = 32 U samples
  let u_sample_location = quad.min + vec2(
    (quad_size.x / f32(U_X)) * f32(u_v_sample_index.x),
    (quad_size.y / f32(U_Y) / 2.0) // top offset
    + (quad_size.y / f32(U_Y)) * f32(u_v_sample_index.y)
  );
  let u_sample = evaluateImage(u_sample_location);
  let u_clip_space = input_buffer.model_view_projection * vec4f(u_sample.xyz, 1.0);
  let u_screen_space = u_clip_space.xy / u_clip_space.w;
  u_samples[u_v_sample_index.y][u_v_sample_index.x] = u_screen_space;

  // 4*8 = 32 V samples
  let v_sample_location = quad.min + vec2(
    (quad_size.x / f32(U_Y) / 2.0) // left offset
    + (quad_size.x / f32(U_Y)) * f32(u_v_sample_index.y),
    (quad_size.y / f32(U_X)) * f32(u_v_sample_index.x),
  );
  let v_sample = evaluateImage(v_sample_location);
  let v_clip_space = input_buffer.model_view_projection * vec4f(v_sample.xyz, 1.0);
  let v_screen_space = v_clip_space.xy / v_clip_space.w;
  v_samples[u_v_sample_index.y][u_v_sample_index.x] = v_screen_space;

  workgroupBarrier(); // wait for u_samples and v_samples
  if (u_v_sample_index.x < U_X - 1) {
    let u_length = distance(u_samples[u_v_sample_index.y][u_v_sample_index.x], u_samples[u_v_sample_index.x + 1]);
    u_lengths[u_v_sample_index.y][u_v_sample_index.x] = u_length;
    // v might go in a different direction, but the array layout is the same
    let v_length = distance(v_samples[u_v_sample_index.y][u_v_sample_index.x], v_samples[u_v_sample_index.y][u_v_sample_index.x + 1]);
    v_lengths[u_v_sample_index.y][u_v_sample_index.x] = v_length;
  }
  workgroupBarrier(); // wait for u_lengths and v_lengths

  // TODO: Test if this is faster with barriers instead
  let u_length = array<f32, U_Y>(
    u_lengths[0][0] + u_lengths[0][1] + u_lengths[0][2] + u_lengths[0][3] + u_lengths[0][4] + u_lengths[0][5] + u_lengths[0][6] + u_lengths[0][7],
    u_lengths[1][0] + u_lengths[1][1] + u_lengths[1][2] + u_lengths[1][3] + u_lengths[1][4] + u_lengths[1][5] + u_lengths[1][6] + u_lengths[1][7],
    u_lengths[2][0] + u_lengths[2][1] + u_lengths[2][2] + u_lengths[2][3] + u_lengths[2][4] + u_lengths[2][5] + u_lengths[2][6] + u_lengths[2][7],
    u_lengths[3][0] + u_lengths[3][1] + u_lengths[3][2] + u_lengths[3][3] + u_lengths[3][4] + u_lengths[3][5] + u_lengths[3][6] + u_lengths[3][7]
  );
  let v_length = array<f32, U_Y>(
    v_lengths[0][0] + v_lengths[0][1] + v_lengths[0][2] + v_lengths[0][3] + v_lengths[0][4] + v_lengths[0][5] + v_lengths[0][6] + v_lengths[0][7],
    v_lengths[1][0] + v_lengths[1][1] + v_lengths[1][2] + v_lengths[1][3] + v_lengths[1][4] + v_lengths[1][5] + v_lengths[1][6] + v_lengths[1][7],
    v_lengths[2][0] + v_lengths[2][1] + v_lengths[2][2] + v_lengths[2][3] + v_lengths[2][4] + v_lengths[2][5] + v_lengths[2][6] + v_lengths[2][7],
    v_lengths[3][0] + v_lengths[3][1] + v_lengths[3][2] + v_lengths[3][3] + v_lengths[3][4] + v_lengths[3][5] + v_lengths[3][6] + v_lengths[3][7]
  );

  if(sample_index == 0) {
    split_patch(quad, u_length, v_length);
  }

  // TODO: Update the copy patches and the indirect dispatch buffer, now that we're computing only one patch per workgroup.
  
  // TODO: Frustum culling
  // Culling is done by checking if all samples are outside of exactly one of the frustum planes :)
  // 5*5 = 25 extra samples for frustum culling
  /*let extra_sample_index = vec2<u32>(sample_index % 5u, sample_index / 5u);
  let extra_sample_location = quad.min + vec2(
    (quad_size.x / 5.0) * f32(u_v_sample_index.x),
    (quad_size.y / 5.0) * f32(u_v_sample_index.y)
  );
  if (sample_index < 25) {
    let extra_sample = evaluateImage(extra_sample_location);
  }*/

  // Warning regarding storage barrier:
  // https://stackoverflow.com/questions/72035548/what-does-storagebarrier-in-webgpu-actually-do
}