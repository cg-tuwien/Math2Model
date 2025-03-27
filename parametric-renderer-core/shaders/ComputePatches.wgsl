//// START sampleObject
fn sampleObject(input: vec2f) -> vec3f {
  let a = time;
  let b = screen;
  let c = mouse;
  let d = extra;
  return vec3(input, 0.0); 
}
//// END sampleObject
var<private> instance_id: u32;

////#include "./Common.wgsl"
//// AUTOGEN 6de14edf9918265eb2e1232f93b94c84430d0069898214379635e51c3d4c9550
struct EncodedPatch {
  u: u32,
  v: u32,
  instance: u32
};
struct Patch {
  min: vec2<f32>,
  max: vec2<f32>,
  instance: u32
};
struct Patches {
  patches_length: atomic<u32>,
  patches_capacity: u32,
  patches : array<EncodedPatch>,
};
struct PatchesRead { // Is currently needed, see https://github.com/gpuweb/gpuweb/discussions/4438
  patches_length: u32, // Same size and alignment as atomic<u32>. Should be legal, right?
  patches_capacity: u32,
  patches : array<EncodedPatch>,
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
fn ceil_div(a: u32, b: u32) -> u32 { return (a + b - 1u) / b; }
// Inspired from https://onrendering.com/data/papers/isubd/isubd.pdf
fn patch_u_child(u: u32, child_bit: u32) -> u32 {
  return (u << 1) | (child_bit & 1);
}
fn patch_top_child(encoded: EncodedPatch) -> EncodedPatch {
  return EncodedPatch(encoded.u, patch_u_child(encoded.v, 0u), encoded.instance);
}
fn patch_bottom_child(encoded: EncodedPatch) -> EncodedPatch {
  return EncodedPatch(encoded.u, patch_u_child(encoded.v, 1u), encoded.instance);
}
fn patch_left_child(encoded: EncodedPatch) -> EncodedPatch {
  return EncodedPatch(patch_u_child(encoded.u, 0u), encoded.v, encoded.instance);
}
fn patch_right_child(encoded: EncodedPatch) -> EncodedPatch {
  return EncodedPatch(patch_u_child(encoded.u, 1u), encoded.v, encoded.instance);
}
fn patch_top_left_child(encoded: EncodedPatch) -> EncodedPatch {
  return patch_top_child(patch_left_child(encoded));
}
fn patch_top_right_child(encoded: EncodedPatch) -> EncodedPatch {
  return patch_top_child(patch_right_child(encoded));
}
fn patch_bottom_left_child(encoded: EncodedPatch) -> EncodedPatch {
  return patch_bottom_child(patch_left_child(encoded));
}
fn patch_bottom_right_child(encoded: EncodedPatch) -> EncodedPatch {
  return patch_bottom_child(patch_right_child(encoded));
}
fn patch_decode(encoded: EncodedPatch) -> Patch {
  // First we go to the implicit 1u
  let leading_zeroes_u = countLeadingZeros(encoded.u);
  let u_bits = extractBits(encoded.u, 0u, 31u - leading_zeroes_u);
  let u_max_bits = u_bits + 1u; // The end position of the patch
  let leading_zeroes_v = countLeadingZeros(encoded.v);
  let v_bits = extractBits(encoded.v, 0u, 31u - leading_zeroes_v);
  let v_max_bits = v_bits + 1u;

  // And every bit after that describes if we go left or right
  // Conveniently, this is already what binary numbers do.
  // 0b0.1 == 0.5
  // 0b0.01 == 0.25
  // 0b0.11 == 0.75
  // And that directly corresponds to how floats work: mantissa * 2^exponent
  // So we can just convert the bits to a float
  // let u = f32(u_bits) * pow(2.0, -1.0 * f32(31 - leading_zeroes_u));
  // And that's equivalent to the size of a patch, see formula below
  let min_value = vec2f(
    f32(u_bits) / f32(1u << (31u - leading_zeroes_u)),
    f32(v_bits) / f32(1u << (31u - leading_zeroes_v))
  );
  let max_value = vec2f(
    f32(u_max_bits) / f32(1u << (31u - leading_zeroes_u)),
    f32(v_max_bits) / f32(1u << (31u - leading_zeroes_v))
  );
  
  // The size of the patch is 1 / 2^(31 - leading_zeroes)
  // let u_size = 1.0 / f32(2 << (31 - leading_zeroes_u));
  // let v_size = 1.0 / f32(2 << (31 - leading_zeroes_v));
  // But we care about this_patch.max == next_patch.min, 
  // so we need to do the floating point calculations more carefully
  
  return Patch(min_value, max_value, encoded.instance);
}

fn assert(condition: bool) {
  // TODO: Implement this
}
//// END OF AUTOGEN

////#include "./EvaluateImage.wgsl"
//// AUTOGEN ced6a506909abff01241bc184a7d6c3a0bede71b7a4d6b1f07430a81dbc637e9
struct Time {
  elapsed: f32,
  delta: f32,
  frame: u32,
}
struct Screen {
  resolution: vec2<u32>,
  inv_resolution: vec2<f32>,
}
struct Mouse {
  pos: vec2<f32>,
  buttons: u32,
}
struct Extra {
  hot_value: f32
}
fn mouse_held(button: u32) -> bool {
  return (mouse.buttons & button) != 0u;
}
// Group 0 is for constants that change once per frame at most
@group(0) @binding(0) var<uniform> time : Time;
@group(0) @binding(1) var<uniform> screen : Screen;
@group(0) @binding(2) var<uniform> mouse : Mouse;
@group(0) @binding(3) var<uniform> extra : Extra;

//// END OF AUTOGEN

struct InputBuffer {
    threshold_factor: f32,
    model_view_projection: mat4x4<f32>,
};

struct ForceRenderFlag {
  flag: u32 // if flag == 0 { false } else { true }
}

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
@group(2) @binding(3) var<uniform> force_render: ForceRenderFlag;

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
var<workgroup> frustum_sides: array<u32, 25>;

/// Split the patch and write it to the output buffers
fn split_patch(quad_encoded: EncodedPatch, u_length: array<f32, U_Y>, v_length: array<f32, U_Y>) {
  // We use threshold_32, because after that, we don't need to split anymore.
  // Instead, we need to compute the correct render buffer to write to.
  let threshold_32 = (32.0 * screen.inv_resolution) * input_buffer.threshold_factor;

  let split_top = u_length[0] > threshold_32.x || u_length[1] > threshold_32.x;
  let split_bottom = u_length[2] > threshold_32.x || u_length[3] > threshold_32.x;
  let split_left = v_length[0] > threshold_32.y || v_length[1] > threshold_32.y;
  let split_right = v_length[2] > threshold_32.y || v_length[3] > threshold_32.y;

  let patch_top = patch_top_child(quad_encoded);
  let patch_bottom = patch_bottom_child(quad_encoded);
  let patch_left = patch_left_child(quad_encoded);
  let patch_right = patch_right_child(quad_encoded);

  let patch_top_left = patch_top_left_child(quad_encoded);
  let patch_top_right = patch_top_right_child(quad_encoded);
  let patch_bottom_right = patch_bottom_right_child(quad_encoded);
  let patch_bottom_left = patch_bottom_left_child(quad_encoded);

  let splits_bitflags = (u32(split_top) << 3) | (u32(split_bottom) << 2) | (u32(split_left) << 1) | u32(split_right);
  if (splits_bitflags == 0u || force_render.flag != 0u) {
    /* No splits, render the patch
    +---+---+
    |       |
    +       +
    |       |
    +---+---+
    */
    let max_u_length = max(max(u_length[0], u_length[1]), max(u_length[2], u_length[3]));
    let max_v_length = max(max(v_length[0], v_length[1]), max(v_length[2], v_length[3]));

    let threshold_16 = (16.0 * screen.inv_resolution) * input_buffer.threshold_factor;
    let threshold_8 = (8.0 * screen.inv_resolution) * input_buffer.threshold_factor;
    let threshold_4 = (4.0 * screen.inv_resolution) * input_buffer.threshold_factor;
    let threshold_2 = (2.0 * screen.inv_resolution) * input_buffer.threshold_factor;

    if (max_u_length > threshold_16.x || max_v_length > threshold_16.y) {
      let write_index = atomicAdd(&render_buffer_32.patches_length, 1u);
      if (write_index < render_buffer_32.patches_capacity) {
        render_buffer_32.patches[write_index] = quad_encoded;
      }
    } else if (max_u_length > threshold_8.x || max_v_length > threshold_8.y) {
      let write_index = atomicAdd(&render_buffer_16.patches_length, 1u);
      if (write_index < render_buffer_16.patches_capacity) {
        render_buffer_16.patches[write_index] = quad_encoded;
      }
    } else if (max_u_length > threshold_4.x || max_v_length > threshold_4.y) {
      let write_index = atomicAdd(&render_buffer_8.patches_length, 1u);
      if (write_index < render_buffer_8.patches_capacity) {
        render_buffer_8.patches[write_index] = quad_encoded;
      }
    } else if (max_u_length > threshold_2.x || max_v_length > threshold_2.y) {
      let write_index = atomicAdd(&render_buffer_4.patches_length, 1u);
      if (write_index < render_buffer_4.patches_capacity) {
        render_buffer_4.patches[write_index] = quad_encoded;
      }
    } else {
      let write_index = atomicAdd(&render_buffer_2.patches_length, 1u);
      if (write_index < render_buffer_2.patches_capacity) {
        render_buffer_2.patches[write_index] = quad_encoded;
      }
    }
  } else if (splits_bitflags == 8u || splits_bitflags == 4u || splits_bitflags == 12u) {
    /* Split top or split bottom or split top-bottom
    => Split along the U axis
    +---+---+    +---+---+   +---+---+
    |   |   |    |       |   |   |   |
    +       +    +       +   +   |   +
    |       |    |   |   |   |   |   |
    +---+---+    +---+---+   +---+---+
    */
    let write_index = atomicAdd(&patches_to_buffer.patches_length, 2u);
    if write_index + 2 < patches_to_buffer.patches_capacity {
      atomicAdd(&dispatch_next.x, 2u);
      patches_to_buffer.patches[write_index + 0] = patch_left;
      patches_to_buffer.patches[write_index + 1] = patch_right;
    }
  } else if (splits_bitflags == 2u || splits_bitflags == 1u || splits_bitflags == 3u) {
    /* Split left or split right or split left-right
    => Split along the V axis
    +---+---+    +---+---+   +---+---+
    |       |    |       |   |       |
    +---    +    +    ---+   +-------+
    |       |    |       |   |       |
    +---+---+    +---+---+   +---+---+
    */
    let write_index = atomicAdd(&patches_to_buffer.patches_length, 2u);
    if write_index + 2 < patches_to_buffer.patches_capacity {
      atomicAdd(&dispatch_next.x, 2u);
      patches_to_buffer.patches[write_index + 0] = patch_top;
      patches_to_buffer.patches[write_index + 1] = patch_bottom;
    }
  } else if(splits_bitflags == 14 || splits_bitflags == 10) {
    /* Split top-bottom-left or split top-left
    => T-split
    => Ambiguous T-split (1110 or 1011)
    +---+---+    +---+---+
    |   |   |    |   |   |
    +---+   +    +---+   +
    |   |   |    |       |
    +---+---+    +---+---+
    */
    let write_index = atomicAdd(&patches_to_buffer.patches_length, 3u);
    if write_index + 3 < patches_to_buffer.patches_capacity {
      atomicAdd(&dispatch_next.x, 3u);
      patches_to_buffer.patches[write_index + 0] = patch_right;
      patches_to_buffer.patches[write_index + 1] = patch_top_left;
      patches_to_buffer.patches[write_index + 2] = patch_bottom_left;
    }
  } else if(splits_bitflags == 13 || splits_bitflags == 5) {
    /* Split top-bottom-right or split bottom-right
    => T-split
    => Ambiguous T-split (1101 or 0111)
    +---+---+    +---+---+
    |   |   |    |       |
    +   +---+    +   +---+
    |   |   |    |   |   |
    +---+---+    +---+---+
    */
    let write_index = atomicAdd(&patches_to_buffer.patches_length, 3u);
    if write_index + 3 < patches_to_buffer.patches_capacity {
      atomicAdd(&dispatch_next.x, 3u);
      patches_to_buffer.patches[write_index + 0] = patch_left;
      patches_to_buffer.patches[write_index + 1] = patch_top_right;
      patches_to_buffer.patches[write_index + 2] = patch_bottom_right;
    }
  } else if(splits_bitflags == 11 || splits_bitflags == 9) {
    /* Split top-left-right or split top-right
    => T-split
    => Ambiguous T-split (1101 or 1011)
    +---+---+    +---+---+
    |   |   |    |   |   |
    +---+---+    +   +---+
    |       |    |       |
    +---+---+    +---+---+
    */
    let write_index = atomicAdd(&patches_to_buffer.patches_length, 3u);
    if write_index + 3 < patches_to_buffer.patches_capacity {
      atomicAdd(&dispatch_next.x, 3u);
      patches_to_buffer.patches[write_index + 0] = patch_top_left;
      patches_to_buffer.patches[write_index + 1] = patch_top_right;
      patches_to_buffer.patches[write_index + 2] = patch_bottom;
    }
  } else if(splits_bitflags == 7 || splits_bitflags == 6) {
    /* Split bottom-left-right or split bottom-left
    => T-split
    => Ambiguous T-split (1110 or 0111)
    +---+---+    +---+---+
    |       |    |       |
    +---+---+    +---+   +
    |   |   |    |   |   |
    +---+---+    +---+---+
    */
    let write_index = atomicAdd(&patches_to_buffer.patches_length, 3u);
    if write_index + 3 < patches_to_buffer.patches_capacity {
      atomicAdd(&dispatch_next.x, 3u);
      patches_to_buffer.patches[write_index + 0] = patch_top;
      patches_to_buffer.patches[write_index + 1] = patch_bottom_left;
      patches_to_buffer.patches[write_index + 2] = patch_bottom_right;
    }
  } else if(splits_bitflags == 15) {
    /*
    Split all 4 ways
    +---+---+
    |   |   |
    +---+---+
    |   |   |
    +---+---+
    */
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

/// Gets a bitflag for the frustum sides of a point in clip space. 6 bits are used, 1 for each side.
/// Based on the equations in https://carmencincotti.com/2022-05-02/homogeneous-coordinates-clip-space-ndc/#clip-space
fn get_frustum_side(point_clip_space: vec4f) -> u32 {
  return u32(
    (u32(point_clip_space.x < -point_clip_space.w) << 5u) |
    (u32(point_clip_space.x >  point_clip_space.w) << 4u) |
    (u32(point_clip_space.y < -point_clip_space.w) << 3u) |
    (u32(point_clip_space.y >  point_clip_space.w) << 2u) |
    (u32(point_clip_space.z < -point_clip_space.w) << 1u) |
    (u32(point_clip_space.z >  point_clip_space.w) << 0u)
  );
}

// assume a single work group
@compute @workgroup_size(WORKGROUP_SIZE, 1, 1)
fn main(@builtin(workgroup_id) workgroup_id : vec3<u32>, 
        @builtin(local_invocation_id) local_invocation_id : vec3<u32>) {
  let patch_index: u32 = workgroup_id.x;
  let sample_index: u32 = local_invocation_id.x; // From 0 to 31 (WORKGROUP_SIZE - 1)
  assert(patch_index < patches_from_buffer.patches_length); // We dispatch one per patch, so this is always true.
  let quad_encoded = patches_from_buffer.patches[patch_index];
  let quad = patch_decode(patches_from_buffer.patches[patch_index]);
  let quad_size = quad.max - quad.min;

  // Culling is done by checking if all samples are outside of exactly one of the frustum planes :)
  // 5*5 = 25 extra samples for frustum culling
  let extra_sample_index = vec2<u32>(sample_index % 5u, sample_index / 5u);
  let extra_sample_location = quad.min + vec2(
    // Divide by 4.0 because we have 5 samples, but we want to go from 0 to 1
    (quad_size.x / 4.0) * f32(extra_sample_index.x),
    (quad_size.y / 4.0) * f32(extra_sample_index.y)
  );
  instance_id = quad_encoded.instance;
  if (sample_index < 25) {
    let extra_sample = sampleObject(extra_sample_location);
    let extra_clip_space = input_buffer.model_view_projection * vec4f(extra_sample.xyz, 1.0);
    frustum_sides[sample_index] = get_frustum_side(extra_clip_space);
  }
  workgroupBarrier(); // wait for frustum_sides
  // Now parallel combine the frustum sides
  for (var i: u32 = 16u; i > 0u; i >>= 1u) {
    if (sample_index < i && sample_index + i < 25u) {
      frustum_sides[sample_index] &= frustum_sides[sample_index + i];
    }
    workgroupBarrier();
  }
  // frustum_sides[0] now contains the combined frustum sides for the entire patch
  if (workgroupUniformLoad(&frustum_sides[0]) != 0u) {
    return; // Skip the entire patch
  }

  let u_v_sample_index = vec2<u32>(sample_index % U_X, sample_index / U_X);
  
  // 4*8 = 32 U samples
  let u_sample_location = quad.min + vec2(
    // 8 samples divide a quad into 7 parts
    (quad_size.x / f32(U_X - 1)) * f32(u_v_sample_index.x),
    (quad_size.y / f32(U_Y) / 2.0) // top offset
    + (quad_size.y / f32(U_Y)) * f32(u_v_sample_index.y)
  );
  let u_sample = sampleObject(u_sample_location);
  let u_clip_space = input_buffer.model_view_projection * vec4f(u_sample.xyz, 1.0);
  let u_screen_space = u_clip_space.xy / u_clip_space.w;
  u_samples[u_v_sample_index.y][u_v_sample_index.x] = u_screen_space;

  // 4*8 = 32 V samples
  let v_sample_location = quad.min + vec2(
    (quad_size.x / f32(U_Y) / 2.0) // left offset
    + (quad_size.x / f32(U_Y)) * f32(u_v_sample_index.y),
    (quad_size.y / f32(U_X - 1)) * f32(u_v_sample_index.x),
  );
  let v_sample = sampleObject(v_sample_location);
  let v_clip_space = input_buffer.model_view_projection * vec4f(v_sample.xyz, 1.0);
  let v_screen_space = v_clip_space.xy / v_clip_space.w;
  v_samples[u_v_sample_index.y][u_v_sample_index.x] = v_screen_space;


  workgroupBarrier(); // wait for u_samples and v_samples
  if (u_v_sample_index.x < U_X - 1) {
    let u_length = distance(u_samples[u_v_sample_index.y][u_v_sample_index.x], u_samples[u_v_sample_index.y][u_v_sample_index.x + 1]);
    u_lengths[u_v_sample_index.y][u_v_sample_index.x] = u_length;
    // v might go in a different direction, but the array layout is the same
    let v_length = distance(v_samples[u_v_sample_index.y][u_v_sample_index.x], v_samples[u_v_sample_index.y][u_v_sample_index.x + 1]);
    v_lengths[u_v_sample_index.y][u_v_sample_index.x] = v_length;
  }
  workgroupBarrier(); // wait for u_lengths and v_lengths

  // TODO: Test if this is faster with barriers instead
  let u_length = array<f32, U_Y>(
    u_lengths[0][0] + u_lengths[0][1] + u_lengths[0][2] + u_lengths[0][3] + u_lengths[0][4] + u_lengths[0][5] + u_lengths[0][6],
    u_lengths[1][0] + u_lengths[1][1] + u_lengths[1][2] + u_lengths[1][3] + u_lengths[1][4] + u_lengths[1][5] + u_lengths[1][6],
    u_lengths[2][0] + u_lengths[2][1] + u_lengths[2][2] + u_lengths[2][3] + u_lengths[2][4] + u_lengths[2][5] + u_lengths[2][6],
    u_lengths[3][0] + u_lengths[3][1] + u_lengths[3][2] + u_lengths[3][3] + u_lengths[3][4] + u_lengths[3][5] + u_lengths[3][6]
  );
  let v_length = array<f32, U_Y>(
    v_lengths[0][0] + v_lengths[0][1] + v_lengths[0][2] + v_lengths[0][3] + v_lengths[0][4] + v_lengths[0][5] + v_lengths[0][6],
    v_lengths[1][0] + v_lengths[1][1] + v_lengths[1][2] + v_lengths[1][3] + v_lengths[1][4] + v_lengths[1][5] + v_lengths[1][6],
    v_lengths[2][0] + v_lengths[2][1] + v_lengths[2][2] + v_lengths[2][3] + v_lengths[2][4] + v_lengths[2][5] + v_lengths[2][6],
    v_lengths[3][0] + v_lengths[3][1] + v_lengths[3][2] + v_lengths[3][3] + v_lengths[3][4] + v_lengths[3][5] + v_lengths[3][6]
  );

  if(sample_index == 0) {
    split_patch(quad_encoded, u_length, v_length);
  }


  // Warning regarding storage barrier:
  // https://stackoverflow.com/questions/72035548/what-does-storagebarrier-in-webgpu-actually-do
}