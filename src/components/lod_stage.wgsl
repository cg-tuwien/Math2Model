const EPSILON = 0.0001;
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
  patches: array<EncodedPatch>,
};
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

struct LODConfigParameters {
  earlyExitMinSize: f32,
  earlyExitMaxCurvature: f32,
  acceptablePlanarity: f32,
  ignoreMinSize: u32,
  ignoreMaxCurvature: u32,
  ignorePlanarity: u32
}

struct BufferDebugInfo {
    index: atomic<u32>,
    patch_count: atomic<u32>,
    patch_infos: array<PatchInfo>
}

struct PatchInfo {
    p : Patch,
    curvature: f32,
    planarity: f32,
    size: f32,
    v1: vec4f,
    v2: vec4f
}

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

    let min_value = vec2f(
        f32(u_bits) / f32(1u << (31u - leading_zeroes_u)),
        f32(v_bits) / f32(1u << (31u - leading_zeroes_v))
    );
    let max_value = vec2f(
        f32(u_max_bits) / f32(1u << (31u - leading_zeroes_u)),
        f32(v_max_bits) / f32(1u << (31u - leading_zeroes_v))
    );
    return Patch(min_value, max_value, encoded.instance);
}

fn assert(condition: bool) {
  // TODO: Implement this
}

// --- Added helper: inverse of a 3x3 matrix ---
fn inverse3x3(m: mat3x3<f32>) -> mat3x3<f32> {
    let a = m[0][0]; let b = m[0][1]; let c = m[0][2];
    let d = m[1][0]; let e = m[1][1]; let f = m[1][2];
    let g = m[2][0]; let h = m[2][1]; let i = m[2][2];
    let det = a * (e * i - f * h) - b * (d * i - f * g) + c * (d * h - e * g);
    return mat3x3<f32>(
        vec3<f32>((e * i - f * h) / det, (c * h - b * i) / det, (b * f - c * e) / det),
        vec3<f32>((f * g - d * i) / det, (a * i - c * g) / det, (c * d - a * f) / det),
        vec3<f32>((d * h - e * g) / det, (b * g - a * h) / det, (a * e - b * d) / det)
    );
}

// A private variable to hold our computed planarity score
var<private> planarity_score: f32;

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

@group(1) @binding(0) var<uniform> input_buffer : InputBuffer;
@group(1) @binding(1) var<storage, read_write> render_buffer_2 : RenderBuffer;
@group(1) @binding(2) var<uniform> export_config: LODConfigParameters;

@group(2) @binding(0) var<storage, read_write> dispatch_next : DispatchIndirectArgs;
@group(2) @binding(1) var<storage, read> patches_from_buffer : PatchesRead;
@group(2) @binding(2) var<storage, read_write> patches_to_buffer : Patches;
@group(2) @binding(3) var<uniform> force_render: ForceRenderFlag;

@group(2) @binding(4) var<storage, read_write> debugBuffer: BufferDebugInfo;
fn triangle_area(a: vec3<f32>, b: vec3<f32>, c: vec3<f32>) -> f32 {
    return 0.5 * length(cross(b - a, c - a));
}

const U_X = 8u;
const U_Y = 4u;
const WORKGROUP_SIZE = U_X * U_Y;
alias vec2Screen = vec3<f32>;

var<workgroup> u_samples: array<array<vec2Screen, U_X>, U_Y>;
var<workgroup> v_samples: array<array<vec2Screen, U_X>, U_Y>;
const U_LENGTHS_X = U_X - 1;
var<workgroup> u_lengths: array<array<f32, U_LENGTHS_X>, U_Y>;
var<workgroup> v_lengths: array<array<f32, U_LENGTHS_X>, U_Y>;
var<workgroup> frustum_sides: array<u32, 25>;

