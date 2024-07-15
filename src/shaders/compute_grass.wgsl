#import bevy_pbr::utils::rand_f

@group(0) @binding(0)
var<storage, read> positions: array<vec4<f32>>;
@group(1) @binding(0)
var<storage, read> indices: array<u32>;
@group(2) @binding(0)
var<storage, read_write> output: array<vec4<f32>>;

@compute @workgroup_size(32)
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>, 
    @builtin(local_invocation_id) local_id: vec3<u32>, 
    @builtin(workgroup_id) workgroup_id: vec3<u32>
) {
    let v0 = positions[indices[workgroup_id.x * 3]].xyz;
    let v1 = positions[indices[workgroup_id.x * 3 + 1]].xyz;
    let v2 = positions[indices[workgroup_id.x * 3 + 2]].xyz;

    var state: u32 = global_id.x;
    let r1 = sqrt(rand_f(&state));
    let r2 = rand_f(&state);
    let r = vec3<f32>(1.0 - r1, r1 * (1.0 - r2), r1 * r2);

    let position = (v0 * r.x + v1 * r.y + v2 * r.z);

    output[global_id.x] = vec4<f32>(position, 0.0); 

    return;
}