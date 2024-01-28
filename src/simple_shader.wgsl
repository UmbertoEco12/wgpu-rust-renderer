// Vertex shader
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct Camera {
    matrix: mat4x4<f32>,
    proj_matrix: mat4x4<f32>,
}
@group(1) @binding(0)
var<uniform> camera: Camera;

struct ModelMatrix {
    matrix: mat4x4<f32>,
}
@group(2) @binding(0)
var<uniform> model_matrix: ModelMatrix;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = camera.proj_matrix * model_matrix.matrix * vec4<f32>(model.position, 1.0);
    return out;
}


// Fragment shader

@group(0) @binding(0)
var<uniform> color: vec4<f32>;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return color;
}