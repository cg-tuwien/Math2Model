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

// TODO: Remove this code duplication
struct FragmentOutput {
  @location(0) color: vec4f,
  @location(1) object_id: u32,
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

fn make_grid(pos: vec2f, scale: vec2f, thicc: f32) -> f32 {
    let scaled_pos = pos * scale;
    let grid = abs(fract(scaled_pos - 0.5) - 0.5) / (fwidthFine(scaled_pos) * thicc);
    let line = min(grid.x, grid.y);
    return line;
}

fn fade_from_center(coord: vec2f) -> f32 {
    // The / 50.0 should match the radius of the grid
    let dist = length(coord) / 50.0;
    return 1.0 - dist;
}

fn light_gradient(coord: vec2f) -> f32 {
    let dist = length(coord);
    let attenuation = 1.0 / (2.2 + 0.2 * dist + 0.05 * dist * dist);
    return attenuation;
}

fn grid_to_color(grid: f32) -> f32 {
    let color = 1.0 - min(grid, 1.0);
    return pow(color, 1.0 / 2.2); // Gamma correction helps a lot
}

@fragment
fn fs_main(
    @location(0) uv: vec2f,
    @builtin(position) pos: vec4f
) -> FragmentOutput {
    let coord = uv.xy * uniforms.grid_scale;

    let large_grid = make_grid(coord, vec2f(0.5), 0.6);
    let small_grid = make_grid(coord, vec2f(4.0), 0.5);
    var color = grid_to_color(large_grid) * 0.1;
    // See https://madebyevan.com/shaders/grid/
    color += grid_to_color(small_grid) * 0.05;

    let fade_factor = fade_from_center(coord);

    let gradient = vec4f(vec3f(1.0), light_gradient(coord));
    var output: FragmentOutput;
    output.color = gradient + vec4f(vec3f(1.0), color * fade_factor);
    return output;
}

