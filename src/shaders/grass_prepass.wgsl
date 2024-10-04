#import bevy_pbr::{
    prepass_io::VertexOutput,
    pbr_bindings,
    mesh_functions::{get_world_from_local, mesh_position_local_to_world},
    view_transformations::position_world_to_clip,
    utils::rand_f,
}
#import bevy_render::maths::PI_2
#import bevy_procedural_grass::{
    GrassMaterial,
    identity_matrix, rotate, quadratic_bezier, bezier_tangent,
    grass_material as grass,
};

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,

    @location(3) i_pos: vec4<f32>,
}

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var position = vertex.position;

    var ipos = vertex.i_pos.xyz;

    let width = grass.width * (1.0 - pow(vertex.uv.y, 2.0)) * (0.7 + (1.0 - 0.7) * vertex.uv.y); // TODO: change this
    position.x *= width;

    let p0 = vec2<f32>(0.0);
    let p2 = vec2<f32>(1.0, 0.7);
    var curve = grass.curve;
    var midpoint = 0.5;

    let p1 = vec2<f32>(midpoint, curve);
    let bezier = quadratic_bezier(vertex.uv.y, p0, p1, p2);

    position.y = bezier.y;
    position.z = bezier.x;

    var state = bitcast<u32>(vertex.i_pos.x * 100.0 + vertex.i_pos.y * 20.0 + vertex.i_pos.z * 2.0);
    let facing_angle: f32 = rand_f(&state) * PI_2;
    let facing = vec2<f32>(cos(facing_angle), sin(facing_angle));

    position = rotate(position, facing);
    position += ipos;
    
    var out: VertexOutput;
    out.world_position = mesh_position_local_to_world(
        identity_matrix,
        vec4<f32>(position, 1.0)
    );
    out.position = position_world_to_clip(out.world_position.xyz);
    out.clip_position_unclamped = out.position;
    out.position.z = min(out.position.z, 1.0);
    out.uv = vertex.uv;

    return out;
}