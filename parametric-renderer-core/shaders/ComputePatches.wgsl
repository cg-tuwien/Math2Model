//#include "./Common.wgsl"
// AUTOGEN 8da34041f064de8dc1cd234d8f42053c996d4b74ec15cf08a817af40f8985abc
struct Patch {
  min: vec2<f32>,
  max: vec2<f32>,
};

// Is currently needed, see https://github.com/gpuweb/gpuweb/discussions/4438
struct PatchesRead {
  readStart: u32,
  readEnd: u32,
  write: u32, // Same size and alignment as atomic<u32>. Should be legal, right?
  patchesLength: u32,
  patches : array<Patch>,
};

struct PatchesReadWrite {
  readStart: u32,
  readEnd: u32,
  write: atomic<u32>,
  patchesLength: u32,
  patches : array<Patch>,
};

struct RenderBuffer {
  instanceCount: atomic<u32>,
  patchesLength: u32,
  patches: array<Patch>,
};
// END OF AUTOGEN

struct InputBuffer {
    modelViewProjection: mat4x4<f32>,
};

@group(0) @binding(1) var<uniform> inputBuffer : InputBuffer;
@group(0) @binding(2) var<storage, read_write> patchesBuffer : PatchesReadWrite;
@group(0) @binding(3) var<storage, read_write> renderBuffer : RenderBuffer;

//#include "./HeartSphere.wgsl"
// AUTOGEN e84d0ee6fc105ba2ac366787a1da26c344ed59204771f659ad0dda335af5c535
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

/*fn evaluateImage(input2: vec2f) -> vec3f {
    let pos = vec3(input2.x, 0.0, input2.y);
    return vec3(input2.xy, 0.);
}*/

// END OF AUTOGEN

fn triangleArea(a: vec3f, b: vec3f, c: vec3f) -> f32 {
  return 0.5 * length(cross(b - a, c - a));
}

// assume a single work group
@compute @workgroup_size(64, 1, 1)
fn main(@builtin(global_invocation_id) global_id : vec3<u32>) {
  // TODO: Do loops in compute shaders make sense?
  // Answer: Nope (pls benchmark)
  for (var i = 0u; i < 8u; i = i + 1u) {

  let patchIndex = global_id.x + patchesBuffer.readStart;
  if (patchIndex < patchesBuffer.readEnd) {
    let quad = patchesBuffer.patches[patchIndex];

    let corners = array<vec3f, 4>(
      evaluateImage(vec2f(quad.min.x, quad.min.y)),
      evaluateImage(vec2f(quad.max.x, quad.min.y)),
      evaluateImage(vec2f(quad.max.x, quad.max.y)),
      evaluateImage(vec2f(quad.min.x, quad.max.y))
    );
    
    let cornersClipSpace = array<vec4f, 4>(
      (inputBuffer.modelViewProjection * vec4f(corners[0], 1.0)),
      (inputBuffer.modelViewProjection * vec4f(corners[1], 1.0)),
      (inputBuffer.modelViewProjection * vec4f(corners[2], 1.0)),
      (inputBuffer.modelViewProjection * vec4f(corners[3], 1.0))
    );
    // TODO: Clipping (aka discard if outside of the frustum)
    // Answer: No clipping. We're only doing culling, cause clipping would be a pointless overkill
    // Culling is done by checking if all samples are outside of exactly one of the frustum planes :)
    let cornersScreenSpace = array<vec2f, 4>(
      cornersClipSpace[0].xy / cornersClipSpace[0].w,
      cornersClipSpace[1].xy / cornersClipSpace[1].w,
      cornersClipSpace[2].xy / cornersClipSpace[2].w,
      cornersClipSpace[3].xy / cornersClipSpace[3].w
    );
    let cornersForArea = array<vec3f, 4>(
      vec3f(cornersScreenSpace[0], 0.0),
      vec3f(cornersScreenSpace[1], 0.0),
      vec3f(cornersScreenSpace[2], 0.0),
      vec3f(cornersScreenSpace[3], 0.0)
    );
    var area = triangleArea(cornersForArea[0], cornersForArea[1], cornersForArea[2]) + 
    triangleArea(cornersForArea[0], cornersForArea[2], cornersForArea[3]);
    
    let sizeThreshold = 0.05; // TODO: Will depend on the screen resolution (and we need different thresholds for X and Y)

    // Bonus check for degenerate cases
    let centerSpot = inputBuffer.modelViewProjection * vec4f(evaluateImage(mix(quad.min, quad.max, 0.5)), 1.0);
    if (distance(centerSpot.xyz / centerSpot.w, cornersClipSpace[0].xyz / cornersClipSpace[0].w) > 0.1) { // In screen space + depth
      area = 2 * sizeThreshold;
    }

    if (area < sizeThreshold * sizeThreshold) {
      // TODO: Write to different render buffers
      // Super duper 1x1 pixel patches => planes with 4 vertices
      // Slightly larger patches => plane with more vertices
      // etc.

      // TODO: Should we do instancing, or should we directly generate vertices here?
      // Done, please render
      let writeIndex = min(atomicAdd(&renderBuffer.instanceCount, 1u), renderBuffer.patchesLength - 1u);
      renderBuffer.patches[writeIndex] = quad;
    } else {
      // Split in four
      // TODO: Properly handle overflow
      let writeIndex = min(atomicAdd(&patchesBuffer.write, 4u), patchesBuffer.patchesLength - 4u);

      let center = mix(quad.min, quad.max, 0.5);
      patchesBuffer.patches[writeIndex + 0] = Patch(quad.min, center);
      patchesBuffer.patches[writeIndex + 1] = Patch(vec2f(center.x, quad.min.y), vec2f(quad.max.x, center.y));
      patchesBuffer.patches[writeIndex + 2] = Patch(center, quad.max);
      patchesBuffer.patches[writeIndex + 3] = Patch(vec2f(quad.min.x, center.y), vec2f(center.x, quad.max.y));
    }
  }
  // TODO: What's the most efficient approach (multiple buffers? start-end? etc.)
  // Answer: Ping pong buffer
  storageBarrier(); // Wait for all threads to finish reading "readStart" and "readEnd"
  if(global_id.x == 0u && global_id.y == 0u && global_id.z == 0u) {
    patchesBuffer.readStart = patchesBuffer.readEnd;
    patchesBuffer.readEnd = min(patchesBuffer.readEnd + 64, atomicLoad(&patchesBuffer.write));
  }
  storageBarrier(); // Wait until the writes are done before continuing

  } // end of for loop
}