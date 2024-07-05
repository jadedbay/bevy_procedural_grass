@group(0) @binding(0)
var<storage, read> positions: array<vec4<f32>>;

@compute @workgroup_size(64)
fn main(@builtin(local_invocation_id) local_id: vec3<u32>) {
    let x = local_id.x;

    return;
}