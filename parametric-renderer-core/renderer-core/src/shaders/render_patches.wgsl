import package::sample_object_dummy::{sampleObject};
import package::common::{
  RenderBufferRead,
  patch_decode,
};
import package::gltf_pbr::{
  MaterialInfo,
  getMetallicRoughnessInfo,
  getLighIntensity,
  clamped_dot,
  LightSource,
};

//// START getColor
fn getColor(input: vec2f) -> vec3f {
  if material.has_texture != 0u {
    return textureSample(t_diffuse, linear_sampler, input).rgb;
  } else {
    return material.color_roughness.rgb;
  }
}
//// END getColor


var<private> instance_id: u32;

alias Vec3Padded = vec4<f32>;

struct Camera {
    world_position: Vec3Padded,
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
}

const LIGHT_TYPE_POINT: u32 = 0;
const LIGHT_TYPE_DIRECTIONAL: u32 = 1;


struct Lights {
    ambient: Vec3Padded,
    points_length: u32,
    points: array<LightSource>,
}

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
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
    // is a boolean
    has_texture: u32
}

@group(0) @binding(3) var<uniform> camera: Camera;
@group(0) @binding(4) var<storage, read> lights: Lights;
@group(0) @binding(5) var linear_sampler: sampler;
@group(1) @binding(1) var<uniform> model: Model;
@group(1) @binding(2) var<storage, read> render_buffer: RenderBufferRead;
@group(1) @binding(3) var<uniform> material: Material;
@group(1) @binding(4) var t_diffuse: texture_2d<f32>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) world_position: vec3<f32>,
    @location(2) texture_coords: vec2<f32>,
    @location(3) color: vec4<f32>,
}

const color_options = array<vec4f,8>(
    vec4f(1.0, 0.0, 0.0, 1.0),
    vec4f(0.0, 1.0, 0.0, 1.0),
    vec4f(0.0, 0.0, 1.0, 1.0),
    vec4f(1.0, 1.0, 0.0, 1.0),
    vec4f(1.0, 0.0, 1.0, 1.0),
    vec4f(0.0, 1.0, 1.0, 1.0),
    vec4f(1.0, 1.0, 1.0, 1.0),
    vec4f(0.5, 0.5, 0.5, 1.0),
);


@vertex
fn vs_main(
    in: VertexInput,
) -> VertexOutput {
    let quad = patch_decode(render_buffer.patches[in.instance_index]);
    let quad_point = mix(quad.min, quad.max, in.uv);
    instance_id = quad.instance;
    let pos = sampleObject(quad_point);
    let world_pos = model.model_similarity * vec4<f32>(pos, 1.0);


    var out: VertexOutput;
    out.clip_position = camera.projection * camera.view * world_pos;
    out.world_position = world_pos.xyz;
    out.texture_coords = quad_point;
    let normal = vec3<f32>(0.0, -1.0, 0.0); // TODO: We'll compute this later
    out.world_normal = (model.model_similarity * vec4<f32>(normal, 0.0)).xyz; // Only uniform scaling

    let i = in.instance_index % 8;
    if (i == 0) {
    out.color = color_options[0];
    } else if (i == 1) {
    out.color = color_options[1];
    } else if (i == 2) {
    out.color = color_options[2];
    } else if (i == 3) {
    out.color = color_options[3];
    } else if (i == 4) {
    out.color = color_options[4];
    } else if (i == 5) {
    out.color = color_options[5];
    } else if (i == 6) {
    out.color = color_options[6];
    } else if (i == 7) {
    out.color = color_options[7];
    }else {
    out.color = vec4<f32>(1.0, 1.0, 1.0, 1.0);
    }
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let v = normalize(camera.world_position.xyz - in.world_position);
    // let n = normalize(in.world_normal);
    let n = normalize(-cross(dpdxFine(in.world_position), dpdyFine(in.world_position)));

    var materialInfo = MaterialInfo(
        getColor(in.texture_coords),
        vec3f(0.04),
        vec3f(1.0),
        vec3f(0.0),
        1.0,
        1.0
    );
    materialInfo = getMetallicRoughnessInfo(materialInfo, material.emissive_metallic.a, material.color_roughness.a);

    var f_diffuse = vec3f(0.0);
    var f_specular = vec3f(0.0);
    for (var i: u32 = 0u; i < lights.points_length; i += 1u) {
        let light = lights.points[i];
        var pointToLight: vec3f;
        if (light.light_type != LIGHT_TYPE_DIRECTIONAL)
        {
            pointToLight = light.position_range.xyz - in.world_position;
        }
        else
        {
            pointToLight = -light.position_range.xyz;
        }


        let l = normalize(pointToLight); // Direction from surface point to light
        let h = normalize(l + v);        // Direction of the vector between l and v, called halfway vector
        let intensity: vec3f = getLighIntensity(light, pointToLight);
        let NdotL = clamped_dot(n, l);
        if(NdotL > 0.0) {
            let NdotV = clamped_dot(n, v);
            let NdotH = clamped_dot(n, h);
            let VdotH = clamped_dot(v, h);
            f_diffuse += intensity * NdotL *  BRDF_lambertian(
                materialInfo.f0, 
                materialInfo.f90, 
                materialInfo.c_diff, 
                materialInfo.specularWeight, 
                VdotH
            );
            f_specular += intensity * NdotL * BRDF_specularGGX(
                materialInfo.f0, 
                materialInfo.f90, 
                materialInfo.alphaRoughness, 
                materialInfo.specularWeight, 
                VdotH, 
                NdotL, 
                NdotV, 
                NdotH
            );
        }
    }

    let ambient: vec3f = lights.ambient.rgb * materialInfo.baseColor;

    let color = f_diffuse * 2.0 
        + f_specular * 2.0 
        + ambient 
        + material.emissive_metallic.rgb;

    return vec4<f32>(color, 1.0);
    // return in.color; TODO: Why does this cause z-buffer fighting?
}

 

 