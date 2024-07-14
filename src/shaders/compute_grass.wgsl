@group(0) @binding(0)
var<storage, read> positions: array<vec4<f32>>;
@group(1) @binding(0)
var<storage, read> indices: array<f32>;
@group(2) @binding(0)
var<storage, read_write> output: array<f32, 8>;

@compute @workgroup_size(64)
fn main(@builtin(local_invocation_id) local_id: vec3<u32>) {
    output = array<f32, 8>(0.0, 0.0, 0.0, 0.0, 2.0, 3.0, 5.0, 0.0);

    return;
}