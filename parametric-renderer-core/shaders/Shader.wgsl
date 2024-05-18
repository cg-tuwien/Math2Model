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

alias Vec3Padded = vec4<f32>;

struct Camera {
    world_position: Vec3Padded,
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
}

struct PointLight {
    // position.xyz is the position of the light in world space
    // position.w is the range of the light
    position_range: vec4<f32>,
    // color.rgb is the color of the light
    // color.a is the intensity of the light
    color_intensity: vec4<f32>,
}

struct Lights {
    ambient: Vec3Padded,
    // TODO: Directional light
    points_length: u32,
    points: array<PointLight>,
}

struct VertexInput {
    @location(0) position: vec3<f32>,
    @builtin(instance_index) instance_index: u32,
    @builtin(vertex_index) vertex_index: u32,
}

struct Model {
    model_similarity: mat4x4<f32>,
}

struct Material {
    // color.rgb is the color of the material
    // color.a is the roughness of the material
    color_roughness: vec4<f32>,
    // emissive_metallic.rgb is the emissive color of the material
    // emissive_metallic.a is the metallicness of the material
    emissive_metallic: vec4<f32>,
}

@group(0) @binding(0) var<uniform> camera: Camera;
@group(0) @binding(1) var<storage, read> lights: Lights;
@group(0) @binding(2) var<uniform> model: Model;
@group(0) @binding(3) var<storage, read> render_buffer: RenderBufferRead;
@group(0) @binding(4) var<uniform> material: Material;

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


const PI = 3.14159265359;

fn distributionGGXTrowbridgeReitz(n: vec3f, h: vec3f, alpha: f32) -> f32 {
    let alphaSquared = alpha * alpha;

    let nDoth = max(dot(n,h), 0.0);
    let nDothSquared = nDoth * nDoth;

    let partDenom = nDothSquared * (alphaSquared - 1.0) + 1.0;

    return alphaSquared / (PI * partDenom * partDenom);
}

// x: in this context only v or l are allowed to be x
fn geometrySchlickBeckmann(n: vec3f, x: vec3f, alpha: f32) -> f32 {
    let k = alpha / 2.0; // there are other options for this
    let nDotx = max(dot(n, x), 0.0);

    return nDotx / max(nDotx * (1.0 - k) + k, 0.000001);
}


fn geometrySmith(n: vec3f, v: vec3f, l: vec3f, alpha: f32) -> f32  {
    return geometrySchlickBeckmann(n, v, alpha) * geometrySchlickBeckmann(n, l, alpha);
}

fn fresnelSchlick(f0: vec3f, v: vec3f, h: vec3f) -> vec3f {
    let vDoth = max(dot(v, h), 0.0);

    return f0 + (1.0 - f0) * pow(1.0 - vDoth, 5.0);
}

fn pbr_common(lightIntensity: vec3f, l: vec3f, n: vec3f, v: vec3f, albedo: vec3f, f0: vec3f) -> vec3f {
    let h = normalize(v + l);

    let fLambert = albedo / PI;

    let alpha = material.color_roughness.a * material.color_roughness.a;

    // D: Normal Distribution Function (GGX/Trowbridge-Reitz)
    let D = distributionGGXTrowbridgeReitz(n, h, alpha);

    // G: Geometry Function (Smith Model using Schlick-Beckmann)
    let G = geometrySmith(n, v, l, alpha);

    // F: Fresnel Function
    let F = fresnelSchlick(f0, v, h);

    let fCookTorranceNumerator: vec3f = D * G * F;
    var fCookTorranceDenominator = 4.0 * max(dot(n, l), 0.0) * max(dot(n, v), 0.0);
    fCookTorranceDenominator = max(fCookTorranceDenominator, 0.000001);

    let fCookTorrance: vec3f =  fCookTorranceNumerator / fCookTorranceDenominator;

    let ks = F;
    var kd = vec3f(1.0) - ks;
    kd *= 1.0-material.emissive_metallic.a;

    let diffuseBRDF = kd * fLambert;
    let specularBRDF: vec3f = /* ks + */ fCookTorrance;
    let nDotL: f32 = max(dot(n, l), 0.0);

    return (diffuseBRDF + specularBRDF) * lightIntensity * nDotL;
}

fn pbr(pointLight: PointLight, n: vec3f, v: vec3f, worldPos: vec3f, albedo: vec3f, f0: vec3f) -> vec3f {
    let positionToLight = pointLight.position_range.xyz - worldPos;
    let l = normalize(positionToLight);
    let dSquared = max(dot(positionToLight, positionToLight), 0.000001);

    let attenuation = 1.0 / dSquared;
    let lightIntensity = pointLight.color_intensity.rgb * pointLight.color_intensity.a * attenuation;
    return pbr_common(lightIntensity, l, n, v, albedo, f0);
}



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
    let quad = render_buffer.patches[in.instance_index];

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
    let world_pos = model.model_similarity * vec4<f32>(pos, 1.0);

    var out: VertexOutput;
    out.clip_position = camera.projection * camera.view * world_pos;
    out.world_position = world_pos.xyz;
    out.texture_coords = uv;
    let normal = vec3<f32>(0.0, -1.0, 0.0); // TODO: We'll compute this later
    out.world_normal = (model.model_similarity * vec4<f32>(normal, 0.0)).xyz; // Only uniform scaling
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let world_pos = in.world_position;
    let n = normalize(in.world_normal);
    let v = normalize(camera.world_position.xyz - world_pos);

    // Debug color
    // let albedo: vec3f = vec3f(in.texture_coords, 0.1);
    let albedo: vec3f = material.color_roughness.rgb;

    // reflectance at normal incidence (base reflectance)
    // if dia-electric (like plastic) use F0 of 0.04 and if it's a metal, use the albedo as F0 (metallic workflow)
    let f0 = mix(vec3f(0.04), albedo, material.emissive_metallic.a);

    // out going light
    var Lo = vec3f(0.0);
    for (var i: u32 = 0u; i < lights.points_length; i = i + 1u) {
        Lo += pbr(lights.points[i], n, v, world_pos, albedo, f0);
    }

    let ambient: vec3f = lights.ambient.rgb * albedo;

    let color: vec3f = Lo + ambient + material.emissive_metallic.rgb;

    return vec4<f32>(pbr(lights.points[0], n, v, world_pos, albedo, f0), 1.0);
}

 

 