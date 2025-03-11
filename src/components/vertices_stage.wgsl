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
  y: atomic<u32>,
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

//// START sampleObject
fn sampleObject(input: vec2f) -> vec3f { return vec3(input, 0.0); }
//// END sampleObject
//// END OF AUTOGEN

struct Model {
    model_similarity: mat4x4<f32>,
}

struct OutputBuffer {
    length: atomic<u32>,
    capacity: u32,
    // Always in groups of 4
    vertices: array<VertexOutput>
}

//@group(1) @binding(0) var<uniform> model: Model;
@group(1) @binding(0) var<storage, read> render_buffer: RenderBufferRead;
@group(1) @binding(1) var<storage, read_write> output_buffer: OutputBuffer;

@group(2) @binding(0) var<uniform> target_instance_id: u32;

struct VertexOutput {
   // TODO: Compute normal vectors
   world_position: vec3<f32>,
   texture_coords: vec2<f32>,
   vertex_instance: u32
}

var<workgroup> quad_vertices: array<VertexOutput, 4>;

@compute @workgroup_size(4, 1, 1)
fn main(
    @builtin(workgroup_id) workgroup_id : vec3<u32>,
    @builtin(local_invocation_id) local_invocation_id : vec3<u32>
) {
    // This 8192u must match the limit in indirect_dispatch_rebalance
    let patch_index: u32 = workgroup_id.x+workgroup_id.y*8192u;
    let sample_index: u32 = local_invocation_id.x; // From 0 to 3, aka the four corners
    // Get the patch
    let quad = patch_decode(render_buffer.patches[patch_index]);
    instance_id = quad.instance;
    // Coordinates of one of the corners
    var quad_point = quad.min;
    if(local_invocation_id.x == 0) {
        quad_point = quad.min;
    } else if (local_invocation_id.x == 1) {
        quad_point = vec2f(quad.min.x, quad.max.y);
    } else if (local_invocation_id.x == 2) {
        quad_point = quad.max;
    } else {
        quad_point = vec2f(quad.max.x, quad.min.y);
    }

    // Sample corner
    let pos = sampleObject(quad_point);
    let world_pos = /*model.model_similarity * */vec4<f32>(pos, 1.0);

    // Prepare output
    var out: VertexOutput;
    out.world_position = world_pos.xyz;
    //let size = calculateWorldSpaceSizeOfPatch(quad);
    out.texture_coords = quad_point.xy;//quad_point;
    out.vertex_instance = quad.instance;

    // Write to shared array and wait
    quad_vertices[local_invocation_id.x] = out;
    workgroupBarrier();

    // Finally, write to output_buffer
    if(local_invocation_id.x == 0u && quad.instance == target_instance_id) {
        let write_index = atomicAdd(&output_buffer.length, 4u);
        if (write_index < output_buffer.capacity) {
            output_buffer.vertices[write_index] = quad_vertices[0];
            output_buffer.vertices[write_index + 1u] = quad_vertices[1];
            output_buffer.vertices[write_index + 2u] = quad_vertices[2];
            output_buffer.vertices[write_index + 3u] = quad_vertices[3];
        }
    }
}
