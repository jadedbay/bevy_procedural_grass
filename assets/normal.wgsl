#import bevy_pbr::forward_io::VertexOutput

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    return vec4<f32>(mesh.world_normal, 1.0);
}
