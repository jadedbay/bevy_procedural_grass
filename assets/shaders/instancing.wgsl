#import bevy_pbr::mesh_functions  mesh_position_local_to_clip
#import bevy_pbr::mesh_bindings   mesh

struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,

    @location(3) i_pos: vec3<f32>,
};

struct Color {
    ao: vec4<f32>,
    color_1: vec4<f32>,
    color_2: vec4<f32>,
    tip: vec4<f32>,
};
@group(2) @binding(0)
var<uniform> color: Color;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    let position = vertex.position + vertex.i_pos.xyz;
    var out: VertexOutput;
    out.clip_position = mesh_position_local_to_clip(
        mesh.model, 
        vec4<f32>(position, 1.0)
    );
    out.color = vec4(0.0, 1.0, 0.0, 1.0);
    out.uv = 1.0 - vertex.uv;
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let color_gradient = mix(color.color_1, color.color_2, in.uv.y);

    let ao = mix(color.ao, vec4<f32>(1.0, 1.0, 1.0, 1.0),  in.uv.y);
    let tip = mix(vec4<f32>(0.0, 0.0, 0.0, 0.0), color.tip,  in.uv.y *  in.uv.y);

    let final_color = (color_gradient + tip) * ao;
    
    return final_color;
}