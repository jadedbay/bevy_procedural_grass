#import bevy_pbr::{
    forward_io::VertexOutput,
    pbr_types::StandardMaterial,
    pbr_bindings::material,
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
    mesh_functions::{get_world_from_local, mesh_position_local_to_clip},
    utils::rand_f,
}
#import bevy_render::maths::PI_2

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,

    @location(3) i_pos: vec4<f32>,
};

// struct GrassVertexOutput {
//     @builtin(position) clip_position: vec4<f32>,
//     @location(0) color: vec4<f32>,
// };

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var position = vertex.position;

    var ipos = vertex.i_pos.xyz;

    let width = 0.05 * (1.0 - pow(vertex.uv.y, 2.0)) * (0.7 + (1.0 - 0.7) * vertex.uv.y);
    position.x *= width;

    let p0 = vec3<f32>(0.0);
    let p2 = vec3<f32>(0.0, 0.6, 1.0);
    var curve = 0.5;
    var midpoint = 0.5;

    let p1 = vec3<f32>(0.0, curve, midpoint);
    position = quadratic_bezier(vertex.uv.y, p0, p1, p2);
    position.x += vertex.position.x * width;

    let tangent = normalize(bezier_tangent(vertex.uv.y, p0, p1, p2));
    let binormal = normalize(cross(vertex.normal, tangent));
    var normal = normalize(cross(tangent, binormal));

    let normal_curve_factor = (vertex.uv.x - 0.5) * 0.7; 
    normal = normalize(normal + binormal * normal_curve_factor);

    // Calculate facing angle and direction
    var state = bitcast<u32>(vertex.i_pos.x * 100.0 + vertex.i_pos.y * 20.0 + vertex.i_pos.z * 2.0);
    let facing_angle: f32 = rand_f(&state) * PI_2;
    // let facing_angle = material.diffuse_transmission;
    let facing = vec2<f32>(cos(facing_angle), sin(facing_angle));

    position = rotate(position, facing);

    normal = rotate(normal, facing);

    position += ipos;
    
    var out: VertexOutput;
    out.position = mesh_position_local_to_clip(
        identity_matrix,
        vec4<f32>(position, 1.0)
    );
    out.world_position = vec4<f32>(position, 1.0);

    
    out.world_normal = normal;

    out.uv = vertex.uv;

    return out;
}

@fragment fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> @location(0) vec4<f32> {
    let pbr_input = pbr_input_from_standard_material(in, is_front);
    var color = apply_pbr_lighting(pbr_input);
    color = main_pass_post_lighting_processing(pbr_input, color);
    return color;

    // var normal = pbr_input.world_normal;
    // return vec4<f32>(normal, 1.0);
}

const identity_matrix: mat4x4<f32> = mat4x4<f32>(
    vec4<f32>(1.0, 0.0, 0.0, 0.0),
    vec4<f32>(0.0, 1.0, 0.0, 0.0),
    vec4<f32>(0.0, 0.0, 1.0, 0.0),
    vec4<f32>(0.0, 0.0, 0.0, 1.0)
);

fn rotate(v: vec3<f32>, direction: vec2<f32>) -> vec3<f32> {
    let angle = atan2(direction.y, direction.x);
    let rotation_matrix = mat3x3<f32>(
        cos(angle), 0.0, -sin(angle),
        0.0, 1.0, 0.0,
        sin(angle), 0.0, cos(angle)
    );

    return rotation_matrix * v;
}

fn quadratic_bezier(t: f32, p0: vec3<f32>, p1: vec3<f32>, p2: vec3<f32>) -> vec3<f32> {
    let u = 1.0 - t;
    let tt = t * t;
    let uu = u * u;

    var p = uu * p0;
    p = p + 2.0 * u * t * p1;
    p = p + tt * p2;

    return p;
}

fn bezier_tangent(t: f32, p0: vec3<f32>, p1: vec3<f32>, p2: vec3<f32>) -> vec3<f32> {
    let u = 1.0 - t;
    
    let tangent = 2.0 * u * (p1 - p0) + 2.0 * t * (p2 - p1);
    
    return tangent;
}

fn rotate_around_axis(v: vec3<f32>, axis: vec3<f32>, angle: f32) -> vec3<f32> {
    let cos_angle = cos(angle);
    let sin_angle = sin(angle);
    return v * cos_angle + cross(axis, v) * sin_angle + axis * dot(axis, v) * (1.0 - cos_angle);
}