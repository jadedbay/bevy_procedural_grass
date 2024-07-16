#import bevy_pbr::{
    mesh_functions::{get_world_from_local, mesh_position_local_to_clip},
    utils::rand_f,
}
#import bevy_render::maths::PI_2

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(2) uv: vec2<f32>,

    @location(3) i_pos: vec4<f32>,
    @location(4) i_normal: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var position = vertex.position;

    let width = 0.05 * (1.0 - pow(vertex.uv.y, 2.0));
    position.x *= width;

    var state = bitcast<u32>(vertex.i_pos.x * 100.0 + vertex.i_pos.y * 20.0 + vertex.i_pos.z * 2.0);
    let facing_angle: f32 = rand_f(&state) * PI_2;
    let facing = vec2<f32>(cos(facing_angle), sin(facing_angle));
    position = rotate(position, facing);

    let rotation_matrix = rotate_align(vec3<f32>(0.0, 1.0, 0.0), vertex.i_normal.xyz);
    position = rotation_matrix * position;

    position += vertex.i_pos.xyz;
    
    var out: VertexOutput;
    out.clip_position = mesh_position_local_to_clip(
        identity_matrix,
        vec4<f32>(position, 1.0)
    );
    out.color = vec4<f32>(1.0, 1.0, 0.5, 1.0);
    return out;
}

@fragment fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}

const identity_matrix: mat4x4<f32> = mat4x4<f32>(
    vec4<f32>(1.0, 0.0, 0.0, 0.0),
    vec4<f32>(0.0, 1.0, 0.0, 0.0),
    vec4<f32>(0.0, 0.0, 1.0, 0.0),
    vec4<f32>(0.0, 0.0, 0.0, 1.0)
);

fn rotate_align(v1: vec3<f32>, v2: vec3<f32>) -> mat3x3<f32> {
    let axis = cross(v1, v2);

    let cos_a = dot(v1, v2);
    let k = 1.0 / (1.0 + cos_a);

    let result = mat3x3f( 
            (axis.x * axis.x * k) + cos_a, (axis.x * axis.y * k) + axis.z, (axis.x * axis.z * k) - axis.y,
            (axis.y * axis.x * k) - axis.z, (axis.y * axis.y * k) + cos_a,  (axis.y * axis.z * k) + axis.x, 
            (axis.z * axis.x * k) + axis.y, (axis.z * axis.y * k) - axis.x, (axis.z * axis.z * k) + cos_a 
        );

    return result;
}

fn rotate(v: vec3<f32>, direction: vec2<f32>) -> vec3<f32> {
    let angle = atan2(direction.y, direction.x);
    let rotation_matrix = mat3x3<f32>(
        cos(angle), 0.0, -sin(angle),
        0.0, 1.0, 0.0,
        sin(angle), 0.0, cos(angle)
    );

    return rotation_matrix * v;
}