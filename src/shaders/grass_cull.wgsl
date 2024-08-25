#import bevy_render::view::View
#import bevy_procedural_grass::grass_types::{GrassInstance, Aabb};

@group(0) @binding(0) var<uniform> aabb: Aabb;
@group(0) @binding(1) var<storage, read> instances: array<GrassInstance>;
@group(0) @binding(2) var<storage, read_write> vote: array<u32>;
@group(0) @binding(3) var<uniform> view: View;

@compute @workgroup_size(256)
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>,
) {
    let instance = instances[global_id.x];
    if (instance.position.w != 1.0 || !point_in_frustum(instance.position.xyz)) { return; }

    vote[global_id.x] = 1u;
}

fn point_in_frustum(point: vec3<f32>) -> bool {
    for (var i = 0u; i < 6u; i++) {
        let plane = view.frustum[i];
        if (dot(vec4<f32>(point, 1.0), vec4<f32>(plane.xyz, plane.w + 1.0)) < 0.0) {
            return false;
        }
    }
    return true;
}
