#import bevy_pbr::utils::rand_f

struct GrassInstanceData {
    position: vec4<f32>,
    normal: vec4<f32>,
}

struct Aabb {
    min: vec3<f32>,
    _padding: f32,
    max: vec3<f32>,
    _padding2: f32,
}

@group(0) @binding(0) var<storage, read> positions: array<vec3<f32>>;
@group(0) @binding(1) var<storage, read> indices: array<u32>;

@group(1) @binding(0) var<uniform> aabb: Aabb;
@group(1) @binding(1) var<storage, read> indices_index: array<u32>;
@group(1) @binding(2) var<storage, read_write> vote: array<u32>;
@group(1) @binding(3) var<storage, read_write> output: array<GrassInstanceData>;

@compute @workgroup_size(8)
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>, 
    @builtin(local_invocation_id) local_id: vec3<u32>, 
    @builtin(workgroup_id) workgroup_id: vec3<u32>
) {
    let v0 = positions[indices[indices_index[workgroup_id.x] * 3]].xyz;
    let v1 = positions[indices[indices_index[workgroup_id.x] * 3 + 1]].xyz;
    let v2 = positions[indices[indices_index[workgroup_id.x] * 3 + 2]].xyz;

    let area = length(cross(v1 - v0, v2 - v0)) / 2.0;
    let scaled_density = u32(ceil(12.0 * area));
    if (scaled_density < local_id.x) {
        return;
    }

    let normal = normalize(cross(v1 - v0, v2 - v0));

    var state: u32 = global_id.x;
    let r1 = sqrt(rand_f(&state));
    let r2 = rand_f(&state);
    let r = vec3<f32>(1.0 - r1, r1 * (1.0 - r2), r1 * r2);

    let position = (v0 * r.x + v1 * r.y + v2 * r.z);

    if (!point_in_aabb(position, aabb)) {
        return;
    }

    output[global_id.x] = GrassInstanceData(vec4<f32>(position, 0.0), vec4<f32>(normal, 0.0)); 
    vote[global_id.x] = 1u;

    return;
}

fn point_in_aabb(point: vec3<f32>, aabb: Aabb) -> bool {
    return all(point >= aabb.min && point <= aabb.max);
}
