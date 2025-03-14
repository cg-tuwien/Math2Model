struct Uniforms {
    view_projection_matrix: mat4x4f,
    background_color: vec4f,
    sun_direction: vec4f,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>
}
struct VertexOutput {
    @builtin(position) pos: vec4f,
    /// Can be used to sample a samplerCube
    @location(0) tex_coords: vec3f,
}

// TODO: Remove this code duplication
struct FragmentOutput {
  @location(0) color: vec4f,
  @location(1) object_id: u32,
}

@vertex
fn vs_main(
    in: VertexInput,
) -> VertexOutput {
    // https://learnopengl.com/Advanced-OpenGL/Cubemaps
    let pos = uniforms.view_projection_matrix * vec4f(in.position, 1.0);

    var output: VertexOutput;
    // we are using reverse-z, so the depth range is [1.0, 0.0], with 0.0 being faaaar away
    // and the GPU is about to divide by w
    output.pos = vec4f(pos.xy, 0.0, 1.0);
    output.tex_coords = in.position.xyz;
    return output;
}

@fragment
fn fs_main(
    in: VertexOutput
) -> FragmentOutput {
    let ambient_light = 1.0;
    let directional_light = pow(clamp(dot(normalize(in.tex_coords), uniforms.sun_direction.xyz), 0.0, 1.0), 3.0) * 0.4;
    let color = uniforms.background_color.rgb * (ambient_light + directional_light);
    var output: FragmentOutput;
    output.color = vec4f(color, 1.0);
    return output;
}