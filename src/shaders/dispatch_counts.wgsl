@group(0) @binding(0) var<storage, read> positions: array<vec4<f32>>;
@group(0) @binding(1) var<storage, read> indices: array<u32>;
@group(0) @binding(2) var<storage, read_write> dispatch_counts: array<f32>;
@group(0) @binding(3) var<uniform> density: f32;

@compute @workgroup_size(64)
fn main(
  @builtin(global_invocation_id) global_id: vec3<u32>,
) {
  let v0 = positions[indices[global_id * 3]].xyz;
  let v1 = positions[indices[global_id * 3 + 1]].xyz;
  let v2 = positions[indices[global_id * 3 + 2]].xyz;

  let area = length(cross(v1 - v0, v2 - v0)) / 2.0;
  let blade_count = ceil(density * area);
  let dispatch_count = u32(ceil(blade_count / 16.0));

  dispatch_counts[global_id] = dispatch_count;
}
