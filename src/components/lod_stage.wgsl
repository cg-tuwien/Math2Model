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
//// AUTOGEN 026c5fd0ec94098b53d2549d4f92f2c2ffb569eb1f84c0d3e800e02ecebbaa63
struct Time {
  elapsed: f32,
  delta: f32,
  frame: u32,
};
struct Screen {
  resolution: vec2<u32>,
  inv_resolution: vec2<f32>,
};
struct Mouse {
  pos: vec2<f32>,
  buttons: u32,
};
fn mouse_held(button: u32) -> bool {
  return (mouse.buttons & button) != 0u;
}
// Group 0 is for constants that change once per frame at most
@group(0) @binding(0) var<uniform> time : Time;
@group(0) @binding(1) var<uniform> screen : Screen;
@group(0) @binding(2) var<uniform> mouse : Mouse;
var<private> instance_id: u32;

//// START sampleObject
fn sampleObject(input: vec2f) -> vec3f { return vec3(input, 0.0); }
//// END sampleObject
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


// Storage in workgroup to combine our 32 samples
var<workgroup> u_samples: array<array<vec2Screen, U_X>, U_Y>;
var<workgroup> v_samples: array<array<vec2Screen, U_X>, U_Y>;
const U_LENGTHS_X = U_X - 1; // Last sample per row doesn't have a next sample
var<workgroup> u_lengths: array<array<f32, U_LENGTHS_X>, U_Y>;
var<workgroup> v_lengths: array<array<f32, U_LENGTHS_X>, U_Y>;
var<workgroup> frustum_sides: array<u32, 25>;

fn force_render_internal(quad_encoded: EncodedPatch)
{
    let write_index = atomicAdd(&render_buffer_2.patches_length, 1u);
    if (write_index < render_buffer_2.patches_capacity) {
        render_buffer_2.patches[write_index] = quad_encoded;
    }
}

/// Split the patch and write it to the output buffers
fn split_patch(quad_encoded: EncodedPatch, quad: Patch, u_length: array<f32, U_Y>, v_length: array<f32, U_Y>) {
  let patch_top_left = patch_top_left_child(quad_encoded);
  let patch_top_right = patch_top_right_child(quad_encoded);
  let patch_bottom_right = patch_bottom_right_child(quad_encoded);
  let patch_bottom_left = patch_bottom_left_child(quad_encoded);


  let normala = calculateNormalOfPatch(patch_decode(patch_top_left));
  let normalb = calculateNormalOfPatch(patch_decode(patch_top_right));
  let normalc = calculateNormalOfPatch(patch_decode(patch_bottom_right));
  let normald = calculateNormalOfPatch(patch_decode(patch_bottom_left));
  let simab = cosinesim(normala,normalb);
  let simcd = cosinesim(normalc,normald);
  let size = calculateWorldSpaceSizeOfPatch(quad);

  let quad_point_a = quad.min;
  let quad_point_b = vec2f(quad.min.x, quad.max.y);
  let quad_point_c = quad.max;
  let quad_point_d = vec2f(quad.max.x, quad.min.y);

  let cap = sampleObject(quad_point_a);
  let cbp = sampleObject(quad_point_b);
  let ccp = sampleObject(quad_point_c);
  let cdp = sampleObject(quad_point_d);

  let avg = (cap + cbp + ccp + cdp)/4f;
  let isflat = simab+simcd > 1.8f;


  let acceptable_size = 0.1f;

  if (force_render.flag == 1u || isflat) {
//  if (force_render.flag == 1u || size < acceptable_size) {
    force_render_internal(quad_encoded);
  } else {
    // Split all 4 ways
    // +---+---+
    // |   |   |
    // +---+---+
    // |   |   |
    // +---+---+
    let write_index = atomicAdd(&patches_to_buffer.patches_length, 4u);
    if(write_index + 4 < patches_to_buffer.patches_capacity) {
      atomicAdd(&dispatch_next.x, 4u);
      patches_to_buffer.patches[write_index + 0] = patch_top_left;
      patches_to_buffer.patches[write_index + 1] = patch_top_right;
      patches_to_buffer.patches[write_index + 2] = patch_bottom_right;
      patches_to_buffer.patches[write_index + 3] = patch_bottom_left;
    }
  }
}

fn calculateWorldSpaceSizeOfPatch(quad: Patch) -> f32
{
    let quad_point_a = quad.min;
    let quad_point_b = vec2f(quad.min.x, quad.max.y);
    let quad_point_c = quad.max;
    let quad_point_d = vec2f(quad.max.x, quad.min.y);
    let cap = sampleObject(quad_point_a);
    let cbp = sampleObject(quad_point_b);
    let ccp = sampleObject(quad_point_c);
    let cdp = sampleObject(quad_point_d);
    return length(cross((cbp-cap),(cdp-cap)))/2f+
        length(cross((cbp-ccp),(cdp-ccp)))/2f;
}

fn cosinesim(v1: vec3<f32>, v2: vec3<f32>) -> f32
{
    return dot(v1,v2); //  /(length(v1)*length(v2)) // (length of both is always 1 because they are normalized) -;
}

fn calculateNormalOfPatch(p: Patch) -> vec3<f32> {
    let quad = (p);

    let quad_point_a = quad.min;
    let quad_point_b = vec2f(quad.min.x, quad.max.y);
    let quad_point_c = quad.max;
    let quad_point_d = vec2f(quad.max.x, quad.min.y);

    let cap = sampleObject(quad_point_a);
    let cbp = sampleObject(quad_point_b);
    let ccp = sampleObject(quad_point_c);
    let cdp = sampleObject(quad_point_d);

    let tnorma = normalize(cross((cbp-cap),(cdp-cap)));
    let tnormc = normalize(cross((cbp-ccp),(cdp-ccp)));
    return normalize((tnorma+tnormc)/2.0f);
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
    // calculate distances between u samples
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
    split_patch(quad_encoded, quad, u_length, v_length);
  }


  // Warning regarding storage barrier:
  // https://stackoverflow.com/questions/72035548/what-does-storagebarrier-in-webgpu-actually-do
}