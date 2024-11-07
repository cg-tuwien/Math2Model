struct Uniforms {
    model_matrix: mat4x4f,
    view_projection_matrix: mat4x4f,
    grid_scale: f32
}
@binding(0) @group(0) var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>
}
struct VertexOutput {
    @builtin(position) pos: vec4f,
    @location(0) uv: vec2f
}

@vertex
fn vs_main(
    in: VertexInput,
) -> VertexOutput {
    var output: VertexOutput;
    output.pos = uniforms.view_projection_matrix * uniforms.model_matrix * vec4f(in.position, 1.0);
    output.uv = (uniforms.model_matrix * vec4f(in.position, 1.0)).xz;
    return output;
}

@fragment
fn fs_main(
    @location(0) uv: vec2f,
    @builtin(position) pos: vec4f
) -> @location(0) vec4f {
    let coord = uv.xy * uniforms.grid_scale * vec2f(0.5);
    let grid = abs(fract(coord - 0.5) - 0.5) / fwidth(coord) * 2.0;
    let line = min(grid.x, grid.y);

    let color = 1.0 - min(line, 1.0);
    let center = vec2f(0.5);
    let dist = length(coord - center) / 10.0;
    let fade_factor = 1.0 - dist;
    return vec4f(vec3f(1.0), color * fade_factor);
}