#import bevy_render::view::View
#import bevy_procedural_grass::{GrassInstance, GrassConfig};

@group(0) @binding(0) var<storage, read> instances: array<GrassInstance>;
@group(0) @binding(1) var<storage, read_write> vote: array<u32>;
@group(0) @binding(2) var<uniform> view: View;
@group(0) @binding(3) var<uniform> config: GrassConfig;
@group(0) @binding(4) var<storage, read_write> lod_vote: array<u32>;
#ifdef SHADOW
    @group(0) @binding(5) var<storage, read_write> shadow_vote: array<u32>;
#endif


@compute @workgroup_size(256)
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>,
) {
    let instance = instances[global_id.x];
    let in_frustum = point_in_frustum(instance.position.xyz);
    // vote[global_id.x] = u32(in_frustum);

    let distance = length(instance.position.xyz - view.world_position);
    if (distance < config.lod_distance) {
        vote[global_id.x] = u32(in_frustum);
        lod_vote[global_id.x] = 0u;
    } else {
        vote[global_id.x] = 0u;
        lod_vote[global_id.x] = u32(in_frustum);
    }
    #ifdef SHADOW
        shadow_vote[global_id.x] = u32(in_frustum && distance < config.shadow_distance);
    #endif
}

fn point_in_frustum(point: vec3<f32>) -> bool {
    for (var i = 0u; i < 6u; i++) {
        let plane = view.frustum[i];
        if (dot(vec4<f32>(point, 1.0), vec4<f32>(plane.xyz, plane.w + 1.5)) < 0.0) {
            return false;
        }
    }
    return true;
}