fn force_render_internal(quad_encoded: EncodedPatch) {
    let write_index = atomicAdd(&render_buffer_2.patches_length, 1u);
    if write_index < render_buffer_2.patches_capacity {
        render_buffer_2.patches[write_index] = quad_encoded;
    }
}
fn split_patch(quad_encoded: EncodedPatch, quad: Patch, u_length: array<f32, U_Y>, v_length: array<f32, U_Y>, planarity: f32) { 
    let patch_top_left = patch_top_left_child(quad_encoded);
    let patch_top_right = patch_top_right_child(quad_encoded);
    let patch_bottom_right = patch_bottom_right_child(quad_encoded);
    let patch_bottom_left = patch_bottom_left_child(quad_encoded);

    let normala = calculateNormalOfPatch(patch_decode(patch_top_left));
    let normalb = calculateNormalOfPatch(patch_decode(patch_top_right));
    let normalc = calculateNormalOfPatch(patch_decode(patch_bottom_right));
    let normald = calculateNormalOfPatch(patch_decode(patch_bottom_left));

    let verifyNormals = length(normala)+length(normalb)+length(normalc)+length(normald);

    let simab = cosinesim(normala, normalb);
    let simcd = cosinesim(normalc, normald);
    let simac = cosinesim(normala, normalc);
    let simbd = cosinesim(normalb, normald);
    let simad = cosinesim(normala, normald);
    let simbc = cosinesim(normalb, normalc);

    // Compute additional debug info.
    let size_val = calculateWorldSpaceSizeOfPatch(quad);
    let curvature = simab+simcd+simac+simbd+simad+simbc; // curvature computation.
    // Add a debug entry into debugBuffer:
    // debugBuffer.patch_infos[debug_index].p.instance = 500u;
    // debugBuffer.patch_infos[debug_index].p.min = vec2(-10.1,-10.2);
    // debugBuffer.patch_infos[debug_index].p.max = vec2(-20.1,-20.2);
    // debugBuffer.patch_infos[debug_index].planarity = -30.0f;
    // debugBuffer.patch_infos[debug_index].size = -40.0f;
    // debugBuffer.patch_infos[debug_index].curvature = -50.0f;
    // debugBuffer.patch_infos[debug_index].v1 = vec4f(1.,2.,3,4.);
    // debugBuffer.patch_infos[debug_index].v2 = vec4f(5,6.,7.,8.);

    // --- End of debug entry addition

    let isflat = (export_config.ignoreMaxCurvature != 0u) && (curvature/6f >= export_config.earlyExitMaxCurvature);
    let totalVLength = v_length[0] + v_length[1] + v_length[2] + v_length[3];
    let totalULength = u_length[0] + u_length[1] + u_length[2] + u_length[3];
    let isSmall = (export_config.ignoreMinSize != 0u) && (totalULength + totalVLength < export_config.earlyExitMinSize && totalULength + totalVLength > EPSILON/100f);

    let planarityThreshold = export_config.acceptablePlanarity;
    

    let firstStep = abs(quad.max.x-quad.min.x) >= 0.99f;
    let shouldRender = isSmall ||
      (
        (
            isflat && (verifyNormals > 3.8f) && size_val > 0f
        )
        ||
         ((planarity >= planarityThreshold) && export_config.ignorePlanarity != 0u)
         ||
         (planarity < 0. && curvature >= 6.-EPSILON)
      );

    // debugBuffer.patch_infos[debug_index].planarity = planarity;
    // debugBuffer.patch_infos[debug_index].curvature = (simab + simcd);
    // debugBuffer.patch_infos[debug_index].size = 5f;
//     var debug_index = atomicAdd(&debugBuffer.index, 1u);
//    debugBuffer.patch_infos[debug_index] = PatchInfo(quad, curvature, planarity, max(EPSILON,size_val), vec4f(normala,0.), vec4f(verifyNormals,0f,0f,0f));
//     debugBuffer.patch_infos[debug_index].v1 = vec4f(sampleObject(quad.min),0.);
//     debugBuffer.patch_infos[debug_index].v2 = vec4f(sampleObject(quad.max),55.);
//     atomicAdd(&debugBuffer.patch_count, 1u);

    
    // var p = getWorldPatch(quad);

    // var x = p[0].x;
    // var y=  p[0].y;
    // var z = p[0].z;
    // debugBuffer.patch_infos[debug_index] = PatchInfo(quad, x,y,z);
    // atomicAdd(&debugBuffer.patch_count, 1u);
    
    // debug_index = atomicAdd(&debugBuffer.index, 1u);
    // p = getWorldPatch(quad);

    // x = p[2].x;
    // y=  p[2].y;
    // z = p[2].z;
    // debugBuffer.patch_infos[debug_index] = PatchInfo(quad, x,y,z);
    // atomicAdd(&debugBuffer.patch_count, 1u);
    
    if force_render.flag == 1u || (shouldRender) {
        force_render_internal(quad_encoded);
    } else {
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


fn calculateWorldSpaceSizeOfPatch(quad: Patch) -> f32 {
    let quad_point_a = quad.min;
    let quad_point_b = vec2f(quad.min.x, quad.max.y);
    let quad_point_c = quad.max;
    let quad_point_d = vec2f(quad.max.x, quad.min.y);
    let cap = sampleObject(quad_point_a);
    let cbp = sampleObject(quad_point_b);
    let ccp = sampleObject(quad_point_c);
    let cdp = sampleObject(quad_point_d);
    return length(cross((cbp - cap), (cdp - cap))) / 2f + length(cross((cbp - ccp), (cdp - ccp))) / 2f;
}

fn cosinesim(v1: vec3<f32>, v2: vec3<f32>) -> f32 {
    
    return max(-100f,(dot(v1, v2) + 1.) / 2.);
}

fn getWorldPatch(p: Patch) -> mat4x3<f32> {
    let quad = p;
    let quad_point_a = quad.min;
    let quad_point_b = vec2f(quad.min.x, quad.max.y);
    let quad_point_c = quad.max;
    let quad_point_d = vec2f(quad.max.x, quad.min.y);

    let cap = sampleObject(quad_point_a);
    let cbp = sampleObject(quad_point_b);
    let ccp = sampleObject(quad_point_c);
    let cdp = sampleObject(quad_point_d);

    let wp = mat4x3(cap,cbp,ccp,cdp);

    return wp;
}

fn calculateNormalOfPatch(p: Patch) -> vec3<f32> {
    let quad = p;
    let quad_point_a = quad.min;
    let quad_point_b = vec2f(quad.min.x, quad.max.y);
    let quad_point_c = quad.max;
    let quad_point_d = vec2f(quad.max.x, quad.min.y);

    let cap = sampleObject(quad_point_a);
    let cbp = sampleObject(quad_point_b);
    let ccp = sampleObject(quad_point_c);
    let cdp = sampleObject(quad_point_d);

    // Calculate two vectors lying on the plane
    let v1 = cbp - cap;  // Edge from A to B
    let v2 = cdp - cap;  // Edge from A to D

    // Compute the cross product of the two vectors to get the normal
    let normal = normalize(cross(v1, v2));

    return normal;  // Return the calculated normal
}

@compute @workgroup_size(WORKGROUP_SIZE, 1, 1)
fn main(@builtin(workgroup_id) workgroup_id: vec3<u32>,
    @builtin(local_invocation_id) local_invocation_id: vec3<u32>) {
    // This 8192u must match the limit in indirect_dispatch_rebalance
    let patch_index: u32 = workgroup_id.x+workgroup_id.y*8192u; 
    let sample_index: u32 = local_invocation_id.x; // 0 to 31
    if(patch_index >= patches_from_buffer.patches_length) {return;}
    //assert(patch_index < patches_from_buffer.patches_length);
    let quad_encoded = patches_from_buffer.patches[patch_index];
    let quad = patch_decode(patches_from_buffer.patches[patch_index]);
    let quad_size = quad.max - quad.min;
    //instance_id = quad_encoded.instance;
    let u_v_sample_index = vec2<u32>(sample_index % U_X, sample_index / U_X);

    let u_sample_location = quad.min + vec2(
        (quad_size.x / f32(U_X - 1)) * f32(u_v_sample_index.x),
        (quad_size.y / f32(U_Y) / 2.0) + (quad_size.y / f32(U_Y)) * f32(u_v_sample_index.y)
    );
    let u_sample = sampleObject(u_sample_location);
    u_samples[u_v_sample_index.y][u_v_sample_index.x] = u_sample;

    let v_sample_location = quad.min + vec2(
        (quad_size.x / f32(U_Y) / 2.0) + (quad_size.x / f32(U_Y)) * f32(u_v_sample_index.y),
        (quad_size.y / f32(U_X - 1)) * f32(u_v_sample_index.x)
    );
    let v_sample = sampleObject(v_sample_location);
    let v_clip_space = input_buffer.model_view_projection * vec4f(v_sample.xyz, 1.0);
    let v_screen_space = v_clip_space.xy / v_clip_space.w;
    v_samples[u_v_sample_index.y][u_v_sample_index.x] = v_sample;

    workgroupBarrier();
    if u_v_sample_index.x < U_X - 1 {
        let u_len = distance(u_samples[u_v_sample_index.y][u_v_sample_index.x],
            u_samples[u_v_sample_index.y][u_v_sample_index.x + 1]);
        u_lengths[u_v_sample_index.y][u_v_sample_index.x] = u_len;
        let v_len = distance(v_samples[u_v_sample_index.y][u_v_sample_index.x],
            v_samples[u_v_sample_index.y][u_v_sample_index.x + 1]);
        v_lengths[u_v_sample_index.y][u_v_sample_index.x] = v_len;
    }
    workgroupBarrier();
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

    if sample_index == 0 {
        
    // --- Compute planarity from the point cloud in u_samples ---
        // Sum up all the points
        var sum_points: vec3<f32> = vec3<f32>(0.0, 0.0, 0.0);
        for (var y: u32 = 0u; y < U_Y; y = y + 1u) {
            for (var x: u32 = 0u; x < U_X; x = x + 1u) {
                sum_points = sum_points + u_samples[y][x];
            }
        }
        // Get centroid by calculating average point of the point cloud
        let local_centroid = sum_points / f32(U_X * U_Y);
        var cov: mat3x3<f32> = mat3x3<f32>(
            vec3<f32>(0.0, 0.0, 0.0),
            vec3<f32>(0.0, 0.0, 0.0),
            vec3<f32>(0.0, 0.0, 0.0)
        );

        for (var y: u32 = 0u; y < U_Y; y = y + 1u) {
            for (var x: u32 = 0u; x < U_X; x = x + 1u) {
                // Calculate difference from centroid
                let d = u_samples[y][x] - local_centroid;
                cov = cov + mat3x3<f32>(
                    vec3<f32>(d.x * d.x, d.x * d.y, d.x * d.z),
                    vec3<f32>(d.y * d.x, d.y * d.y, d.y * d.z),
                    vec3<f32>(d.z * d.x, d.z * d.y, d.z * d.z)
                );
            }
        }
        // Covariance per sample
        cov = cov * (1. / f32(U_X * U_Y));
        let invCov = inverse3x3(cov);
        var v_vec: vec3<f32> = normalize(vec3<f32>(1.0, 1.0, 1.0));
        for (var iter: u32 = 0u; iter < 10u; iter = iter + 1u) {
            v_vec = normalize(invCov * v_vec);
        }
        var lambda_inv = dot(v_vec, invCov * v_vec);
        let lambda_min = 1.0 / lambda_inv;
        let trace = cov[0][0] + cov[1][1] + cov[2][2];
        var planarity_local = 1.0 - (lambda_min);
        // wgsl hack, if planarity would be exactly one, aka a perfect plane, the function breaks down and returns NaN, but min() returns the non NaN value when possible
        planarity_local = max(-1.,planarity_local); 
        planarity_score = planarity_local;
        if(patch_index <= 1000000u)
        {
            split_patch(quad_encoded, quad, u_length, v_length, planarity_local);
        }
    }
}
