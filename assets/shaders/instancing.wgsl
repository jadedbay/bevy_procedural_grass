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
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) y_position: f32, // Add y_position to VertexOutput
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
    out.y_position = position.y; // Pass y position to VertexOutput
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let ao = vec4<f32>(0.11, 0.28, 0.07, 1.0);
    let color_1 = vec4<f32>(0.25, 0.72, 0.27, 1.0);
    let color_2 = vec4<f32>(0.32, 0.85, 0.49, 1.0);
    let tip = vec4<f32>(0.56, 0.95, 0.66, 1.0);

    // Normalize y position to range [0, 1]
    let y_normalized = clamp(in.y_position, 0.0, 1.0); 

    // Create a gradient from color_1 to color_2 based on the y position
    let color_gradient = mix(color_1, color_2, y_normalized); 

    // Additively blend top of mesh with tip and bottom with ao
    //let final_color = mix(mix(ao, color_gradient, y_normalized), tip, y_normalized);

    return color_gradient;
}