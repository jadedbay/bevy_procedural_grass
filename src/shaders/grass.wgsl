#import bevy_pbr::mesh_functions::{get_world_from_local, mesh_position_local_to_clip}

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(2) uv: vec2<f32>,

    @location(3) i_pos: vec4<f32>
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var position = vertex.position + vertex.i_pos.xyz;
    let width = 0.05 * (1.0 - pow(vertex.uv.y, 2.0));
    position.x *= width;

    var out: VertexOutput;
    out.clip_position = mesh_position_local_to_clip(
        get_world_from_local(0u),
        vec4<f32>(position, 1.0)
    );
    out.color = vec4<f32>(1.0, 1.0, 0.5, 1.0);
    return out;
}

@fragment fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}