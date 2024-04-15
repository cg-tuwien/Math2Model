alias Vec3Padded = vec4<f32>;

struct Camera {
    view_position: Vec3Padded,
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
}
@group(0) @binding(0)
var<uniform> camera: Camera;

// A point light without attenuation
struct Light {
    position: Vec3Padded,
    color: Vec3Padded
}
@group(0) @binding(1)
var<uniform> light: Light;

struct VertexInput {
    @location(0) position: vec3<f32>,
}
struct InstanceInput {
    /** Translation, rotation and uniform scale. Could be compressed. */
    @location(1) model_similarity_0: vec4<f32>,
    @location(2) model_similarity_1: vec4<f32>,
    @location(3) model_similarity_2: vec4<f32>,
    @location(4) model_similarity_3: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) world_position: vec3<f32>,
    // @location(0) texture_coords: vec2<f32>,
}

@vertex
fn vs_main(
    in: VertexInput,
    in_instance: InstanceInput,
) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        in_instance.model_similarity_0,
        in_instance.model_similarity_1,
        in_instance.model_similarity_2,
        in_instance.model_similarity_3
    );

    var out: VertexOutput;
    // Later down the road, we'll compute position, normal and texture coordinates based on the patches!
    out.clip_position = camera.projection * camera.view * model_matrix * vec4<f32>(in.position, 1.0);
    let normal = vec3<f32>(0.0, 0.0, 1.0);;
    out.world_normal = (model_matrix * vec4<f32>(normal, 0.0)).xyz; // Only uniform scaling
    out.world_position = in.position;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let object_color: vec4f = vec4f(0.3, 0.2, 0.1, 1.0);

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

 

 