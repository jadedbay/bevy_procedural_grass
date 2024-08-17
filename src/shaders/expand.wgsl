@group(0) @binding(0) var<storage, read> dispatch_counts: array<u32>;
@group(0) @binding(1) var<storage, read> scan_buffer: array<u32>;
@group(0) @binding(2) var<storage, read> block_sums: array<u32>;
@group(0) @binding(3) var<storage, read_write> indices_index: array<u32>;

@compute @workgroup_size(128)
fn expand(
  @builtin(global_invocation_id) global_id: vec3<u32>,
) {
  let idx = global_id.x;
  let group_idx = workgroup_id.x;
  var block_sum: u32;
  if (group_idx > 0u) {
    block_sum = block_sums[group_idx];
  }

  for (var i = 0; i < dispatch_counts[global_idx]; i++) {
    indices_index[scan_buffer[idx] + block_sum + i] = global_idx;
  }
}
