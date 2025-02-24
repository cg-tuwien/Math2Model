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

// From https://docs.rs/wgpu/latest/wgpu/util/struct.DrawIndexedIndirectArgs.html
struct DrawIndexedIndirectArgs  {
  index_count: u32,
  instance_count: u32,
  first_index: u32,
  base_vertex: i32,
  first_instance: u32,
};

@group(0) @binding(0) var<storage, read> render_buffer_2 : RenderBufferRead;
@group(0) @binding(1) var<storage, read> render_buffer_4 : RenderBufferRead;
@group(0) @binding(2) var<storage, read> render_buffer_8 : RenderBufferRead;
@group(0) @binding(3) var<storage, read> render_buffer_16 : RenderBufferRead;
@group(0) @binding(4) var<storage, read> render_buffer_32 : RenderBufferRead;

@group(0) @binding(5) var<storage, read_write> indirect_draw: array<DrawIndexedIndirectArgs, 5>;

/// Copies the render buffer sizes to indirect draws
@compute @workgroup_size(1, 1, 1)
fn main(@builtin(global_invocation_id) global_id : vec3<u32>) {
  indirect_draw[0].instance_count = render_buffer_2.patches_length;
  indirect_draw[1].instance_count = render_buffer_4.patches_length;
  indirect_draw[2].instance_count = render_buffer_8.patches_length;
  indirect_draw[3].instance_count = render_buffer_16.patches_length;
  indirect_draw[4].instance_count = render_buffer_32.patches_length;
}