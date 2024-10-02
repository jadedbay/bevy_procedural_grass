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
    @location(2) uv: vec2<f32>,

    @location(3) i_pos: vec4<f32>,
};

// struct VertexOutput {
//     @builtin(position) clip_position: vec4<f32>,
//     @location(0) color: vec4<f32>,
// };

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var position = vertex.position;

    var ipos = vertex.i_pos.xyz;
    // out.color = vec4<f32>(0.0, 1.0, 0.0, 1.0);

    let width = 0.05 * (1.0 - pow(vertex.uv.y, 2.0));
    position.x *= width;

    var state = bitcast<u32>(vertex.i_pos.x * 100.0 + vertex.i_pos.y * 20.0 + vertex.i_pos.z * 2.0);
    let facing_angle: f32 = rand_f(&state) * PI_2;
    let facing = vec2<f32>(cos(facing_angle), sin(facing_angle));
    position = rotate(position, facing);

    position += ipos;
    
    var out: VertexOutput;
    out.position = mesh_position_local_to_clip(
        identity_matrix,
        vec4<f32>(position, 1.0)
    );
    out.world_position = vec4<f32>(position, 1.0);
    out.world_normal = normalize(vec3<f32>(facing.x, 0.0, facing.y));

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
    // return vec4<f32>(0.0, 1.0, 0.0, 1.0);
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