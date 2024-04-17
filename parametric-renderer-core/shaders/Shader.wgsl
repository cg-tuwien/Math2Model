//#include "./Common.wgsl"
// AUTOGEN 8f314de05189acdd25bff27345cda3548c9b99c7fa5df3ad72fc2781340b0546
struct Patch {
  min: vec2<f32>,
  max: vec2<f32>,
};
struct Patches {
  readStart: u32,
  readEnd: u32,
  write: atomic<u32>,
  patchesLength: u32,
  patches : array<Patch>,
};
struct PatchesRead { // Is currently needed, see https://github.com/gpuweb/gpuweb/discussions/4438
  readStart: u32,
  readEnd: u32,
  write: u32, // Same size and alignment as atomic<u32>. Should be legal, right?
  patchesLength: u32,
  patches : array<Patch>,
};
struct RenderBuffer {
  instanceCount: atomic<u32>,
  patchesLength: u32,
  patches: array<Patch>,
};
struct RenderBufferRead {
  instanceCount: u32, // Same size as atomic<u32>
  patchesLength: u32,
  patches: array<Patch>,
};
// END OF AUTOGEN

alias Vec3Padded = vec4<f32>;

struct Camera {
    view_position: Vec3Padded,
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
}
@group(0) @binding(0) var<uniform> camera: Camera;

// A point light without attenuation
struct Light {
    position: Vec3Padded,
    color: Vec3Padded
}
@group(0) @binding(1) var<uniform> light: Light;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @builtin(instance_index) instance_index: u32,
    @builtin(vertex_index) vertex_index: u32,
}

struct Model {
    model_similarity: mat4x4<f32>,
}
@group(0) @binding(2) var<uniform> model: Model;

@group(0) @binding(3) var<storage, read> renderBuffer: RenderBuffer;


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

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) world_position: vec3<f32>,
    @location(2) texture_coords: vec2<f32>,
}

@vertex
fn vs_main(
    in: VertexInput,
) -> VertexOutput {
    let quad = renderBuffer.patches[in.instance_index];

    var uv = vec2<f32>(quad.min.x, quad.min.y);
    if (in.vertex_index == 0) {
        uv = vec2<f32>(quad.min.x, quad.min.y);
    } else if (in.vertex_index == 1) {
        uv = vec2<f32>(quad.max.x, quad.min.y);
    } else if (in.vertex_index == 2) {
        uv = vec2<f32>(quad.max.x, quad.max.y);
    } else if (in.vertex_index == 3) {
        uv = vec2<f32>(quad.min.x, quad.max.y);
    }

    let pos = evaluateImage(uv);

    var out: VertexOutput;
    out.clip_position = camera.projection * camera.view * model.model_similarity * vec4<f32>(pos, 1.0);
    out.world_position = pos;
    out.texture_coords = uv;
    let normal = vec3<f32>(0.0, 0.0, 1.0); // TODO: We'll compute this later
    out.world_normal = (model.model_similarity * vec4<f32>(normal, 0.0)).xyz; // Only uniform scaling
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let object_color: vec4f = vec4f(in.texture_coords, 0.1, 1.0);

    // We're doing lighting in world space for simplicity
    let ambient_strength = 0.1;
    let ambient_color = light.color.rgb * ambient_strength;

    let light_dir = normalize(light.position.xyz - in.world_position);
    let view_dir = normalize(camera.view_position.xyz - in.world_position);
    let halfway_dir = normalize(light_dir + view_dir);

    let diffuse_strength = max(dot(in.world_normal, light_dir), 0.0);
    let diffuse_color = light.color.rgb * diffuse_strength;

    let specular_strength = pow(max(dot(in.world_normal, halfway_dir), 0.0), 32.0);
    let specular_color = light.color.rgb * specular_strength;

    let result = (ambient_color + diffuse_color + specular_color) * object_color.rgb;

    return vec4<f32>(result, object_color.a);
}

 

 