// Vertex shader
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) tangent: vec4<f32>,
    @location(4) bone_ids: vec4<f32>,
    @location(5) weights: vec4<f32>,
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

const MAX_BONES: i32 = 100;
@group(3) @binding(0)
var<uniform> bone_matrices: array<mat4x4<f32>, MAX_BONES>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_position: vec3<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.world_normal = model.normal;
    // Calculate bone transformation
    var bone_transform: mat4x4<f32> = mat4x4<f32>();
    // Check if any bone influences are present
    if (model.weights.x > 0.0 || model.bone_ids.x > -1.0 ||
        model.weights.y > 0.0 || model.bone_ids.y > -1.0 ||
        model.weights.z > 0.0 || model.bone_ids.z > -1.0 ||
        model.weights.w > 0.0 || model.bone_ids.w > -1.0) {
        
        // Apply bone transformations
        if (model.weights.x > 0.0 && model.bone_ids.x > -1.0) {
            bone_transform = bone_transform + model.weights.x * bone_matrices[u32(model.bone_ids.x)];
        }
        if (model.weights.y > 0.0 && model.bone_ids.y > -1.0) {
            bone_transform = bone_transform + model.weights.y * bone_matrices[u32(model.bone_ids.y)];
        }
        if (model.weights.z > 0.0 && model.bone_ids.z > -1.0) {
            bone_transform = bone_transform + model.weights.z * bone_matrices[u32(model.bone_ids.z)];
        }
        if (model.weights.w > 0.0 && model.bone_ids.w > -1.0) {
            bone_transform = bone_transform + model.weights.w * bone_matrices[u32(model.bone_ids.w)];
        }
    } else {
        // Set to identity matrix if no bone influences
        bone_transform = mat4x4<f32>(
            vec4<f32>(1.0, 0.0, 0.0, 0.0),
            vec4<f32>(0.0, 1.0, 0.0, 0.0),
            vec4<f32>(0.0, 0.0, 1.0, 0.0),
            vec4<f32>(0.0, 0.0, 0.0, 1.0)
        );
    }
    var total_position: vec4<f32> = bone_transform * vec4<f32>(model.position, 1.0);
    //var total_position: vec4<f32> = vec4<f32>(model.position, 1.0);
    out.world_position = total_position.xyz;
    out.clip_position = camera.proj_matrix * model_matrix.matrix * total_position;
    return out;
}


// Fragment shader

// @group(0) @binding(0)
// var<uniform> color: vec4<f32>;

struct Light {
    position: vec3<f32>,
    color: vec3<f32>,
}
@group(0) @binding(0)
var<uniform> light: Light;

// @fragment
// fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
//     return vec4<f32>(in.tex_coords, 0.0, 1.0);
// }

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let object_color: vec4<f32> = vec4<f32>(1.0,1.0,1.0,1.0);
    
    // Simplify lighting to get a toon shading effect
    let light_dir = normalize(light.position - in.world_position);
    let diffuse_strength = max(dot(in.world_normal, light_dir), 0.0);

    // Use smoothstep for smoother transitions between shadow and light
    let threshold_min = 0.4; // Adjust as needed
    let threshold_max = 0.6; // Adjust as needed
    let toon_diffuse = smoothstep(threshold_min, threshold_max, diffuse_strength);

    // Quantize the result to create a stylized look
    let toon_color = vec3<f32>(1.0, 1.0, 1.0); // White color for objects in light
    let result = mix(vec3<f32>(0.3), toon_color, toon_diffuse) * object_color.xyz;
    //return object_color;
    return vec4<f32>(result, object_color.a);
}